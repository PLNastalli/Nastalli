use x86_64::instructions::segmentation::{DS, ES, Segment};

pub unsafe fn enter(entry: u64, stack_top: u64) -> ! {
    let (user_code, user_data) = crate::gdt::user_selectors();

    unsafe {
        DS::set_reg(user_data);
        ES::set_reg(user_data);
        core::arch::asm!(
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
