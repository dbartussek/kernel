#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use core::panic::PanicInfo;
use cpu_local_storage::get_core_id;
use log::*;
use page_management::physical::map::PhysicalMemoryMap;
use parameters::KernelArguments;
use serial_io::*;

/// Import the global allocator from the allocators crate.
///
/// This import has a side effect.
#[allow(unused_imports)]
use allocators::GLOBAL_ALLOCATOR;
use interrupt_handling::perform_system_call;
use x86_64::instructions::interrupts::int3;

pub fn exit(status: i32) -> ! {
    qemu_exit::x86::exit::<u32, { 0xf4 }>(status as u32)
}

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

    int3();

    let syscall_result = perform_system_call(0, 0x22, 0x33, 0x44, 0x55, 0x66);
    info!("Performed system call: {:#X?}", syscall_result);

    exit(0);
}

fn allocation_test() {
    info!("Testing allocator start");
    let pages = PhysicalMemoryMap::global(|m| m.pages());
    let mut msg = format!("Testing allocator: {}; ", pages);

    for _ in 0..32 {
        msg.push('a');
    }

    info!("{}", msg);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Kernel Panic: {}", info);
    exit(-1);
}
