# Architecture

## Current state

Nastalli is an experimental Rust operating-system kernel. The current public runtime is `v0.0.9`; `v0.1.0` is under active development and now includes verified IRQ-driven Ring 0 preemption while building toward the first persistent isolated userspace.

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
- cooperative kernel context construction/switching;
- complete 64-bit interrupt/preemption frame representation;
- raw timer entry, register save/restore, and `iretq` preemption return.

The cooperative context-switch primitive preserves `rsp` plus the SysV callee-saved register set (`rbp`, `rbx`, `r12`-`r15`). It remains useful as a separate mechanism but is no longer the mechanism used by the IRQ-driven preemption proof.

`arch::preemption::InterruptContext` represents the 15 general-purpose registers followed by the five-word 64-bit hardware return frame (`RIP`, `CS`, `RFLAGS`, `RSP`, `SS`). Keeping this layout identical to the stack consumed by the assembly entry/`iretq` path is a correctness invariant.

### `crates/hal`

Provides safe hardware-facing interfaces with real current consumers. The HAL is deliberately small; Nastalli does not add abstractions merely because a future OS might need them.

### `crates/kernel`

Owns architecture-independent policy and long-lived runtime state where practical:

- physical-frame allocation model;
- kernel heap;
- task identity/state/context ownership;
- kernel-stack metadata;
- round-robin scheduler policy;
- conversion of scheduler switch decisions into cooperative or preemptive task-owned contexts;
- startup/runtime orchestration;
- preparation of the experimental Ring 3 path.

`Task` can own cooperative `Context` state, `PreemptionContext` state, and optional `KernelStack` metadata. `Scheduler` owns the task table. The scheduler changes runnable state and selects `from/to`; the architecture layer performs the privileged register/stack transition.

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
    +--> install timer preemption hook
    |
    +--> bootstrap spins
    |       |
    |       +--> PIT IRQ0
    |               |
    |               +--> save 15 GPRs + RIP/CS/RFLAGS/RSP/SS
    |               +--> scheduler selects worker
    |               +--> iretq into worker
    |
    +--> worker spins forever without yielding
    |       |
    |       +--> PIT IRQ0 preempts worker
    |               +--> save worker frame
    |               +--> scheduler selects bootstrap
    |               +--> iretq into bootstrap
    |
    +--> repeat until four IRQ-driven switches are observed
    +--> clear preemption hook
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
          |      +--> saved preemption frame after first IRQ switch
          |
          +--> Task 1 (worker)
                 +--> prepared/saved PreemptionContext
                 +--> KernelStack metadata
```

The early scheduler object itself currently resides in a dedicated physical frame so its address remains stable while execution moves between stacks.

This is a bring-up mechanism, not the final general allocator/lifetime design. Stack freeing, task reaping, dynamic virtual mappings, and resource teardown remain incomplete.

## Interrupt/preemption boundary

IRQ-driven preemption is now implemented and verified for the current Ring 0 task proof.

The architecture path is:

```text
PIT IRQ0
   |
   v
raw interrupt entry
   |
   +--> CPU supplies RIP/CS/RFLAGS/RSP/SS
   +--> assembly saves all 15 GPRs
   |
   v
InterruptContext
   |
   +--> timer accounting / PIC acknowledgement
   +--> kernel scheduler policy
   |
   v
select next runnable task
   |
   +--> save outgoing frame pointer into Task
   +--> select incoming task-owned frame
   |
   v
restore GPRs
   |
   v
iretq
```

The full five-word hardware frame is mandatory in 64-bit mode. An earlier three-word synthetic frame reached the worker with `RSP = 0` and caused a triple fault. QEMU diagnostics and the kernel ELF localized that failure, and the complete-frame contract is now covered by regression tests.

### Ring 3 preemption boundary

The same frame shape can represent a timer interrupt originating in Ring 3, but **frame shape alone is not enough to make userspace preemption safe**.

The TSS currently points privilege transitions at one shared Ring 0 interrupt stack. If multiple Ring 3 tasks were suspended while their saved frames lived on that same stack, a later privilege transition could overwrite an older task's frame. General Ring 3 scheduling therefore requires:

- an owned Ring 0 privilege stack for every scheduler-managed user task;
- a controlled architecture API for selecting the active TSS `RSP0` before returning to that task;
- task/scheduler state that keeps the selected privilege stack and saved user frame alive together;
- tests proving repeated Ring 3 -> IRQ -> scheduler -> Ring 3 transitions cannot overwrite another task's state.

Until those invariants exist, the current preemption proof remains intentionally Ring 0-only.

## Current userspace boundary

Nastalli currently proves that it can:

- map a user-accessible code page and stack;
- enter Ring 3 with `iretq`;
- receive experimental vector `0x80` from Ring 3;
- return to the same user execution;
- subsequently enter the kernel through a Ring 3 breakpoint using the TSS privilege stack.

This is not yet a process runtime. The user pages belong to a validation path, not an isolated process/address-space object, and the probe is not yet owned by the scheduler.

## Current missing subsystems

Not yet implemented:

- per-task Ring 0 privilege stacks and dynamic TSS `RSP0` selection;
- scheduler-managed Ring 3 task preemption;
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
