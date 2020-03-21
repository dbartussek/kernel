use core::ptr::slice_from_raw_parts_mut;
use uefi::table::boot::{AllocateType, BootServices, MemoryType};
use x86_64::structures::paging::{PageSize, Size4KiB};

pub fn divide_ceil(a: usize, b: usize) -> usize {
    let result = a / b;
    if a % b != 0 {
        result + 1
    } else {
        result
    }
}

pub fn bytes_to_pages(bytes: usize) -> usize {
    divide_ceil(bytes, Size4KiB::SIZE as usize)
}

pub fn allocate_pages(
    bt: &BootServices,
    pages: usize,
) -> Option<&'static mut [u8]> {
    let address = bt
        .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
        .ok()?
        .log();

    let address = address as *mut u8;

    Some(unsafe {
        &mut *slice_from_raw_parts_mut(
            address,
            (Size4KiB::SIZE as usize) * pages,
        )
    })
}

pub fn allocate_pages_byte_size(
    bt: &BootServices,
    bytes: usize,
) -> Option<&'static mut [u8]> {
    allocate_pages(bt, bytes_to_pages(bytes))
}

pub fn allocate_pages_array<T>(
    bt: &BootServices,
    length: usize,
) -> Option<&'static mut [T]>
where
    T: Copy,
{
    let raw: &'static mut [u8] =
        allocate_pages_byte_size(bt, length * core::mem::size_of::<T>())?;

    let ptr = raw.as_mut_ptr();

    let slice =
        unsafe { &mut *slice_from_raw_parts_mut(ptr as *mut T, length) };

    Some(slice)
}
