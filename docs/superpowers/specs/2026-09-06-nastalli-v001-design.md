# Nastalli v0.0.1 — Foundation Design

## Goal

Boot a Rust `no_std` kernel on x86_64 through UEFI, emit diagnostics over serial, and produce initial visual output through the framebuffer.

## Components

- `boot` — entry point and `BootInfo` integration contract.
- `arch` — x86_64 assembly and architecture-specific I/O.
- `hal` — safe serial-facing interface.
- `kernel` — initialization logic and initial framebuffer use.
- `xtask` — build, image creation, execution, and test automation.

## Safety and evolution

All crates deny implicit unsafe operations inside unsafe functions through `#![deny(unsafe_op_in_unsafe_fn)]`.

Assembly and privileged architecture-specific operations remain in `arch`; higher layers should not directly depend on ports, registers, or instruction details.

ABI, handles, userspace, and other larger subsystems are added only when real behavior requires them rather than being scaffolded speculatively.

The project follows an owner-controlled direction: no project master key, mandatory telemetry, or mandatory remote-server dependency should be introduced as a condition for ordinary local ownership. Secure Boot, when implemented, should support owner-controlled keys.

`v0.0.1` does not implement cryptography or a complete security model; it only establishes foundations that do not require a central authority.

## Verification criteria

The foundation milestone is validated through the applicable checks:

```text
cargo fmt --check
cargo check
cargo clippy
cargo xtask build
cargo xtask image
QEMU boot with OVMF
```
