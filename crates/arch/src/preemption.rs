use core::{arch::global_asm, ptr};

const SAVED_GPRS: usize = 15;
const CPU_FRAME_WORDS: usize = 3;
const FRAME_WORDS: usize = SAVED_GPRS + CPU_FRAME_WORDS;
const FRAME_SIZE: usize = FRAME_WORDS * core::mem::size_of::<u64>();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct InterruptContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreemptionContext {
    stack_pointer: u64,
}

impl PreemptionContext {
    pub const fn new(stack_pointer: u64) -> Self {
        Self { stack_pointer }
    }

    pub const fn stack_pointer(self) -> u64 {
        self.stack_pointer
    }

    pub fn as_ptr(self) -> *mut InterruptContext {
        self.stack_pointer as *mut InterruptContext
    }
}

global_asm!(
    r#"
    .global nastalli_timer_interrupt_entry
    .type nastalli_timer_interrupt_entry,@function
nastalli_timer_interrupt_entry:
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    mov rdi, rsp
    and rsp, -16
    call {timer_dispatch}

    mov rsp, rax
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax
    iretq

    .size nastalli_timer_interrupt_entry, .-nastalli_timer_interrupt_entry
"#,
    timer_dispatch = sym crate::interrupts::timer_dispatch,
);

unsafe extern "C" {
    fn nastalli_timer_interrupt_entry();
}

pub(crate) fn timer_entry_address() -> u64 {
    nastalli_timer_interrupt_entry as usize as u64
}

/// Prepares a Ring 0 interrupt-return frame for a task that has not executed yet.
///
/// The prepared task starts with interrupts enabled. The frame intentionally models
/// only same-privilege Ring 0 interrupt return (`RIP`, `CS`, `RFLAGS`); Ring 3
/// preemption requires the larger privilege-transition frame and is not covered by
/// this bring-up primitive.
///
/// # Safety
///
/// `stack_bottom..stack_bottom + stack_len` must remain writable and exclusively
/// owned by the task while the returned context can be scheduled. `entry` and
/// `argument` must remain valid for the lifetime of that task.
pub unsafe fn prepare_kernel_task(
    stack_bottom: *mut u8,
    stack_len: usize,
    entry: extern "C" fn(*mut ()) -> !,
    argument: *mut (),
) -> PreemptionContext {
    let start = stack_bottom as usize;
    let end = start
        .checked_add(stack_len)
        .expect("preemption stack address overflow");
    let aligned_end = end & !0xf;

    // Leave one word above the iretq frame so the resumed entry observes the
    // SysV x86_64 function-entry stack alignment (RSP % 16 == 8).
    let resumed_rsp = aligned_end
        .checked_sub(core::mem::size_of::<u64>())
        .expect("preemption stack too small");
    let frame_start = resumed_rsp
        .checked_sub(FRAME_SIZE)
        .expect("preemption stack too small");
    assert!(frame_start >= start, "preemption stack is too small");

    unsafe {
        ptr::write(resumed_rsp as *mut u64, 0);
        ptr::write(frame_start as *mut InterruptContext, InterruptContext {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rbp: 0,
            rdi: argument as usize as u64,
            rsi: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,
            instruction_pointer: entry as usize as u64,
            code_segment: crate::gdt::kernel_code_selector().0 as u64,
            cpu_flags: 0x202,
        });
    }

    PreemptionContext::new(frame_start as u64)
}

#[cfg(test)]
mod tests {
    use super::{InterruptContext, PreemptionContext};

    extern "C" fn dummy_entry(_argument: *mut ()) -> ! {
        loop {
            core::hint::spin_loop();
        }
    }

    #[repr(align(16))]
    struct AlignedStack([u8; 4096]);

    #[test]
    fn prepared_kernel_task_contains_iret_entry_and_argument() {
        let mut stack = AlignedStack([0; 4096]);
        let argument = 0x1234usize as *mut ();
        let prepared = unsafe {
            super::prepare_kernel_task(stack.0.as_mut_ptr(), stack.0.len(), dummy_entry, argument)
        };
        let frame = unsafe { &*prepared.as_ptr() };
        let resumed_rsp = stack.0.as_ptr() as u64 + stack.0.len() as u64 - 8;

        assert_eq!(frame.instruction_pointer, dummy_entry as usize as u64);
        assert_eq!(frame.rdi, argument as usize as u64);
        assert_ne!(frame.cpu_flags & (1 << 9), 0);
        assert_eq!(frame.stack_pointer, resumed_rsp);
        assert_ne!(frame.stack_segment, 0);
        assert_eq!(
            prepared.stack_pointer() + core::mem::size_of::<InterruptContext>() as u64,
            resumed_rsp
        );
    }

    #[test]
    fn preemption_context_round_trips_stack_pointer() {
        let context = PreemptionContext::new(0x1234);
        assert_eq!(context.stack_pointer(), 0x1234);
    }
}
