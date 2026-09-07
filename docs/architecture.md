# Architecture

## Current state: v0.0.6

Nastalli is intentionally small at its current stage. The workspace is split into a few focused crates with explicit dependency direction:

```text
crates/boot
    |
    v
crates/kernel
    |
    v
crates/hal
    |
    v
crates/arch
    |
    v
hardware / privileged CPU state

 tools/xtask  -> host-side build, image, test, and run automation
```

### `crates/boot`

Owns bootloader integration and the kernel entry point. It receives `BootInfo` from `bootloader_api`, forwards control to the kernel, and contains binary-level boot/panic integration. It should not become a general hardware or policy layer.

### `crates/arch`

Owns architecture-specific and privileged x86_64 operations. This is where low-level port I/O, GDT/TSS setup, IDT/PIC/PIT integration, interrupt details, and other CPU-specific mechanisms belong.

`arch` is expected to contain the narrowest unavoidable `unsafe` and assembly boundaries.

### `crates/hal`

Exposes safe hardware-facing interfaces to higher layers. The current HAL is intentionally small and contains only abstractions that already have real consumers, such as serial and keyboard input.

The HAL must not become an abstract framework for hardware that Nastalli does not yet support.

### `crates/kernel`

Owns kernel policy and architecture-independent core state where practical. The current kernel contains memory, heap, and task-model logic, coordinates initialization, and consumes safe interfaces from lower layers.

The kernel should not directly perform port I/O or scatter architecture-specific privileged operations through policy code.

### `tools/xtask`

Runs on the development host with `std` and centralizes build, image creation, tests, QEMU execution, and related tooling. It is not part of the target kernel runtime.

## Current initialization flow

At a high level, the current boot path is:

```text
UEFI firmware / OVMF
        |
        v
bootloader 0.11.10
        |
        v
crates/boot entry point
        |
        v
nastalli_kernel::start(BootInfo)
        |
        +--> serial diagnostics
        +--> GDT/TSS
        +--> IDT + PIC/PIT infrastructure
        +--> static kernel heap
        +--> physical-memory inspection
        +--> persistent bootstrap TaskTable
        +--> framebuffer paint
        +--> long-running kernel loop
```

The task table created during initialization is returned and moved into the long-lived kernel runtime. This keeps ownership explicit and avoids introducing a global scheduler singleton before the scheduler exists.

## Current non-goals

The following are **not implemented in v0.0.6**:

- preemptive scheduling or context switching;
- processes or threads as separate runtime objects;
- Ring 3 execution;
- a public syscall ABI;
- per-process virtual address spaces;
- VFS or persistent filesystems;
- networking;
- USB;
- general graphics or audio stacks;
- SMP/multicore scheduling;
- capability enforcement;
- production security guarantees.

Roadmap documents may describe these as future targets, but architecture documentation must not present them as current features.

## Architectural principles

### Safe Rust by default

Safe Rust is the default. `unsafe` is reserved for boundaries where hardware, boot integration, privileged state, ABI contracts, or invariants outside the compiler's model require it. See [`unsafe-policy.md`](unsafe-policy.md).

### Explicit ownership

Long-lived kernel state should have an explicit owner. Global state is not forbidden, but it should be introduced only when the architecture requires truly global synchronization or access semantics.

### Dependency direction

Higher layers should consume narrow interfaces from lower layers rather than reaching through them.

Conceptually:

```text
policy -> abstraction -> mechanism -> hardware
```

Architecture-specific types should not leak into generic kernel APIs unless the abstraction would otherwise be artificial or misleading.

### No speculative abstraction

Nastalli does not create empty subsystem crates merely because a mature OS will eventually need those subsystems. New modules and boundaries are introduced when real implementation pressure justifies them.

### Owner-controlled trust

The long-term design treats the machine owner as the final authority over the device. Security mechanisms should protect resources without creating an unavoidable project-controlled remote authority, mandatory project account, or project master key.

See [`security-model.md`](security-model.md).

## Long-term direction

The following diagram describes a **target direction**, not the current implementation:

```text
Applications / Services
        |
        v
Stable Userspace ABI
        |
        v
+--------------------------------+
|         Nastalli Kernel        |
|                                |
|  Scheduler                     |
|  Virtual Memory                |
|  IPC                           |
|  Capability / Handle Model     |
|  VFS                           |
|  Process / Thread Model        |
|  Device Management             |
+---------------+----------------+
                |
               HAL
                |
       +--------+--------+
       |                 |
     x86_64            ARM64
```

The precise split between kernel-space and user-space services or drivers is intentionally not frozen yet. The project can evolve toward stronger isolation as IPC, capability management, scheduling, and driver contracts become mature enough to support it without blocking early development.

## ABI policy

Nastalli does not currently expose a stable userspace ABI. Early interfaces may change freely while the kernel model is still being developed.

Before `v1.0.0`, the project intends to define and validate a stable userspace ABI candidate during the `v0.9.x` stabilization period. Stability guarantees must be documented explicitly before applications are expected to depend on them long term.

## Hardware support policy

QEMU/OVMF x86_64 is the initial reference platform. Real-hardware support will be introduced gradually and documented through support tiers rather than assumed globally.

The roadmap defines the intended Tier 1, Tier 2, Tier 3, and unsupported categories. A production-grade release will be considered stable only for configurations included in its documented support matrix.

## Architectural change policy

A meaningful architectural change should update, in the same development cycle:

- this document;
- the affected subsystem documentation;
- `progress.md` with evidence once behavior is verified;
- `roadmap.md` if milestone scope changes;
- security documentation if trust boundaries change.

Implementation evidence takes precedence over planned architecture.
