use core::ops::Range;
use goblin::elf::{program_header::PT_LOAD, Elf};
use x86_64::VirtAddr;

pub fn elf_address_range(elf: &Elf) -> Range<VirtAddr> {
    elf.program_headers
        .iter()
        .filter(|header| header.p_type == PT_LOAD)
        .fold(None, |acc: Option<Range<VirtAddr>>, it| {
            acc.map(|acc| {
                let vm_range: Range<usize> = it.vm_range();
                (acc.start.min(VirtAddr::new(vm_range.start as u64)))
                    ..(acc.end.max(VirtAddr::new(vm_range.end as u64)))
            })
            .or_else(|| {
                Some({
                    let range = it.vm_range();
                    VirtAddr::new(range.start as u64)
                        ..VirtAddr::new(range.end as u64)
                })
            })
        })
        .unwrap_or(VirtAddr::new(0)..VirtAddr::new(0))
}
