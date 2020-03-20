#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate alloc;

pub mod load_elf;
pub mod read_kernel;

use crate::{load_elf::load_elf, read_kernel::read_kernel};
use log::*;
use uefi::prelude::*;

#[entry]
fn efi_main(_image: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    st.stdout().reset(false).unwrap().log();

    info!("Initialized");

    let kernel = read_kernel(&st);
    info!("Kernel loaded: {} bytes", kernel.len());

    let (_loaded_kernel, kernel_entry) = load_elf(&kernel, st.boot_services());

    kernel_entry();
    panic!("Kernel ")
}
