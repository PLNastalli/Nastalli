# Technical Progress History

This file records what was implemented, why it was implemented, and what evidence exists for each milestone. Planned behavior belongs in [`roadmap.md`](roadmap.md); this document is evidence-oriented and should describe only work that actually happened.

## v0.1.0 development — Scheduler-managed task execution

### Goal

Replace the standalone context-switch validation state with execution state owned by real kernel tasks and prove that scheduler decisions can move CPU execution between those task-owned contexts using real PIT tick accounting.

### Implemented

- `Task` now owns a saved `arch::context::Context` and optional `KernelStack` metadata;
- `Task` is no longer `Copy`, preventing accidental duplication of ownership-bearing execution state;
- `TaskTable::get` now returns a reference to the stored task;
- `TaskTable::install_execution` attaches a prepared context and stack to a task;
- `Scheduler::prepare_context_switch` resolves `Switch { from, to }` into the task-owned outgoing and incoming contexts;
- a stable scheduler runtime is placed in a dedicated physical frame during the early bring-up path;
- a scheduler-managed worker task executes on its own 4 KiB physical-frame-backed kernel stack;
- real 100 Hz PIT tick accounting drives two complete bootstrap/worker scheduling round trips;
- the existing Ring 3 syscall and breakpoint validation still executes afterward.

The low-level architecture switch remains outside scheduler policy. `kernel::scheduler` chooses tasks; `arch::context` performs stack/register manipulation.

### TDD and CI evidence

The task/context integration was developed with explicit RED/GREEN checks before the development workflow was moved back to the repository's requested main-only model.

Key evidence:

- CI #115: RED at host tests because task execution-ownership APIs did not yet exist;
- CI #118: task-owned context/stack model passed formatting, host tests, check, Clippy, target build and QEMU;
- CI #119: RED at host tests because scheduler context-switch preparation did not yet exist;
- CI #122: scheduler context resolution passed the complete pipeline;
- CI #123: static/build stages passed while QEMU failed only because the new `Task switch:` runtime markers were intentionally required before the runtime integration existed;
- main CI #127: formatting, host tests, `cargo check`, Clippy, target build, QEMU/OVMF smoke validation and all task/Ring 3 runtime markers passed.

The runtime evidence required in one boot is:

```text
Task switch: worker entered.
Task switch: bootstrap resumed.
Task switch: worker resumed.
Task switch: bootstrap resumed twice.
Ring 3 syscall entered kernel and returned.
Ring 3 probe reached kernel breakpoint.
```

This proves that scheduler-selected task contexts, rather than only a standalone context-switch probe, now drive real CPU execution across independent kernel stacks.

### Important limit: not IRQ-driven preemption yet

The current switch path consumes real PIT ticks from normal kernel execution. IRQ0 records ticks and acknowledges the interrupt, but it does not directly switch tasks.

The existing cooperative `arch::context::switch()` is intentionally **not** called inside the PIT handler. Doing so would save the interrupt-handler context rather than the complete interrupted task state.

The next preemption slice therefore requires an explicit interrupt/trap-frame representation and an architecture entry/return path capable of saving the interrupted task and restoring the scheduler-selected task through interrupt return.

### Remaining v0.1.0 work

- IRQ-driven preemption with full interrupted task state;
- generic stack allocation/free and task teardown lifecycle;
- repeated preemptive task execution stress coverage;
- first persistent userspace `init`;
- minimal userspace shell;
- syscall dispatch required by the first userspace programs;
- basic process/task termination and reaping;
- final repository-wide v0.1.0 architecture, dependency, unsafe, correctness, security and documentation review.

---

## v0.1.0 development — Cooperative context-switch foundation

### Goal

Prove that Nastalli can save a live kernel execution context, run a second kernel context on an independent stack, resume the bootstrap context, resume the suspended worker, and return again before integrating context ownership into scheduler-managed tasks.

### Implemented

The architecture layer introduced `Context`, fresh-context stack preparation, a trampoline, and x86_64 assembly save/restore for `rbp`, `rbx`, `r12`, `r13`, `r14`, `r15`, and `rsp`.

The first runtime proof allocated an independent worker stack and performed:

```text
bootstrap -> worker -> bootstrap -> resumed worker -> bootstrap
```

The same boot then continued through the existing Ring 3 syscall path.

### Verification

CI #108 passed formatting, host tests, checks, Clippy, target build, and QEMU/OVMF smoke validation with the original cooperative context-switch markers and the later Ring 3 markers.

This slice was intentionally a mechanism proof. The later scheduler-managed task integration superseded its probe-specific ownership model while keeping the same low-level architecture primitive.

---

## v0.0.9 — Experimental ABI and syscall entry/return

### Implemented

- independent `crates/abi` `no_std` crate;
- experimental syscall vector `0x80`;
- Ring 3-callable IDT gate;
- kernel entry that returns through the interrupt frame;
- Ring 3 probe equivalent to `int 0x80; int3`.

### Verification

CI #98 proved:

```text
Ring 3 syscall entered kernel and returned.
Ring 3 probe reached kernel breakpoint.
```

CI #100 passed the finalized `v0.0.9` runtime state.

### Limits

The ABI remains experimental. There is no general dispatcher, stable syscall namespace, argument/error/handle contract, or process lifecycle yet.

---

## v0.0.8 — Controlled Ring 3 transition foundation

### Implemented

- Ring 3 code/data descriptors;
- TSS Ring 0 privilege stack;
- user-accessible code/stack mappings;
- `iretq` transition helper;
- Ring 3-callable breakpoint gate.

Validation exposed privilege-stack/TSS/IST mistakes that could triple-fault QEMU. Writable/aligned TSS/IST stack storage was established before the milestone was considered complete.

### Verification

QEMU smoke validation required:

```text
Ring 3 probe reached kernel breakpoint.
```

This proved user execution reached Ring 3 and could re-enter the kernel through the configured privilege boundary.

---

## v0.0.7 — Initial scheduler policy

### Implemented

- round-robin scheduler with default 5-tick quantum;
- real 100 Hz PIT tick counter;
- task-state rotation tests;
- runtime scheduler ownership of `TaskTable`;
- IRQ0 enabled alongside IRQ1;
- QEMU/OVMF smoke validation;
- modern 4 MiB OVMF pflash launch support.

Enabling IRQ0 exposed `#GP error=0x10` because the newly loaded GDT reused selector `0x10` for TSS while inherited segment registers could still reference it as a data selector. The fix added an explicit kernel data descriptor and reloaded `SS`, `DS`, and `ES` before loading the TSS.

### Verification

The milestone passed formatting, tests, checks, Clippy, target build and QEMU. Runtime evidence included the scheduler initialization and first real PIT tick marker.

---

## v0.0.6 — Task structure

Introduced `TaskId`, `TaskState`, `Task`, fixed-capacity `TaskTable`, monotonic IDs, explicit state transitions, bootstrap task ownership and regression coverage. At this milestone there was no scheduler, context switching, independent task stack, process model or userspace execution.

---

## v0.0.5 — PS/2 keyboard input

Implemented IRQ1 scancode capture, minimal Set 1 decoding through HAL, normal-context serial reporting, and regression tests while keeping the interrupt path allocation-free and policy-light.

---

## v0.0.4 — Initial kernel heap

Implemented a statically allocated 64 KiB kernel heap backed by `linked_list_allocator::LockedHeap`, with explicit initialization and alignment coverage.

---

## v0.0.3 — Physical memory

Implemented boot-memory-map interpretation, aligned 4 KiB usable `PhysicalFrame` allocation and usable-frame counting while deliberately deferring general virtual-memory ownership.

---

## v0.0.2 — Exceptions and interrupts

Implemented GDT/TSS, IDT, exception handlers, PIC remapping, PIT setup, atomic tick foundation, OVMF discovery and headless QEMU support.

---

## v0.0.1 — Boot

Established the Cargo workspace, `bootloader`/`bootloader_api` 0.11.10 integration, `x86_64-unknown-none` build, UEFI entry, panic handling, COM1 serial output, framebuffer access and `xtask` build/image/run/test commands.

---

## Maintenance and project-quality work

Notable maintenance work includes:

- Cargo resolver 3 for Edition 2024;
- smaller `kernel::start()` orchestration stages;
- persistent runtime task ownership;
- `rust-src` and `llvm-tools-preview` pinned in the toolchain;
- CI coverage for formatting, host tests, checks, Clippy, target build and QEMU runtime smoke;
- OVMF pflash support using a writable VARS copy under `target/`;
- public documentation rewritten in English around explicit implementation evidence and limitations.

The project does not use subjective claims such as "100% stable". Later stability claims must be scoped to documented supported configurations and backed by repeatable evidence.

## Evidence policy going forward

As Nastalli grows, the verification bar must grow with it. Early milestones can use host tests, target builds, QEMU boot and serial evidence. Later milestones must add the tests appropriate to their risk, including stress testing, fuzzing, fault injection, SMP race validation, real-hardware qualification, reproducible builds, security review and upgrade testing.

The roadmap is not evidence. This file records only behavior that has actually been implemented and verified.
