#![no_std]
#![no_main]

use core::panic::PanicInfo;
use page_table::IdentityMappedPageTable;
use page_usage::PhysicalMemoryMap;
use uefi::table::{Runtime, SystemTable};

#[no_mangle]
pub extern "sysv64" fn _start(
    _st: SystemTable<Runtime>,
    map: PhysicalMemoryMap,
    _page_table: IdentityMappedPageTable,
) -> ! {
    assert_ne!(map.pages(), 0);

    exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(-1);
}

pub fn exit(status: i32) -> ! {
    qemu_exit::x86::exit::<u32, { 0xf4 }>(status as u32)
}
