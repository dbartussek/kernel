#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use core::{cell::UnsafeCell, ops::Range, ptr::slice_from_raw_parts_mut};
use goblin::elf::{
    program_header::{ProgramHeader, PT_LOAD},
    Elf,
};
use log::*;
use uefi::{
    prelude::*,
    proto::media::file::{File, FileAttribute, FileMode, FileType},
    table::boot::{AllocateType, MemoryType},
};
use x86_64::structures::paging::{PageSize, Size4KiB};

fn find_protocols<P>(bt: &BootServices) -> Vec<&UnsafeCell<P>>
where
    P: uefi::proto::Protocol,
{
    let handles = bt
        .find_handles::<uefi::proto::media::fs::SimpleFileSystem>()
        .unwrap()
        .log();
    let mut result = Vec::with_capacity(handles.len());

    for h in handles {
        result.push(bt.handle_protocol(h).unwrap().log());
    }

    result
}

fn read_kernel(st: &SystemTable<Boot>) -> Vec<u8> {
    let file_systems = find_protocols::<uefi::proto::media::fs::SimpleFileSystem>(
        st.boot_services(),
    );

    for fs in file_systems {
        let fs = unsafe { &mut *fs.get() };

        let mut dir = fs.open_volume().unwrap().log();

        let mut buffer = Vec::new();
        loop {
            match dir.read_entry(&mut buffer) {
                Ok(entry) => match entry.log() {
                    Some(entry) => {
                        let attributes = entry.attribute();
                        let name = format!("{}", entry.file_name());
                        if (attributes & FileAttribute::DIRECTORY).is_empty()
                            && name.as_str() == "kernel.elf"
                        {
                            let size = entry.file_size();
                            info!("Found kernel: {}: {} bytes", name, size);

                            let kernel_file = dir
                                .open(&name, FileMode::Read, attributes)
                                .unwrap()
                                .log();
                            match kernel_file.into_type().unwrap().log() {
                                FileType::Dir(_) => unreachable!(),
                                FileType::Regular(mut kernel_file) => {
                                    assert!(size <= (core::usize::MAX as u64));

                                    let mut kernel = vec![0; size as usize];
                                    kernel_file
                                        .read(&mut kernel)
                                        .unwrap()
                                        .log();
                                    return kernel;
                                },
                            }
                        }
                    },
                    None => break,
                },
                Err(e) => match e.data() {
                    Some(size) => buffer.resize(*size, 0),
                    None => panic!("{:?}", e),
                },
            }
        }
    }

    panic!("Could not find kernel");
}

fn elf_address_range<'lt, It>(headers: It) -> Range<usize>
where
    It: IntoIterator<Item = &'lt ProgramHeader>,
{
    headers
        .into_iter()
        .fold(None, |acc: Option<Range<usize>>, it| {
            acc.map(|acc| {
                let vm_range: Range<usize> = it.vm_range();
                (acc.start.min(vm_range.start))..(acc.end.max(vm_range.end))
            })
            .or_else(|| Some(it.vm_range()))
        })
        .unwrap_or(0..0)
}

fn divide_ceil(a: usize, b: usize) -> usize {
    let result = a / b;
    if a % b != 0 {
        result + 1
    } else {
        result
    }
}

fn range_size(r: &Range<usize>) -> usize {
    r.end - r.start
}

fn load_elf64<'buffer>(
    elf: &Elf,
    elf_buffer: &[u8],
    buffer: &'buffer mut [u8],
) -> &'buffer mut [u8] {
    let address_range = elf_address_range(&elf.program_headers);

    for it in buffer.iter_mut() {
        *it = 0;
    }

    for header in elf
        .program_headers
        .iter()
        .filter(|header| header.p_type == PT_LOAD)
    {
        let memory_range = header.vm_range();
        let file_range = header.file_range();

        let memory_base = memory_range.start - address_range.start;
        let memory_size = range_size(&memory_range);
        let file_size = range_size(&file_range);
        let size = memory_size.min(file_size);

        (&mut buffer[memory_base..(memory_base + size)])
            .copy_from_slice(&elf_buffer[file_range]);
    }

    buffer
}

fn allocate_pages(
    bt: &BootServices,
    pages: usize,
) -> Option<&'static mut [u8]> {
    let address = bt
        .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
        .ok()?
        .log();

    let address = address as *mut u8;

    Some(unsafe {
        &mut *slice_from_raw_parts_mut(
            address,
            pages * (Size4KiB::SIZE as usize),
        )
    })
}

#[entry]
fn efi_main(_image: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    st.stdout().reset(false).unwrap().log();

    info!("Initialized");

    let kernel = read_kernel(&st);
    info!("Kernel loaded: {} bytes", kernel.len());

    match Elf::parse(&kernel) {
        Ok(elf) => {
            info!(
                "Detected elf, {}, {}",
                if elf.is_64 { "64" } else { "32" },
                if elf.is_lib { "dyn" } else { "exe" }
            );

            if !elf.is_64 {
                panic!("Kernel is 32 bit");
            }
            if !elf.is_lib {
                panic!("Elf is not relocatable");
            }

            let address_range = elf_address_range(&elf.program_headers);
            let binary_size = address_range.end - address_range.start;
            let page_size = divide_ceil(binary_size, Size4KiB::SIZE as usize);

            info!("Address range: {:X?}", address_range);
            info!(
                "Kernel memory size: 0x{:X}, pages: {}",
                binary_size, page_size
            );

            let buffer = allocate_pages(st.boot_services(), page_size).unwrap();
            info!("Allocated: 0x{:X}", buffer.as_ptr() as usize);

            let buffer = load_elf64(&elf, &kernel, buffer);

            let entry_pointer = {
                let entry_address = (elf.entry as usize) - address_range.start;

                info!("Entry offset 0x{:X}", entry_address);

                &buffer[entry_address] as *const u8
            };

            info!("Loaded kernel, entry: 0x{:X}", entry_pointer as usize);
        },
        _ => panic!("Unknown binary type"),
    }

    loop {}
}
