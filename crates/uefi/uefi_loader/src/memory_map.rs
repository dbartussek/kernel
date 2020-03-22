use crate::alloc_utils::{allocate_pages_array, allocate_pages_byte_size};
use alloc::vec::Vec;
use core::ops::Range;
use page_usage::{PageUsage, PageUsageRawType, PhysicalMemoryMap};
use uefi::{
    table::{
        boot::{MemoryDescriptor, MemoryType},
        Boot, Runtime, SystemTable,
    },
    Handle,
};
use x86_64::{
    structures::paging::{PageSize, PhysFrame, Size4KiB},
    PhysAddr,
};

pub fn create_memory_map_vec(st: &SystemTable<Boot>) -> Vec<MemoryDescriptor> {
    let mut buffer = vec![];
    let bt = st.boot_services();

    loop {
        buffer.resize(bt.memory_map_size(), 0);

        match bt.memory_map(buffer.as_mut_slice()) {
            Ok(r) => {
                let (_, iter) = r.log();

                let mut memory_info: Vec<MemoryDescriptor> =
                    iter.copied().collect();

                memory_info.sort_unstable_by_key(|it| it.phys_start);

                return memory_info;
            },
            _ => (),
        }
    }
}

fn physical_memory_range(memory: MemoryDescriptor) -> Range<usize> {
    let start = memory.phys_start as usize;
    let size = (memory.page_count as usize) * (Size4KiB::SIZE as usize);

    start..(start + size)
}

fn physical_range(memory: &[MemoryDescriptor]) -> Range<usize> {
    let start = memory
        .first()
        .copied()
        .map(physical_memory_range)
        .map(|r| r.start)
        .unwrap_or(0);
    let end = memory
        .last()
        .copied()
        .map(physical_memory_range)
        .map(|r| r.end)
        .unwrap_or(0);

    start..end
}

pub fn enter_descriptor_into_memory_map(
    memory: MemoryDescriptor,
    map: &mut PhysicalMemoryMap,
) {
    let usage = match memory.ty {
        MemoryType::CONVENTIONAL => PageUsage::Empty,
        _ => PageUsage::Unusable,
    };

    let base = PhysFrame::containing_address(PhysAddr::new(memory.phys_start));

    for index in 0..memory.page_count {
        let frame = base + index;

        // If we don't have more specific information, set the page to unusable, just in case
        if matches!(
            map.get(frame),
            None | Some(PageUsage::Empty) | Some(PageUsage::Unusable)
        ) {
            map.set(frame, usage);
        }
    }
}

pub fn create_physical_memory_map(
    st: &SystemTable<Boot>,
) -> PhysicalMemoryMap<'static> {
    let memory_info = create_memory_map_vec(st);
    let physical_range = physical_range(&memory_info);

    let physical_base = PhysFrame::containing_address(PhysAddr::new(
        physical_range.start as u64,
    ));
    let physical_end =
        PhysFrame::containing_address(PhysAddr::new(physical_range.end as u64));

    let physical_size = physical_end - physical_base + 1;

    let buffer = allocate_pages_array::<PageUsageRawType>(
        st.boot_services(),
        physical_size as usize,
    )
    .unwrap();

    let mut map =
        PhysicalMemoryMap::new(buffer, physical_base, PageUsage::Unusable);

    for memory in memory_info.iter() {
        enter_descriptor_into_memory_map(*memory, &mut map);
    }

    map
}

pub fn exit_boot_services(
    image: Handle,
    st: SystemTable<Boot>,
    map: &mut PhysicalMemoryMap,
) -> SystemTable<Runtime> {
    let map_size = st.boot_services().memory_map_size();
    let mut buffer = allocate_pages_byte_size(
        st.boot_services(),
        map_size + (core::mem::size_of::<MemoryDescriptor>() * 5),
    )
    .unwrap();
    let (st, memory_iter) =
        st.exit_boot_services(image, &mut buffer).unwrap().unwrap();

    for it in memory_iter {
        enter_descriptor_into_memory_map(*it, map);
    }

    st
}
