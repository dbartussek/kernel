#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use core::panic::PanicInfo;
use cpu_local_storage::get_core_id;
use interrupt_handling::perform_system_call;
use log::*;
use page_management::physical::map::PhysicalMemoryMap;
use parameters::KernelArguments;
use raw_cpuid::*;
use serial_io::*;
use x86_64::instructions::{
    interrupts,
    interrupts::{enable_interrupts_and_hlt, int3},
};

/// Import the global allocator from the allocators crate.
///
/// This import has a side effect.
#[allow(unused_imports)]
use allocators::GLOBAL_ALLOCATOR;

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

    interrupts::enable();

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
    info!("After breakpoint");

    let syscall_result = perform_system_call(0, 0x22, 0x33, 0x44, 0x55, 0x66);
    info!("Performed system call: {:#X?}", syscall_result);

    enable_interrupts_and_hlt();

    info!("CPUID 0x00: {:?}", cpuid!(0));
    info!("TscInfo: {:#?}", CpuId::new().get_tsc_info());
    info!(
        "ProcessorFrequencyInfo: {:#?}",
        CpuId::new().get_processor_frequency_info()
    );

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
