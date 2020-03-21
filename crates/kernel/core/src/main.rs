#![no_std]
#![no_main]

use core::panic::PanicInfo;
use page_usage::PhysicalMemoryMap;
use uefi::table::{Runtime, SystemTable};

#[no_mangle]
pub extern "sysv64" fn _start(
    _st: SystemTable<Runtime>,
    map: PhysicalMemoryMap,
) -> ! {
    assert_ne!(map.pages(), 0);

    panic!("Test");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    qemu_exit::x86::exit::<u32, { 0xf4 }>((-1i32) as u32)
}
