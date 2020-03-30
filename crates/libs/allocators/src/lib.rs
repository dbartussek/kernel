#![allow(incomplete_features)]
#![feature(allocator_api)]
#![no_std]
#![feature(const_generics)]
#![feature(const_fn)]
#![feature(ptr_offset_from)]
#![feature(leading_trailing_ones)]
#![feature(alloc_layout_extra)]
#![feature(alloc_error_handler)]

use crate::{
    allocators::kernel_heap_pages::KernelHeapPages,
    composition::layout_normalizer::LayoutNormalizer,
};
use core::alloc::Layout;

pub mod allocators;
pub mod composition;
pub mod traits;
pub mod utils;

#[alloc_error_handler]
pub fn alloc_err(l: Layout) -> ! {
    panic!("Allocation error: {:?}", l);
}

pub type KernelAllocator = LayoutNormalizer<KernelHeapPages>;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: KernelAllocator =
    LayoutNormalizer::new(KernelHeapPages);
