#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::format;
use allocators::allocators::kernel_heap_pages::KernelHeapPages;
use core::{alloc::Layout, panic::PanicInfo};
use cpu_local_storage::get_core_id;
use log::*;
use page_management::physical::map::PhysicalMemoryMap;
use parameters::KernelArguments;
use serial_io::*;

pub fn exit(status: i32) -> ! {
    qemu_exit::x86::exit::<u32, { 0xf4 }>(status as u32)
}

#[global_allocator]
static A: KernelHeapPages = KernelHeapPages;

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "sysv64" fn _start(args: *mut KernelArguments) -> ! {
    let args = args.read();

    serial_println!("Kernel starting");

    assert_ne!(args.physical_memory_map.pages(), 0);

    let _args = args.init();

    info!("Kernel initialized");

    info!("Kernel core id: {:?}", get_core_id());

    PhysicalMemoryMap::global(|memory_map| {
        assert_ne!(memory_map.pages(), 0);
        info!(
            "Physical memory pages: 0x{:X}; Available: 0x{:X}",
            memory_map.pages(),
            memory_map.empty_frames(),
        );
    });

    allocation_test();

    PhysicalMemoryMap::global(|_| {
        PhysicalMemoryMap::global(|_| {
            panic!("Recursive lock on PhysicalMemoryMap::global")
        })
    });

    exit(0);
}

fn allocation_test() {
    info!("Testing allocator start");
    let pages = PhysicalMemoryMap::global(|m| m.pages());
    let msg = format!("Testing allocator: {}", pages);
    info!("{}", msg);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Kernel Panic: {}", info);
    exit(-1);
}

#[alloc_error_handler]
fn alloc_err(l: Layout) -> ! {
    info!("Allocation error: {:?}", l);
    exit(-1);
}
