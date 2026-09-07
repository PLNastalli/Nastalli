## Summary

<!-- What changed, and what concrete problem does this solve? -->

## Verification

Check the commands that apply to this change:

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo xtask test`
- [ ] `cargo check -p nastalli-arch -p nastalli-hal -p nastalli-kernel -p xtask`
- [ ] `cargo clippy -p nastalli-arch -p nastalli-hal -p nastalli-kernel -p xtask -- -D warnings`
- [ ] `cargo xtask build`
- [ ] `cargo xtask run` (required when the change affects target boot/runtime or hardware behavior)

## Architecture and safety

- [ ] I updated the relevant documentation when behavior, architecture, build requirements, security assumptions, or roadmap state changed.
- [ ] Every new `unsafe` block has a documented invariant and is placed at the narrowest appropriate boundary.
- [ ] I did not introduce architecture-specific hardware access into generic kernel policy without a documented reason.
- [ ] I documented known limitations, risks, and unsupported behavior below.

## Security impact

<!-- Describe privilege, memory-safety, input-validation, trust-boundary, persistence, or data-integrity impact. Write "None" if not applicable. -->

## Notes

<!-- Include relevant serial output, test evidence, limitations, risks, migration concerns, or follow-up work. -->
