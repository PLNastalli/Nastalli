use x86_64::instructions::segmentation::{DS, ES, Segment};

/// Transfers execution from Ring 0 to Ring 3 using an `iretq` frame.
///
/// # Safety
///
/// `entry` must be a canonical, present, user-accessible executable virtual address and
/// `stack_top` must be a canonical, present, user-accessible writable stack address. The GDT
/// must already be loaded with valid Ring 3 code/data selectors, and the TSS must provide a
/// valid Ring 0 privilege stack for exceptions returning to the kernel.
pub unsafe fn enter(entry: u64, stack_top: u64) -> ! {
    let (user_code, user_data) = crate::gdt::user_selectors();

    unsafe {
        DS::set_reg(user_data);
        ES::set_reg(user_data);
        core::arch::asm!(
            "cli",
            "push {user_data}",
            "push {stack_top}",
            "pushfq",
            "push {user_code}",
            "push {entry}",
            "iretq",
            user_data = in(reg) user_data.0 as u64,
            stack_top = in(reg) stack_top,
            user_code = in(reg) user_code.0 as u64,
            entry = in(reg) entry,
            options(noreturn)
        );
    }
}
