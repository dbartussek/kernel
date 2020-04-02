use lazy_static::*;
use log::*;
use x86_64::{
    instructions::{
        segmentation::{load_ss, set_cs},
        tables::load_tss,
    },
    structures::{
        gdt::{
            Descriptor, DescriptorFlags, GlobalDescriptorTable, SegmentSelector,
        },
        tss::TaskStateSegment,
    },
    VirtAddr,
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let stack_selector = gdt.add_entry({
            Descriptor::UserSegment(
                (DescriptorFlags::USER_SEGMENT
                    | DescriptorFlags::PRESENT
                    | DescriptorFlags::WRITABLE)
                    .bits(),
            )
        });
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                stack_selector,
                tss_selector,
            },
        )
    };
}

pub struct Selectors {
    pub code_selector: SegmentSelector,
    pub stack_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

pub unsafe fn init() {
    GDT.0.load();

    trace!("code selector: {:?}", GDT.1.code_selector);
    trace!("tss selector: {:?}", GDT.1.tss_selector);

    set_cs(GDT.1.code_selector);
    load_ss(GDT.1.stack_selector);
    load_tss(GDT.1.tss_selector);
}
