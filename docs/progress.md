# Technical Progress History

This file records what was implemented, why it was implemented, and what evidence exists for each milestone. Planned behavior belongs in [`roadmap.md`](roadmap.md); this document is evidence-oriented and should describe only work that actually happened.

## v0.0.7 — Initial scheduler policy

### Goal

Connect the existing task-state model to real timer ticks and validate a minimal round-robin scheduling policy without claiming full CPU context switching or userspace execution.

### Implemented

- `crates/kernel/src/scheduler.rs` with an initial round-robin policy and a default 5-tick quantum;
- scheduler tests covering rotation of ready tasks when a quantum expires;
- runtime ownership of the persistent `TaskTable` by the scheduler;
- PIT IRQ0 unmasked alongside keyboard IRQ1;
- a real 100 Hz PIT tick counter consumed by the kernel runtime;
- serial validation that the first hardware timer tick reaches the scheduler path;
- QEMU/OVMF smoke validation in CI;
- modern 4 MiB OVMF launch support through pflash with a writable VARS copy under `target/`.

### GDT/IRQ0 defect found during validation

The first runtime smoke test with IRQ0 enabled exposed a General Protection Fault immediately after interrupts were enabled. Instrumentation narrowed the failure to the first timer interrupt and reported:

```text
FAULT: GP error=0x0000000000000010
```

The loaded kernel GDT used selector `0x10` for the TSS while segment registers inherited from the bootloader could still reference `0x10` as a data selector. The fix added an explicit kernel data descriptor and reloads `SS`, `DS`, and `ES` after installing the kernel GDT, before loading the TSS. After that correction, IRQ0 could be delivered and return normally.

During isolation, the timer handler was temporarily reduced to EOI-only. After the GDT defect was fixed, `TICKS.fetch_add(1, Ordering::Relaxed)` was restored before the PIC EOI.

### Verification

The final CI run for the functional fix passed:

- `cargo fmt --all -- --check`;
- `cargo xtask test`;
- applicable `cargo check`;
- Clippy with `-D warnings`;
- `cargo xtask build` for the x86_64 kernel target;
- QEMU + OVMF smoke validation.

Observed runtime output included:

```text
NASTALLI OS v0.0.7
Interrupt init: enable complete.
Scheduler initialized: round-robin, 5 tick quantum.
Scheduler timer active: first PIT tick observed.
```

### Limits

- scheduler policy and task-state rotation exist, but full CPU context switching is not implemented yet;
- tasks do not yet own independent execution stacks;
- multiple task bodies are not yet preemptively executed;
- there are no processes or Ring 3 userspace yet;
- ABI and syscall entry are still future milestones.

---

## Maintenance after v0.0.6

### Runtime and CI foundation

After `v0.0.6`, several maintenance changes prepared the codebase for the scheduler milestone:

- the Cargo workspace was updated to `resolver = "3"` for Edition 2024;
- `kernel::start()` was reduced to high-level orchestration and delegates banner, platform, memory, task, framebuffer, and runtime-loop work to smaller initialization functions;
- the `TaskTable` stopped being temporary: `initialize_tasks()` returns the table and moves ownership into the long-running kernel runtime;
- a regression test was added to confirm that bootstrap-state construction produces exactly one task;
- framebuffer painting remains directly in the boot flow because a second real consumer does not yet justify a separate `framebuffer -> console` abstraction;
- the HAL remains limited to abstractions with concrete consumers, currently serial and keyboard input;
- the Rust toolchain declaration now includes `rust-src` and `llvm-tools-preview`;
- CI also installs the LLVM tools because `bootloader 0.11.10` requires them during its build.

The first public CI runs failed before host tests completed because LLVM tools were unavailable. The failure was confirmed in GitHub Actions logs as:

```text
failed to get llvm tools: NotFound
```

After `llvm-tools-preview` was added to the toolchain/CI environment, the corrected CI run completed successfully.

### Documentation and project presentation

The public repository documentation was professionalized while preserving the distinction between implemented and planned behavior:

- public documentation was converted to English-first;
- the README was rewritten around current capabilities, explicit experimental status, build/run instructions, architecture, and engineering principles;
- architecture documentation separates the current implementation from long-term target layers;
- the roadmap was expanded from short bring-up milestones into an evidence-based maturity path through `v1.0.0` and beyond;
- `v1.0.0` was defined as a production-grade baseline for explicitly supported configurations rather than a claim of universal Windows/Linux-level hardware compatibility;
- future hardware support is described through explicit Tier 1/Tier 2/Tier 3/unsupported categories;
- contribution, security, subsystem, and unsafe-code documentation was rewritten in professional English while preserving current implementation facts;
- a design record and implementation plan were added under `docs/superpowers/` for the documentation overhaul.

---

## v0.0.6 — Task structure

### Goal

Create a minimal contract for task identity and task state without introducing a scheduler, context switching, or userspace yet.

### Implemented

- `crates/kernel/src/task.rs` with `TaskId`, `TaskState`, `Task`, and `TaskTable`;
- fixed table capacity of 16 tasks without dynamic storage policy;
- monotonic task IDs;
- explicit create, lookup, and state-update operations;
- bootstrap task created during initialization and marked `Running` as descriptive state only;
- bootstrap table kept alive by explicit ownership in the kernel runtime rather than a singleton/global;
- tests covering creation, monotonic IDs, state transitions, capacity, missing tasks, and bootstrap-state construction;
- dedicated documentation in [`tasks.md`](tasks.md).

### Limits

- PIT infrastructure exists, but IRQ0 remains masked in the `v0.0.6` runtime;
- there is no scheduler in `v0.0.6`;
- there is no preemption;
- there is no saved CPU context model;
- tasks do not yet own independent stacks;
- there are no processes or userspace execution.

### Verification

Before the post-v0.0.6 maintenance work, local verification recorded:

- `cargo fmt --all -- --check`: passed;
- `cargo xtask test`: task-model tests and applicable crate tests passed;
- applicable `cargo check`: passed;
- `cargo clippy ... -- -D warnings`: passed;
- `cargo xtask build`: x86_64 kernel build succeeded;
- `cargo xtask run` with QEMU + OVMF: boot validated.

Observed output included:

```text
NASTALLI OS v0.0.6
Physical memory: 30269 usable frames.
Task table initialized: 1 task.
```

QEMU was stopped by a controlled timeout after validation because the kernel intentionally remains in an infinite runtime loop.

---

## v0.0.5 — PS/2 keyboard input

### Goal

Capture basic keyboard input under QEMU through IRQ1 while keeping the interrupt handler small and separating hardware access from scancode decoding.

### Implemented

- `arch::keyboard` reads port `0x60` and stores the most recent scancode atomically;
- keyboard handler installed on IRQ1 after PIC remapping;
- IRQ1 unmasked while IRQ0 remains masked until scheduler work;
- `hal::keyboard` decodes Set 1 scancodes for eight basic keys;
- kernel reports decoded keys through serial diagnostics;
- tests for a supported key press and ignored key-release scancode;
- dedicated documentation in [`input.md`](input.md).

### Decisions

- interrupt context does not allocate, format strings, or call high-level policy code;
- single-event storage was sufficient to validate the hardware path at this stage;
- queueing/backpressure is deferred until tasks create a real concurrent consumer;
- USB and complex keyboard layouts are outside this version's scope.

### Verification

- HAL scancode tests: 2 passed;
- applicable `cargo check`: passed;
- Clippy with `-D warnings`: passed;
- `cargo xtask build`: passed;
- `cargo xtask run` with QEMU + OVMF: passed.

Observed output included:

```text
NASTALLI OS v0.0.5
IDT and keyboard IRQ1 initialized.
Keyboard input initialized on IRQ1.
Physical memory: 30296 usable frames.
```

QEMU was stopped by a controlled timeout after validation because the kernel remains in an infinite loop.

---

## v0.0.4 — Initial kernel heap

### Goal

Provide a small, stable first kernel heap for APIs that require allocation without introducing dynamic paging or userspace heap design.

### Implemented

- `crates/kernel/src/heap.rs` with a statically allocated, 8-byte-aligned 64 KiB heap;
- `linked_list_allocator::LockedHeap` as the global allocator;
- explicit heap initialization in kernel startup;
- unit test for the alignment rule;
- dedicated documentation in [`heap.md`](heap.md).

### Decisions

- static storage avoids depending on kernel-controlled paging at this stage;
- `FrameAllocator` remains separate and is not prematurely coupled to heap growth;
- no custom allocation API or speculative ownership abstraction was added.

### Verification

- heap alignment unit test: passed;
- applicable `cargo check` and `cargo clippy`: passed;
- final `cargo xtask build`: x86_64 kernel compiled;
- final `cargo xtask run` under QEMU + OVMF: boot validated.

Observed output included:

```text
NASTALLI OS v0.0.4
Kernel heap initialized: 64 KiB.
Physical memory: 30298 usable frames.
```

QEMU was stopped by a controlled timeout after validation because the kernel remains in an infinite loop.

---

## v0.0.3 — Physical memory

### Goal

Interpret the `BootInfo` memory map and expose aligned 4 KiB physical frames without implementing a heap or taking control of paging yet.

### Implemented

- `crates/kernel/src/memory.rs` with `FrameAllocator` and `PhysicalFrame`;
- allocation restricted to `MemoryRegionKind::Usable`;
- safe handling of inclusive starts and exclusive ends;
- conservative alignment of region boundaries;
- usable-frame counting during initialization;
- tests covering misaligned boundaries and reserved regions;
- dedicated documentation in [`memory.md`](memory.md).

### Verification

- kernel memory unit tests: 2 passed;
- applicable `cargo check`: passed;
- Clippy with `-D warnings`: passed;
- `cargo xtask build`: passed;
- `cargo xtask run` with QEMU + OVMF: boot validated and reported `30340 usable frames`.

Relevant output:

```text
NASTALLI OS v0.0.3
IDT, PIC and PIT initialized at 100 Hz.
Physical memory: 30340 usable frames.
```

QEMU was stopped by a controlled timeout after validation because the kernel remains in an infinite loop.

---

## v0.0.2 — Exceptions and interrupts

### Goal

Install the minimum x86_64 exception and hardware-interrupt infrastructure without introducing scheduling, processes, or userspace.

### Implemented

- `crates/arch/src/gdt.rs`: GDT with kernel code segment and TSS;
- `crates/arch/src/interrupts.rs`: IDT with breakpoint, page-fault, and double-fault handlers;
- PIC 8259 remapped to vectors 32–47;
- PIT programmed for 100 Hz;
- atomic tick counter with no scheduler consumer yet;
- minimal `nastalli_arch::gdt::init()` and `nastalli_arch::interrupts::init()` APIs;
- test for the PIT divisor used for 100 Hz;
- `xtask run` with OVMF discovery and headless display control through `NASTALLI_QEMU_DISPLAY`.

### Decisions

- assembly remains restricted to `arch`;
- generic kernel code does not manipulate ports/registers/instructions directly;
- page fault and double fault stop the CPU after diagnostics at this stage;
- timer ticks are recorded but not consumed by a scheduler.

### Verification

Validation used the pinned `nightly-2025-01-01` toolchain:

- `rustfmt --check`: passed;
- `cargo check` for `arch`, `hal`, `kernel`, and `xtask`: passed;
- `cargo clippy ... -- -D warnings`: passed;
- `cargo xtask test`: the applicable test passed;
- `cargo xtask build`: passed;
- `cargo xtask run` under QEMU + OVMF: boot validated.

Observed serial output included:

```text
NASTALLI OS v0.0.2
Architecture: x86_64
Boot: UEFI
Kernel initialized successfully.
IDT, PIC and PIT initialized at 100 Hz.
```

QEMU was stopped by a controlled timeout after validation because the kernel remains in an infinite loop.

---

## v0.0.1 — Boot

### Goal

Enter a Rust `no_std` kernel through UEFI, emit serial diagnostics, and access the boot framebuffer.

### Implemented

- minimal Cargo workspace with the initially justified crates;
- `bootloader` / `bootloader_api` pinned to `0.11.10`;
- `x86_64-unknown-none` target using `build-std` in the kernel build path;
- UEFI entry point;
- panic handler;
- COM1 serial output;
- initial framebuffer painting;
- `tools/xtask` commands for build, image creation, execution, and tests.

### Result

The UEFI image was created during the initial milestone and the boot chain was subsequently validated under QEMU/OVMF during the `v0.0.2` work.

---

## Evidence policy going forward

As Nastalli grows, the verification bar must grow with it.

Early milestones can be supported by host tests, target builds, QEMU boot, and serial/runtime evidence. Later milestones should additionally require the tests appropriate to their risk: stress testing, fuzzing, fault injection, SMP race validation, real-hardware qualification, reproducible builds, security review, and upgrade testing.

The roadmap is not evidence. This file should record only verification that actually occurred.
