# Experimental userspace ABI

## Status

The current ABI is an **experimental `v0.0.9` bring-up contract**. It is not stable and may change without compatibility guarantees before the stabilization phase.

The shared target-independent crate is `crates/abi` (`nastalli-abi`). Architecture-specific entry mechanics remain in `crates/arch`.

## Current x86_64 entry path

The first validated userspace-to-kernel path uses software interrupt vector `0x80`:

```text
Ring 3 probe
    |
    | int 0x80
    v
DPL3 IDT gate
    |
    v
x86_64 syscall entry handler
    |
    | iretq return
    v
Ring 3 probe continues
    |
    | int3
    v
Ring 3 breakpoint validation
```

The QEMU smoke test requires both of these serial markers, in addition to the existing boot/scheduler markers:

```text
Ring 3 syscall entered kernel and returned.
Ring 3 probe reached kernel breakpoint.
```

Reaching the second marker after the first demonstrates that the software-interrupt handler returned to Ring 3 successfully.

## What `v0.0.9` proves

- an independent `no_std` ABI crate exists;
- Ring 3 can invoke the reserved experimental syscall entry;
- the x86_64 IDT gate permits invocation from Ring 3;
- privilege transition uses the kernel stack configured through the TSS;
- the handler can return to the original Ring 3 execution context;
- the path is exercised under QEMU/OVMF in CI.

## What it does not provide yet

- no stable syscall numbering or long-term calling convention;
- no general syscall dispatcher;
- no syscall argument validation model;
- no process identity or per-process address space;
- no userspace executable loader;
- no blocking/restart semantics;
- no stable error ABI;
- no `SYSCALL`/`SYSRET` fast path;
- no compatibility guarantee for applications.

Those contracts should be introduced only when the next userspace work creates real consumers for them.

## Boundary rules

- shared userspace/kernel constants and data-layout contracts belong in `nastalli-abi`;
- x86_64 interrupt gates, privilege transitions, assembly, paging mechanics, and CPU state belong in `nastalli-arch`;
- syscall policy and dispatch belong in `nastalli-kernel` once real syscalls are introduced;
- application-facing stability must not be claimed until the project explicitly defines it.
