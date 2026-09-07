use core::cell::UnsafeCell;
use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

pub(crate) const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const DOUBLE_FAULT_STACK_SIZE: usize = 4096;
const KERNEL_INTERRUPT_STACK_SIZE: usize = 16 * 1024;

#[repr(align(16))]
struct WritableStack<const N: usize>(UnsafeCell<[u8; N]>);

impl<const N: usize> WritableStack<N> {
    const fn new() -> Self {
        Self(UnsafeCell::new([0; N]))
    }

    fn top(&self) -> VirtAddr {
        let start = self.0.get().cast::<u8>() as u64;
        VirtAddr::new(start + N as u64)
    }
}

// These stacks are written by the CPU while switching privilege levels or
// entering an IST. UnsafeCell keeps their backing storage in writable memory;
// Rust never creates shared references to the bytes while the CPU uses them.
unsafe impl<const N: usize> Sync for WritableStack<N> {}

static DOUBLE_FAULT_STACK: WritableStack<DOUBLE_FAULT_STACK_SIZE> = WritableStack::new();
static KERNEL_INTERRUPT_STACK: WritableStack<KERNEL_INTERRUPT_STACK_SIZE> = WritableStack::new();

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = DOUBLE_FAULT_STACK.top();
        tss.privilege_stack_table[0] = KERNEL_INTERRUPT_STACK.top();
        tss
    };
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors {
            code_selector,
            data_selector,
            user_data_selector,
            user_code_selector,
            tss_selector,
        })
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
