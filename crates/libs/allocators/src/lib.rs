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
    allocators::{
        fixed_bitmap::FixedBitMap, kernel_heap_pages::KernelHeapPages,
    },
    composition::{
        layout_normalizer::LayoutNormalizer, linked_chain::LinkedChain,
        locked_global_alloc::LockedGlobalAlloc, size_deciding::SizeDeciding,
    },
};
use core::alloc::Layout;
use x86_64::structures::paging::{PageSize, Size4KiB};

pub mod allocators;
pub mod composition;
pub mod traits;
pub mod utils;

#[alloc_error_handler]
pub fn alloc_err(l: Layout) -> ! {
    panic!("Allocation error: {:?}", l);
}

pub type KernelAllocator = LayoutNormalizer<
    SizeDeciding<
        LockedGlobalAlloc<
            LinkedChain<
                FixedBitMap<{ 512 }, { (Size4KiB::SIZE as usize) / 512 - 1 }>,
                KernelHeapPages,
            >,
        >,
        KernelHeapPages,
        { 512 },
    >,
>;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: KernelAllocator =
    LayoutNormalizer::new(SizeDeciding::new(
        LockedGlobalAlloc::new(LinkedChain::new(KernelHeapPages)),
        KernelHeapPages,
    ));
