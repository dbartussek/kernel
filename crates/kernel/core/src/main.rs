#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel_core::KernelArguments;
use page_table::KernelPageTable;

#[no_mangle]
pub extern "sysv64" fn _start(mut args: KernelArguments) -> ! {
    assert_ne!(args.physical_memory_map.pages(), 0);

    args.init();

    let mut page_table = KernelPageTable::current_page_table();
    page_table.get_manager();

    exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(-1);
}

pub fn exit(status: i32) -> ! {
    qemu_exit::x86::exit::<u32, { 0xf4 }>(status as u32)
}
