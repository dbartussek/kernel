use crate::traits::Allocator;
use core::{
    alloc::{AllocErr, AllocRef, GlobalAlloc, Layout},
    ptr::NonNull,
};
use log::*;
use num_integer::Integer;
use page_management::{
    page_table::managed_page_table::{
        kernel_heap_range, ManagedPageTable, ModificationFlags,
    },
    physical::page_usage::PageUsage,
};
use x86_64::{
    structures::paging::{
        page::PageRange, Page, PageSize, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

#[derive(Default, Copy, Clone, Debug)]
pub struct KernelHeapPages;

fn layout_to_page_layout(layout: Layout) -> Result<(Layout, usize), AllocErr> {
    let page_size = Size4KiB::SIZE as usize;

    let layout = layout
        .align_to(layout.align().div_ceil(&page_size) * page_size)
        .map_err(|_| AllocErr)?
        .pad_to_align();
    let size = layout.size() / page_size;

    assert_eq!(layout.size() % page_size, 0);

    Ok((layout, size))
}

unsafe impl GlobalAlloc for KernelHeapPages {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        AllocRef::alloc(&mut KernelHeapPages, layout)
            .map(|r| r.0.as_ptr())
            .unwrap_or(core::ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(ptr) = NonNull::new(ptr) {
            AllocRef::dealloc(&mut KernelHeapPages, ptr, layout);
        }
    }
}

unsafe impl AllocRef for KernelHeapPages {
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> Result<(NonNull<u8>, usize), AllocErr> {
        let (layout, pages) = layout_to_page_layout(layout)?;

        let mapped_pages = ManagedPageTable::modify_global(
            ModificationFlags {
                kernel_heap: true,
                ..Default::default()
            },
            move |manager| -> Result<PageRange<Size4KiB>, AllocErr> {
                let desired_pages = manager
                    .find_free_pages_in_range(
                        kernel_heap_range(),
                        pages as u64,
                        ((layout.align() as u64) / Size4KiB::SIZE).max(1),
                    )
                    .ok_or(AllocErr)?;

                unsafe {
                    manager.map_blank_pages(
                        desired_pages.start,
                        pages,
                        PageTableFlags::PRESENT
                            | PageTableFlags::WRITABLE
                            | PageTableFlags::NO_EXECUTE,
                        true,
                        PageUsage::KernelHeap,
                    )
                }
                .map_err(|_| AllocErr)
                .map(|_| desired_pages)
            },
        )?;

        let ptr = NonNull::new(mapped_pages.start.start_address().as_mut_ptr())
            .unwrap();

        trace!("KernelHeapPages allocated {} pages at {:?}", pages, ptr);

        Ok((ptr, layout.size()))
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let (_, pages) = layout_to_page_layout(layout).unwrap();
        let start = Page::<Size4KiB>::from_start_address(VirtAddr::from_ptr(
            ptr.as_ptr(),
        ))
        .unwrap();
        let range = PageRange {
            start,
            end: start + (pages as u64),
        };

        ManagedPageTable::modify_global(
            ModificationFlags {
                kernel_heap: true,
                ..Default::default()
            },
            move |manager| {
                manager.unmap_pages_and_release(range, true).unwrap();
            },
        );
    }
}

impl Allocator for KernelHeapPages {
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> Result<(NonNull<u8>, usize), AllocErr> {
        AllocRef::alloc(self, layout)
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        AllocRef::dealloc(self, ptr, layout)
    }
}
