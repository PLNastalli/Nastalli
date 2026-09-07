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
        let context = unsafe {
            super::prepare(
                bottom,
                stack.0.len(),
                dummy_entry,
                core::ptr::null_mut(),
            )
        };
        let start = bottom as u64;
        let end = start + stack.0.len() as u64;

        assert!(context.stack_pointer >= start);
        assert!(context.stack_pointer < end);
        assert_eq!(context.stack_pointer % 16, 0);
    }
}
