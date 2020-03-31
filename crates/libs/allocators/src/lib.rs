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

/// Allocations larger than this are allocated as pages
const SIZE_PAGE_FALLBACK: usize = 2 << 8;

pub type Bucket<const SIZE: usize, const PAGES: usize> = LinkedChain<
    FixedBitMap<{ SIZE }, { (Size4KiB::SIZE as usize) * PAGES / SIZE }>,
    KernelHeapPages,
>;

pub type DecidingBucket<A, const SIZE: usize, const PAGES: usize> =
    SizeDeciding<Bucket<{ SIZE }, { PAGES }>, A, { SIZE }>;

pub type KernelAllocator = LayoutNormalizer<
    SizeDeciding<
        LockedGlobalAlloc<
            // Bucket size 16
            DecidingBucket<
                // Bucket size 32
                DecidingBucket<
                    // Bucket size 64
                    DecidingBucket<
                        // Bucket size 128
                        DecidingBucket<
                            // Bucket of size 256
                            DecidingBucket<
                                // Fallback bucket, has to cover all sizes up to SIZE_PAGE_FALLBACK
                                Bucket<{ SIZE_PAGE_FALLBACK }, { 1 }>,
                                { 256 },
                                { 1 },
                            >,
                            { 128 },
                            { 1 },
                        >,
                        { 64 },
                        { 1 },
                    >,
                    { 32 },
                    { 1 },
                >,
                { 16 },
                { 1 },
            >,
        >,
        KernelHeapPages,
        { SIZE_PAGE_FALLBACK },
    >,
>;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: KernelAllocator =
    LayoutNormalizer::new(SizeDeciding::new(
        LockedGlobalAlloc::new(SizeDeciding::new(
            LinkedChain::new(KernelHeapPages),
            SizeDeciding::new(
                LinkedChain::new(KernelHeapPages),
                SizeDeciding::new(
                    LinkedChain::new(KernelHeapPages),
                    SizeDeciding::new(
                        LinkedChain::new(KernelHeapPages),
                        SizeDeciding::new(
                            LinkedChain::new(KernelHeapPages),
                            LinkedChain::new(KernelHeapPages),
                        ),
                    ),
                ),
            ),
        )),
        KernelHeapPages,
    ));
