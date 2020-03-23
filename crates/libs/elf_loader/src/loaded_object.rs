use x86_64::{
    structures::paging::{page::PageRange, Size4KiB},
    VirtAddr,
};

pub struct LoadedObject {
    pub memory: PageRange<Size4KiB>,
    pub relocation_location: PageRange<Size4KiB>,
    pub entry: VirtAddr,
}
