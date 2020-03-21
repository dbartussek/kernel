#![no_std]
#![no_main]

use core::panic::PanicInfo;
use page_usage::PhysicalMemoryMap;

#[no_mangle]
pub extern "sysv64" fn _start(map: PhysicalMemoryMap) -> ! {
    assert_ne!(map.pages(), 0);

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
