#![allow(incomplete_features)]
#![feature(allocator_api)]
#![no_std]
#![feature(const_generics)]
#![feature(const_fn)]
#![feature(ptr_offset_from)]
#![feature(leading_trailing_ones)]
#![feature(alloc_layout_extra)]

pub mod allocators;
pub mod composition;
pub mod traits;
pub mod utils;
