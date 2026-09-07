use core::{arch::global_asm, ptr};

const SAVED_WORDS: usize = 8;
const WORD_SIZE: usize = core::mem::size_of::<u64>();
const INITIAL_FRAME_SIZE: usize = SAVED_WORDS * WORD_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Context {
    pub stack_pointer: u64,
}

impl Context {
    pub const fn empty() -> Self {
        Self { stack_pointer: 0 }
    }
}

global_asm!(
    r#"
    .global nastalli_context_switch
    .type nastalli_context_switch,@function
nastalli_context_switch:
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15
    mov [rdi], rsp
    mov rsp, [rsi]
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret
    .size nastalli_context_switch, .-nastalli_context_switch

    .global nastalli_context_trampoline
    .type nastalli_context_trampoline,@function
nastalli_context_trampoline:
    mov rdi, r12
    jmp r13
    .size nastalli_context_trampoline, .-nastalli_context_trampoline
"#
);

unsafe extern "C" {
    fn nastalli_context_switch(current_stack_pointer: *mut u64, next_stack_pointer: *const u64);
    fn nastalli_context_trampoline();
}

/// Builds the saved register frame required to enter a fresh kernel context.
///
/// # Safety
///
/// `stack_bottom..stack_bottom + stack_len` must be valid writable memory for the lifetime of
/// the returned context. `entry` and `argument` must remain valid until the context starts.
pub unsafe fn prepare(
    stack_bottom: *mut u8,
    stack_len: usize,
    entry: extern "C" fn(*mut ()) -> !,
    argument: *mut (),
) -> Context {
    let start = stack_bottom as usize;
    let end = start
        .checked_add(stack_len)
        .expect("kernel context stack address overflow");
    let aligned_end = end & !0xf;
    assert!(
        aligned_end >= start.saturating_add(INITIAL_FRAME_SIZE),
        "kernel context stack is too small"
    );

    let frame = (aligned_end - INITIAL_FRAME_SIZE) as *mut u64;
    unsafe {
        ptr::write(frame.add(0), 0); // r15
        ptr::write(frame.add(1), 0); // r14
        ptr::write(frame.add(2), entry as usize as u64); // r13
        ptr::write(frame.add(3), argument as usize as u64); // r12
        ptr::write(frame.add(4), 0); // rbx
        ptr::write(frame.add(5), 0); // rbp
        ptr::write(frame.add(6), nastalli_context_trampoline as usize as u64);
        ptr::write(frame.add(7), 0); // fake return slot keeps SysV stack alignment
    }

    Context {
        stack_pointer: frame as u64,
    }
}

/// Saves the current callee-saved register context and restores `next`.
///
/// # Safety
///
/// `current` must stay writable while the switch executes, and `next.stack_pointer` must refer
/// to a frame created by `prepare` or previously saved by this function.
pub unsafe fn switch(current: &mut Context, next: &Context) {
    unsafe {
        nastalli_context_switch(
            &mut current.stack_pointer as *mut u64,
            &next.stack_pointer as *const u64,
        );
    }
}

#[cfg(test)]
mod tests {
    extern "C" fn dummy_entry(_argument: *mut ()) -> ! {
        loop {
            core::hint::spin_loop();
        }
    }

    #[repr(align(16))]
    struct AlignedStack([u8; 4096]);

    #[test]
    fn prepared_context_starts_inside_aligned_stack() {
        let mut stack = AlignedStack([0; 4096]);
        let bottom = stack.0.as_mut_ptr();
        let context =
            unsafe { super::prepare(bottom, stack.0.len(), dummy_entry, core::ptr::null_mut()) };
        let start = bottom as u64;
        let end = start + stack.0.len() as u64;

        assert!(context.stack_pointer >= start);
        assert!(context.stack_pointer < end);
        assert_eq!(context.stack_pointer % 16, 0);
    }
}
