#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel_core::{exit, KernelArguments};
use page_table::KernelPageTable;
use serial_io::*;

#[no_mangle]
pub extern "sysv64" fn _start(args: *mut KernelArguments) -> ! {
    let args = unsafe { args.read() };

    serial_println!("Kernel initialized");

    assert_ne!(args.physical_memory_map.pages(), 0);

    let _args = args.init();

    let mut page_table = KernelPageTable::current_page_table();
    page_table.get_manager();

    exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(-1);
}
