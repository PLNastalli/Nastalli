use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const KERNEL_INTERRUPT_STACK_SIZE: usize = 16 * 1024;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        static DOUBLE_FAULT_STACK: [u8; 4096] = [0; 4096];
        let double_fault_stack_start = VirtAddr::from_ptr(&DOUBLE_FAULT_STACK);
        let double_fault_stack_end = double_fault_stack_start + DOUBLE_FAULT_STACK.len();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = double_fault_stack_end;

        static KERNEL_INTERRUPT_STACK: [u8; KERNEL_INTERRUPT_STACK_SIZE] =
            [0; KERNEL_INTERRUPT_STACK_SIZE];
        let kernel_stack_start = VirtAddr::from_ptr(&KERNEL_INTERRUPT_STACK);
        let kernel_stack_end = kernel_stack_start + KERNEL_INTERRUPT_STACK.len();
        tss.privilege_stack_table[0] = kernel_stack_end;

        tss
    };
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                data_selector,
                user_data_selector,
                user_code_selector,
                tss_selector,
            },
        )
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::{
        segmentation::{CS, DS, ES, SS, Segment},
        tables::load_tss,
    };

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        SS::set_reg(GDT.1.data_selector);
        DS::set_reg(GDT.1.data_selector);
        ES::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}

pub fn user_selectors() -> (SegmentSelector, SegmentSelector) {
    (GDT.1.user_code_selector, GDT.1.user_data_selector)
}

#[cfg(test)]
mod tests {
    use x86_64::PrivilegeLevel;

    #[test]
    fn user_selectors_have_ring3_rpl() {
        let (code, data) = super::user_selectors();
        assert_eq!(code.rpl(), PrivilegeLevel::Ring3);
        assert_eq!(data.rpl(), PrivilegeLevel::Ring3);
    }
}
