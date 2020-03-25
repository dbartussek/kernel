#![no_std]
#![no_main]

use core::panic::PanicInfo;
use log::*;
use parameters::KernelArguments;
use serial_io::*;

pub fn exit(status: i32) -> ! {
    qemu_exit::x86::exit::<u32, { 0xf4 }>(status as u32)
}

#[no_mangle]
pub extern "sysv64" fn _start(args: *mut KernelArguments) -> ! {
    let args = unsafe { args.read() };

    serial_println!("Kernel starting");

    assert_ne!(args.physical_memory_map.pages(), 0);

    let _args = args.init();

    info!("Kernel initialized");

    exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(-1);
}
