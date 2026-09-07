# Architecture

## Current state: v0.0.9 with v0.1.0 context-switch development

Nastalli is intentionally small at its current stage. The workspace is split into focused crates with explicit dependency direction and an experimental userspace contract:

```text
userspace probe
      |
      v
  crates/abi
      |
      v
crates/kernel
      |
      +------> crates/hal
      |           |
      |           v
      +------> crates/arch
                  |
                  v
       hardware / privileged CPU state

crates/boot  -> kernel entry and BootInfo handoff
tools/xtask  -> host-side build, image, test, and run automation
```

`crates/abi` is not a hardware layer. It contains architecture-independent constants and contracts shared across the kernel/userspace boundary. The current ABI is experimental and intentionally tiny.

### `crates/boot`

Owns bootloader integration and the kernel entry point. It receives `BootInfo` from `bootloader_api`, forwards control to the kernel, and contains binary-level boot/panic integration. It should not become a general hardware or policy layer.

### `crates/abi`

Owns the minimal contract that must be shared between userspace and the kernel without importing architecture internals. The current crate is `no_std` and defines the experimental syscall vector used by the first Ring 3 probe.

It does not yet define a stable syscall-number namespace, argument ABI, error model, handle model, or compatibility guarantee.

### `crates/arch`

Owns architecture-specific and privileged x86_64 operations. Current responsibilities include:

- port I/O;
- GDT/TSS and privilege-stack setup;
- IDT/PIC/PIT integration;
- paging operations used by the current probes;
- Ring 0 -> Ring 3 transition;
- the Ring 3-callable syscall interrupt gate;
- low-level kernel context preparation and context switching.

The context-switch mechanism saves/restores `rsp` and the x86_64 SysV callee-saved register set (`rbp`, `rbx`, `r12`-`r15`). Fresh kernel contexts are entered through an architecture trampoline prepared on their own stack.

`arch` contains the narrow unavoidable `unsafe` and assembly boundaries. Scheduler policy remains outside this crate.

### `crates/hal`

Exposes safe hardware-facing interfaces to higher layers. The current HAL remains intentionally small and contains abstractions with real consumers, such as serial and keyboard input.

The HAL must not become an abstract framework for hardware that Nastalli does not yet support.

### `crates/kernel`

Owns kernel policy and architecture-independent core state where practical. Current responsibilities include:

- physical-memory inspection/allocation model;
- static kernel heap;
- task identity/state model;
- round-robin scheduler policy driven by PIT tick accounting;
- startup/runtime orchestration;
- controlled validation of the architecture context-switch mechanism;
- preparation of the current Ring 3 probe.

The kernel does not directly perform port I/O and does not duplicate register-switch assembly. It calls the narrow mechanisms exposed by `arch`.

The current cooperative context-switch probe is deliberately separate from the `Task`/`Scheduler` lifecycle. It proves that the CPU can leave the bootstrap stack, execute on an independent physical-frame-backed kernel stack, return, resume that worker, and return again. It does **not** yet mean scheduler decisions switch real task contexts.

### `tools/xtask`

Runs on the development host with `std` and centralizes build, image creation, tests, QEMU execution, and related tooling. It is not part of the target kernel runtime.

## Current initialization and validation flow

At a high level, the current reference QEMU path is:

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
        +--> GDT/TSS, including Ring 0 privilege stack
        +--> IDT + PIC + 100 Hz PIT
        +--> static kernel heap
        +--> physical-memory inspection
        +--> persistent bootstrap TaskTable
        +--> round-robin scheduler policy
        +--> first real PIT tick observed
        |
        +--> cooperative context-switch probe
        |       |
        |       +--> bootstrap -> worker stack
        |       +--> worker -> bootstrap
        |       +--> bootstrap -> resumed worker
        |       +--> worker -> bootstrap
        |
        +--> allocate/map Ring 3 code and stack
        +--> iretq to Ring 3
                |
                +--> int 0x80 -> Ring 0 syscall gate
                |                 |
                |                 +--> iretq back to Ring 3
                |
                +--> int3 -> Ring 0 breakpoint handler
```

A single monotonic `FrameAllocator` instance is used by the runtime validation path for the worker stack and later Ring 3/page-table allocations. This avoids accidentally handing the same usable physical frame to multiple consumers.

The QEMU smoke test requires the context-switch, syscall-return, and final breakpoint markers. The runtime validation therefore checks both the new kernel-stack transition and the older privilege-transition path in one boot.

## Current execution model

Three pieces exist today but are not fully integrated:

1. **Task model** — identifies tasks and tracks descriptive states.
2. **Scheduler policy** — rotates runnable task state according to a 5-tick round-robin quantum driven by a real 100 Hz PIT counter.
3. **Architecture context switching** — can save/restore a kernel execution context and run a fresh context on an independent stack.

The next scheduler work must connect these pieces so task-owned contexts/stacks are switched by scheduling decisions rather than by a dedicated validation probe.

## Current userspace/ABI boundary

Nastalli can currently:

- map a user-accessible code page and user stack page;
- transition from Ring 0 to Ring 3 through `iretq`;
- accept the experimental software-interrupt syscall vector from Ring 3;
- return from that kernel entry to Ring 3;
- take a subsequent Ring 3 breakpoint through the TSS Ring 0 stack.

This is a privilege-transition and syscall-entry foundation, not yet a general process runtime.

## Current non-goals and missing integration

The following are **not yet implemented**:

- scheduler-driven CPU context switching;
- PIT-driven preemptive execution-context changes;
- generic task-owned kernel stack allocation and teardown;
- saved CPU contexts stored as part of the `Task` lifecycle;
- repeated execution of multiple scheduler-managed task bodies;
- process/thread runtime objects;
- per-process virtual address spaces;
- ELF loading;
- a general syscall dispatcher and stable syscall-number/argument ABI;
- userspace `init` or shell;
- process exit/reaping semantics;
- VFS or persistent filesystems;
- networking;
- USB;
- general graphics or audio stacks;
- SMP/multicore scheduling;
- capability enforcement;
- production security guarantees.

## Architectural principles

### Safe Rust by default

Safe Rust is the default. `unsafe` is reserved for boundaries where hardware, boot integration, privileged state, ABI contracts, raw stack manipulation, or invariants outside the compiler's model require it. See [`unsafe-policy.md`](unsafe-policy.md).

### Explicit ownership

Long-lived kernel state should have an explicit owner. Global state is not forbidden, but it should be introduced only when the architecture requires truly global synchronization or access semantics.

The current context-switch probe intentionally keeps its saved contexts in stable memory rather than relying on a movable/local temporary object while another stack can resume into it. The production task model must make this ownership rule explicit per task.

### Dependency direction

Higher layers should consume narrow interfaces from lower layers rather than reaching through them.

Conceptually:

```text
policy -> abstraction -> mechanism -> hardware
```

Scheduler policy must not absorb x86_64 register/stack assembly. Conversely, `arch` must not decide which task should run next.

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

Nastalli now exposes an **experimental** userspace/kernel contract through `crates/abi`, currently sufficient only for the first syscall-entry proof. It is not stable and may change freely while the kernel/process model is still being developed.

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
