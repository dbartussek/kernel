pub mod managed_page_table;

use core::sync::atomic::{AtomicUsize, Ordering};
use x86_64::{
    structures::paging::{Page, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

const DEFAULT_IDENTITY_BASE: usize = core::usize::MAX;
pub(crate) static IDENTITY_BASE: AtomicUsize =
    AtomicUsize::new(DEFAULT_IDENTITY_BASE);

pub fn identity_base() -> Page<Size4KiB> {
    let base: usize = IDENTITY_BASE.load(Ordering::Acquire);
    Page::from_start_address(VirtAddr::new(base as u64)).unwrap()
}

/// Get the page that maps to this frame
pub fn identity_page(frame: PhysFrame<Size4KiB>) -> Page<Size4KiB> {
    identity_base()
        + (frame - PhysFrame::from_start_address(PhysAddr::new(0)).unwrap())
}

/// # Safety
/// Every time the system wants to write to a physical page, it uses identity_base.
/// You better be sure this invariant is upheld or there will be nasty bugs.
pub unsafe fn initialize_identity_base(identity_base: Page<Size4KiB>) {
    let previous = IDENTITY_BASE.swap(
        identity_base.start_address().as_u64() as usize,
        Ordering::AcqRel,
    );
    assert_eq!(previous, DEFAULT_IDENTITY_BASE);
}
