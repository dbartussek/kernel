#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use core::cell::UnsafeCell;
use log::*;
use uefi::prelude::*;
use uefi::proto::media::file::{File, FileAttribute, FileMode, FileType};

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
    let file_systems =
        find_protocols::<uefi::proto::media::fs::SimpleFileSystem>(st.boot_services());

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

                            let kernel_file =
                                dir.open(&name, FileMode::Read, attributes).unwrap().log();
                            match kernel_file.into_type().unwrap().log() {
                                FileType::Dir(_) => unreachable!(),
                                FileType::Regular(mut kernel_file) => {
                                    assert!(size <= (core::usize::MAX as u64));

                                    let mut kernel = vec![0; size as usize];
                                    kernel_file.read(&mut kernel).unwrap().log();
                                    return kernel;
                                }
                            }
                        }
                    }
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

#[entry]
fn efi_main(_image: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    st.stdout().reset(false).unwrap().log();

    info!("Initialized");

    let kernel = read_kernel(&st);
    info!("Kernel loaded: {} bytes", kernel.len());

    loop {}
}
