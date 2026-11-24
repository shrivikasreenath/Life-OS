use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut GDT: (GlobalDescriptorTable, Selectors) = (
    GlobalDescriptorTable::new(),
    Selectors {
        code_selector: SegmentSelector(0),
        data_selector: SegmentSelector(0),
        tss_selector: SegmentSelector(0),
    },
);

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS, DS, ES, SS};
    use x86_64::instructions::tables::load_tss;

    unsafe {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(&raw const STACK);
        let stack_end = stack_start + STACK_SIZE as u64;
        TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_end;

        let code_selector = GDT.0.add_entry(Descriptor::kernel_code_segment());
        let data_selector = GDT.0.add_entry(Descriptor::kernel_data_segment());
        let tss_selector = GDT.0.add_entry(Descriptor::tss_segment(&*(&raw const TSS)));
        GDT.1 = Selectors {
            code_selector,
            data_selector,
            tss_selector,
        };

        GDT.0.load();
        CS::set_reg(code_selector);
        DS::set_reg(data_selector);
        ES::set_reg(data_selector);
        SS::set_reg(data_selector);
        load_tss(tss_selector);
    }
}
