#![no_std]
#![no_main]

use core::panic::PanicInfo;
use log::*;
use page_management::physical::map::PhysicalMemoryMap;
use parameters::KernelArguments;
use serial_io::*;

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

    {
        let memory_map = PhysicalMemoryMap::global();
        assert_ne!(memory_map.pages(), 0);
        info!(
            "Physical memory pages: 0x{:X}; Available: 0x{:X}",
            memory_map.pages(),
            memory_map.iter().filter(|page| page.is_empty()).count()
        );
    }

    exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(-1);
}
