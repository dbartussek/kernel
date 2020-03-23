use x86_64::structures::paging::{page::PageRange, Size4KiB};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct PagePermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

pub trait LoadParameters {
    /// Allocate pages for the binary
    ///
    /// The second return value is the location for which the binary should have its relocations applied.
    /// This may be different from the address at which it was allocated.
    /// The bootloader uses this to load the kernel before moving the mapping to high memory.
    fn allocate_pages(
        &mut self,
        pages: usize,
    ) -> Option<(PageRange<Size4KiB>, PageRange<Size4KiB>)>;

    /// Deallocate pages allocated by self
    fn deallocate_pages(&mut self, pages: PageRange<Size4KiB>);

    /// Set the permissions for a page range
    fn set_permissions(
        &mut self,
        pages: PageRange<Size4KiB>,
        permissions: PagePermissions,
    );
}

pub struct AdHocLoadParameters<A, D, SP>
where
    A: FnMut(usize) -> Option<(PageRange<Size4KiB>, PageRange<Size4KiB>)>,
    D: FnMut(PageRange<Size4KiB>),
    SP: FnMut(PageRange<Size4KiB>, PagePermissions),
{
    pub allocate: A,
    pub deallocate: D,
    pub set_permissions: SP,
}

impl<A, D, SP> LoadParameters for AdHocLoadParameters<A, D, SP>
where
    A: FnMut(usize) -> Option<(PageRange<Size4KiB>, PageRange<Size4KiB>)>,
    D: FnMut(PageRange<Size4KiB>),
    SP: FnMut(PageRange<Size4KiB>, PagePermissions),
{
    fn allocate_pages(
        &mut self,
        pages: usize,
    ) -> Option<(PageRange<Size4KiB>, PageRange<Size4KiB>)> {
        (self.allocate)(pages)
    }

    fn deallocate_pages(&mut self, pages: PageRange<Size4KiB>) {
        (self.deallocate)(pages)
    }

    fn set_permissions(
        &mut self,
        pages: PageRange<Size4KiB>,
        permissions: PagePermissions,
    ) {
        (self.set_permissions)(pages, permissions)
    }
}
