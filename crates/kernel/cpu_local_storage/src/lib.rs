#![no_std]

pub mod data;

use crate::data::{CoreId, CpuLocalData};
use x86_64::{registers::model_specific::KernelGsBase, VirtAddr};

pub unsafe fn init_raw<T>(pointer: *mut T) {
    KernelGsBase::write(VirtAddr::from_ptr(pointer));
}

pub unsafe fn read_raw<T>() -> *mut T {
    KernelGsBase::read().as_mut_ptr()
}

pub fn read<F, R>(f: F) -> R
where
    F: FnOnce(&CpuLocalData) -> R,
{
    f(unsafe { &*read_raw() })
}

pub fn write<F, R>(f: F) -> R
where
    F: FnOnce(&mut CpuLocalData) -> R,
{
    f(unsafe { &mut *read_raw() })
}

pub fn get_core_id() -> CoreId {
    read(|data| data.core_id)
}
