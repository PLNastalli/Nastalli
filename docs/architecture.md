# Architecture

## Current state

Nastalli is an experimental Rust operating-system kernel. The current public runtime is `v0.0.9`; `v0.1.0` is under active development and is integrating task execution, scheduling, preemption, and the first persistent userspace.

```text
userspace probe
      |
      v
  crates/abi
      |
      v
crates/kernel
  |       |
  |       +--> crates/hal
  |               |
  +------------> crates/arch
                  |
                  v
       hardware / privileged CPU state

crates/boot  -> bootloader entry and BootInfo handoff
tools/xtask  -> host build, image, test and QEMU automation
```

The dependency direction is deliberate: policy lives above mechanisms. `kernel` decides *what* should run; `arch` implements *how* privileged x86_64 state is changed.

## Crate responsibilities

### `crates/boot`

Owns bootloader integration and transfers `BootInfo` to the kernel. It is intentionally thin and should not become a policy or general hardware layer.

### `crates/abi`

Owns architecture-independent contracts shared across the userspace/kernel boundary. It is `no_std` and currently exposes the experimental software-interrupt syscall vector used by the Ring 3 validation path.

The ABI is not stable. A syscall-number namespace, argument/error model, handle model, compatibility policy, and general dispatcher are still future work.

### `crates/arch`

Owns architecture-specific and privileged x86_64 mechanisms:

- port I/O;
- GDT/TSS and privilege stacks;
- IDT, legacy PIC and PIT integration;
- paging helpers required by current mappings;
- controlled Ring 0 -> Ring 3 transition;
- Ring 3-callable syscall/breakpoint gates;
- kernel context construction and context switching.

The cooperative context-switch primitive preserves `rsp` plus the SysV callee-saved register set (`rbp`, `rbx`, `r12`-`r15`). It deliberately contains assembly and raw stack manipulation that do not belong in scheduler policy.

### `crates/hal`

Provides safe hardware-facing interfaces with real current consumers. The HAL is deliberately small; Nastalli does not add abstractions merely because a future OS might need them.

### `crates/kernel`

Owns architecture-independent policy and long-lived runtime state where practical:

- physical-frame allocation model;
- kernel heap;
- task identity/state/context ownership;
- kernel-stack metadata;
- round-robin scheduler policy;
- conversion of scheduler switch decisions into task-owned context switches;
- startup/runtime orchestration;
- preparation of the experimental Ring 3 path.

`Task` now owns a saved `Context` and optional `KernelStack` metadata. `Scheduler` owns the task table. The scheduler changes runnable state and selects `from/to`; the runtime performs the low-level switch through `arch`.

### `tools/xtask`

Runs on the development host with `std`. It centralizes build, image creation, host tests, QEMU/OVMF execution, and diagnostics. It is not part of the target kernel.

## Current boot and execution path

The reference QEMU path is currently:

```text
UEFI / OVMF
    |
bootloader 0.11.10
    |
crates/boot
    |
nastalli_kernel::start(BootInfo)
    |
    +--> serial
    +--> GDT/TSS
    +--> IDT + PIC + PIT 100 Hz
    +--> heap
    +--> physical-memory allocator
    +--> bootstrap Task
    +--> worker Task + independent kernel stack
    +--> Scheduler (5-tick round robin)
    |
    +--> PIT ticks recorded by IRQ0
    |
    +--> normal kernel code consumes ticks
    |       |
    |       +--> scheduler selects worker
    |       +--> bootstrap -> worker
    |       +--> scheduler selects bootstrap
    |       +--> worker -> bootstrap
    |       +--> repeat one more round trip
    |
    +--> map Ring 3 code/stack
    +--> iretq to Ring 3
            |
            +--> int 0x80 -> kernel -> iretq to Ring 3
            +--> int3 -> kernel breakpoint handler
```

One monotonic `FrameAllocator` instance supplies the scheduler state frame, worker stack, Ring 3 pages, and required page-table frames. This prevents duplicated allocator cursors from returning the same physical frames to different early consumers.

## Task and scheduler ownership

Long-lived runtime state must have a stable owner:

```text
Scheduler
   |
   +--> TaskTable
          |
          +--> Task 0 (bootstrap)
          |      +--> saved Context
          |
          +--> Task 1 (worker)
                 +--> saved Context
                 +--> KernelStack metadata
```

The early scheduler object itself currently resides in a dedicated physical frame so its address remains stable while the active CPU stack changes.

This is a bring-up mechanism, not the final general allocator/lifetime design. Stack freeing, task reaping, dynamic virtual mappings, and resource teardown remain incomplete.

## Interrupt/preemption boundary

The current execution switches are **driven by real PIT tick accounting but are not IRQ-preemptive**.

IRQ0 currently performs the minimal hardware path: record a tick and acknowledge the PIC. Normal kernel execution later consumes those ticks and invokes scheduler policy.

The existing cooperative `arch::context::switch()` must **not** simply be called from the timer handler. Doing so would save the handler's callee-saved state and handler `rsp`, not the complete CPU state of the interrupted task.

Real preemption therefore requires a separate architecture contract:

```text
PIT IRQ0
   |
   v
interrupt/trap entry
   |
   +--> save complete interrupted task state
   +--> acknowledge interrupt at the correct point
   |
   v
kernel scheduler policy
   |
   v
select next runnable task
   |
   v
restore selected interrupt context
   |
   v
iretq
```

The trap-frame format, ownership, privilege transitions, stack selection, and return invariants must be explicit and tested before IRQ-driven context switching is enabled.

## Current userspace boundary

Nastalli currently proves that it can:

- map a user-accessible code page and stack;
- enter Ring 3 with `iretq`;
- receive experimental vector `0x80` from Ring 3;
- return to the same user execution;
- subsequently enter the kernel through a Ring 3 breakpoint using the TSS privilege stack.

This is not yet a process runtime. The user pages belong to a validation path, not an isolated process/address-space object.

## Current missing subsystems

Not yet implemented:

- IRQ-driven task preemption;
- complete interrupted-register/trap-frame storage per task;
- generic task stack allocation/free lifecycle;
- sleep/wakeup and wait queues;
- process/thread objects;
- per-process virtual address spaces;
- ELF loading;
- general syscall dispatch and stable ABI;
- persistent userspace `init` and shell;
- process exit/reaping;
- VFS/persistent filesystem;
- networking;
- modern PCIe/ACPI driver framework;
- USB, general graphics or audio;
- SMP/multicore scheduling;
- capability enforcement;
- production security guarantees.

## Architectural principles

### Safe Rust by default

Safe Rust is the default. `unsafe` is restricted to hardware, boot, privileged CPU state, ABI boundaries, raw memory/stack manipulation, and invariants that cannot be expressed safely. Every `unsafe` boundary should state its contract.

### Explicit ownership

CPU context, stack memory, address spaces, resources, and long-lived scheduler state must have identifiable owners. Copies of descriptive IDs are fine; accidental copies of ownership-bearing execution state are not.

### Policy/mechanism separation

```text
scheduler policy -> architecture mechanism -> hardware
```

`kernel::scheduler` selects tasks. `arch` never chooses scheduling policy. `kernel` does not duplicate x86_64 assembly.

### Evidence before claims

A build proves compilation. A host test proves host-visible logic. Runtime/privilege/scheduler claims require QEMU or hardware evidence. Documentation must name limitations explicitly.

### No speculative abstraction

New crates, traits, driver frameworks, and subsystem layers are introduced only when implementation pressure justifies them.

### Owner-controlled trust

The long-term system treats the device owner as final authority. Security should not require a project-controlled master key, mandatory project account, mandatory telemetry, or unavoidable remote control.

## Long-term direction

```text
Applications / Services
        |
        v
Stable Userspace ABI
        |
        v
+--------------------------------+
|         Nastalli Kernel        |
| Scheduler / Processes          |
| Virtual Memory / IPC           |
| Capability + Handle Model      |
| VFS / Device Management        |
+---------------+----------------+
                |
               HAL
                |
       +--------+--------+
       |                 |
     x86_64            ARM64
```

The eventual split between kernel and userspace drivers/services is intentionally not frozen. Stronger isolation should be adopted when IPC, capabilities, scheduling, and driver contracts are mature enough to support it cleanly.

## ABI and hardware support policy

The current ABI is experimental and may change freely before stabilization. A stable candidate is planned for the `v0.9.x` release-candidate period.

QEMU/OVMF x86_64 remains the reference platform. Real hardware will be documented using explicit support tiers; `v1.0.0` means production-grade only for configurations in the published support matrix.

## Architectural change policy

Meaningful architecture changes must update, in the same development cycle:

- implementation and tests;
- this architecture document;
- the affected subsystem document;
- `progress.md` once evidence exists;
- `roadmap.md` when milestone state/scope changes;
- security documentation if trust boundaries change.
