# `unsafe` Policy

## Default rule

Safe Rust is the default throughout Nastalli.

`unsafe` is permitted only when it represents a real boundary that the Rust type system cannot express directly, such as:

- privileged CPU state;
- hardware port or MMIO access;
- bootloader-provided memory contracts;
- ABI boundaries;
- low-level context switching;
- raw memory ownership with externally established invariants.

All Rust crates should retain:

```rust
#![deny(unsafe_op_in_unsafe_fn)]
```

This requires unsafe operations to remain explicit even inside `unsafe fn` bodies.

## Current placement

### `crates/arch`

`arch` is the primary home for architecture-specific privileged operations and unavoidable low-level unsafety.

Current responsibilities include areas such as:

- x86_64 port I/O;
- assembly where required;
- GDT/TSS loading;
- privileged register changes;
- PIC/PIT setup and interrupt-controller operations.

Any unsafe hardware function must document the preconditions required for the operation to be valid.

### `crates/hal`

The HAL should expose safe interfaces to higher layers whenever practical.

The purpose of the HAL is not to hide undefined behavior; it is to establish a narrow contract whose implementation owns the hardware-specific invariant.

### `crates/boot`

The boot crate owns bootloader integration and the target entry point. It may participate in unavoidable boot/ABI boundaries but should not accumulate general kernel or device logic.

### `crates/kernel`

Generic kernel policy should avoid direct hardware `unsafe` operations.

When kernel policy requires a privileged mechanism, the preferred approach is to expose a narrow, documented mechanism from `arch` or an appropriate HAL/driver boundary instead of embedding port I/O, assembly, or raw hardware access in policy code.

## Examples of current trust boundaries

### Serial port I/O

Architecture code uses x86 I/O instructions to access COM1. The safety argument depends on running in the expected architecture/environment and using the intended I/O ports.

### GDT/TSS setup

Operations such as loading descriptor tables, updating segment registers, and loading the TSS modify privileged CPU state. Correctness depends on the referenced static structures and selectors remaining valid.

### Interrupt-controller access

PIC/PIT setup and IRQ completion use privileged port operations. The code must preserve ordering and controller-state assumptions documented by the implementation.

### Kernel heap initialization

The allocator is initialized with a kernel-owned static memory region. The unsafe boundary is valid only because the address, lifetime, alignment, initialization order, and size are controlled by the kernel.

### Bootloader framebuffer/memory structures

Memory exposed by `bootloader_api` is trusted according to the bootloader contract. Nastalli must not extend that trust beyond the documented lifetime or ownership guarantees.

## Required review questions

Every new unsafe block should answer:

1. **Why is `unsafe` necessary?**
2. **What exact invariant makes this operation valid?**
3. **Who establishes that invariant?**
4. **How long does the invariant remain valid?**
5. **Can a safe wrapper prevent callers from violating it?**
6. **What test, assertion, or review evidence would catch a broken assumption?**

If these questions do not have specific answers, the unsafe boundary is not ready.

## Prohibited uses

Do not use `unsafe`:

- merely to bypass the borrow checker;
- to avoid designing ownership/lifetime relationships;
- to scatter architecture-specific operations through generic kernel code;
- to expose raw pointers publicly when a safe reference/handle can represent the contract;
- without documenting the relevant safety invariant;
- to optimize code before measurement demonstrates a need and the safety trade-off is justified.

## Future low-level work

Context switching, page-table management, DMA, userspace transitions, syscall entry, SMP, and device drivers will introduce new unsafe boundaries.

Those additions must not weaken the policy. The project should prefer many small, auditable safe interfaces over one broad "unsafe kernel" region.

When a new subsystem introduces meaningful unsafe code, its subsystem documentation and [`progress.md`](progress.md) should record the trust boundary and validation performed.
