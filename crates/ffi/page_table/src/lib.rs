#![no_std]

pub mod kernel_page_table;
pub mod manager;

pub use self::kernel_page_table::KernelPageTable;
use core::sync::atomic::{AtomicUsize, Ordering};
use x86_64::{structures::paging::Page, VirtAddr};

const DEFAULT_IDENTITY_BASE: usize = core::usize::MAX;
pub(crate) static IDENTITY_BASE: AtomicUsize =
    AtomicUsize::new(DEFAULT_IDENTITY_BASE);

pub(crate) fn identity_base() -> Page {
    let base: usize = IDENTITY_BASE.load(Ordering::Acquire);
    Page::containing_address(VirtAddr::new(base as u64))
}

/// # Safety
/// Every time the system wants to write to a physical page, it uses identity_base.
/// You better be sure this invariant is upheld or there will be nasty bugs.
pub unsafe fn initialize(identity_base: Page) {
    let previous = IDENTITY_BASE.swap(
        identity_base.start_address().as_u64() as usize,
        Ordering::AcqRel,
    );
    assert_eq!(previous, DEFAULT_IDENTITY_BASE);
}
