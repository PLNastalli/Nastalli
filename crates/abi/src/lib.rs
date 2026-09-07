#![no_std]

/// Software interrupt vector reserved for the first experimental Nastalli userspace ABI.
///
/// This ABI is intentionally unstable during the `v0.0.x` bring-up series.
pub const SYSCALL_VECTOR: u8 = 0x80;

#[cfg(test)]
mod tests {
    #[test]
    fn syscall_vector_does_not_overlap_cpu_exceptions_or_legacy_pic() {
        assert!(super::SYSCALL_VECTOR >= 48);
    }
}
