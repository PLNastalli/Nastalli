# Technical Progress History

This file records what was implemented, why it was implemented, and what evidence exists for each milestone. Planned behavior belongs in [`roadmap.md`](roadmap.md); this document is evidence-oriented and should describe only work that actually happened.

## v0.1.0 development — Cooperative context-switch foundation

### Goal

Prove that Nastalli can save a live kernel execution context, run a second kernel context on an independent stack, resume the bootstrap context, resume the suspended worker, and return again before integrating context ownership into scheduler-managed tasks.

### TDD evidence

The context-switch slice was developed with two explicit RED/GREEN cycles on draft PR #1.

The first RED required a fresh-context preparation API from a host test. After formatting was corrected so the test could run, CI failed for the intended reason:

```text
cannot find function `prepare` in module `super`
```

The minimal architecture implementation then added:

- `arch::context::Context` with a saved stack pointer;
- fresh-context stack-frame preparation;
- an x86_64 entry trampoline;
- assembly save/restore for `rbp`, `rbx`, `r12`, `r13`, `r14`, `r15`, and `rsp`;
- host coverage for prepared-stack placement/alignment.

CI #105 passed formatting, host tests, checks, Clippy, target build, and the pre-existing QEMU smoke test with that primitive present.

The second RED added QEMU smoke requirements for four context-switch markers before the existing Ring 3 markers. CI then passed all static/build checks and failed only because the runtime had not yet executed the new switch path. This distinguished a real runtime requirement from scheduler-state-only behavior.

### Runtime implementation

The kernel validation path now:

- keeps one monotonic `FrameAllocator` instance for the worker stack and later Ring 3/page-table allocations;
- allocates one usable physical frame for the context worker;
- zeros and accesses that frame through the direct physical-memory mapping;
- stores stable bootstrap/worker saved contexts at the beginning of the frame;
- uses the remainder of the frame as an independent worker kernel stack;
- performs bootstrap -> worker -> bootstrap -> resumed worker -> bootstrap;
- continues into the existing Ring 3 syscall probe afterward.

The dedicated worker is a validation probe. It is not yet a scheduler-managed `Task`.

### Verification

CI #108 completed successfully with:

- `cargo fmt --all -- --check`;
- `cargo xtask test`;
- applicable `cargo check`;
- Clippy with `-D warnings`;
- `cargo xtask build` for `x86_64-unknown-none`;
- QEMU + OVMF smoke validation.

The smoke test required the following execution evidence in one boot:

```text
Context switch: worker entered.
Context switch: bootstrap resumed.
Context switch: worker resumed.
Context switch: bootstrap resumed twice.
Ring 3 syscall entered kernel and returned.
Ring 3 probe reached kernel breakpoint.
```

This verifies real continuation across two kernel stacks and confirms that the earlier userspace privilege-transition path still works after the context-switch probe.

### Limits

This slice does **not** complete `v0.1.0`:

- `Task` does not yet own a saved `Context` or kernel stack;
- scheduler `Switch { from, to }` decisions do not yet call the architecture switch routine;
- PIT IRQ0 does not yet preemptively switch live task contexts;
- there is no generic task-stack allocation/teardown lifecycle;
- process/thread objects, ELF loading, `init`, shell, and process exit remain future work.

---

## v0.0.9 — Experimental ABI and syscall entry/return

### Goal

Create the first independent userspace/kernel ABI contract and prove a controlled syscall entry from Ring 3 that returns to user execution.

### Implemented

- `crates/abi` as a small `no_std` workspace crate;
- experimental syscall vector `0x80` shared across the userspace/kernel boundary;
- Ring 3-callable IDT gate at that vector;
- kernel handler that returns through the interrupt frame;
- Ring 3 probe bytes equivalent to `int 0x80; int3`;
- CI smoke assertions that require the syscall-return marker before the later breakpoint marker.

### Verification

The functional CI path proved:

```text
Ring 3 syscall entered kernel and returned.
Ring 3 probe reached kernel breakpoint.
```

CI #98 passed the full static/build/QEMU pipeline for the syscall path, and CI #100 passed the finalized `v0.0.9` runtime banner/state.

### Limits

- the ABI is experimental and not stable;
- there is no general syscall dispatcher or syscall-number namespace yet;
- argument, return-value, error, handle, and compatibility contracts are not frozen;
- there is no process lifecycle or first userspace program yet.

---

## v0.0.8 — Controlled Ring 3 transition foundation

### Goal

Prove a controlled x86_64 privilege transition from Ring 0 to Ring 3 and a safe exception transition back to the kernel privilege stack.

### Implemented

- Ring 3 code/data descriptors in the GDT;
- TSS Ring 0 privilege-stack support;
- user-accessible code and stack mappings;
- architecture `iretq` transition helper;
- Ring 3-callable breakpoint gate for validation;
- runtime probe that enters user code and reaches the kernel breakpoint handler.

Validation work also exposed privilege-stack/TSS/IST mistakes that could reset QEMU before useful diagnostics. Those stack invariants were corrected before the milestone was considered complete.

### Verification

CI smoke validation required:

```text
Ring 3 probe reached kernel breakpoint.
```

The passing smoke established that user execution reached Ring 3 and that the CPU subsequently entered the kernel handler through the configured privilege boundary.

### Limits

- no general userspace runtime existed yet;
- no syscall contract was part of this milestone;
- the mapped code/stack were a validation probe rather than a process address space;
- process isolation and lifecycle were not implemented.

---

## v0.0.7 — Initial scheduler policy

### Goal

Connect the existing task-state model to real timer ticks and validate a minimal round-robin scheduling policy without claiming full CPU context switching.

### Implemented

- `crates/kernel/src/scheduler.rs` with an initial round-robin policy and default 5-tick quantum;
- scheduler tests covering rotation of ready tasks when a quantum expires;
- runtime ownership of the persistent `TaskTable` by the scheduler;
- PIT IRQ0 unmasked alongside keyboard IRQ1;
- real 100 Hz PIT tick counter consumed by the kernel runtime;
- serial validation that the first hardware timer tick reaches the scheduler path;
- QEMU/OVMF smoke validation in CI;
- modern 4 MiB OVMF pflash launch support with a writable VARS copy under `target/`.

### GDT/IRQ0 defect found during validation

Enabling IRQ0 exposed a General Protection Fault immediately after interrupts were enabled:

```text
FAULT: GP error=0x0000000000000010
```

The loaded kernel GDT used selector `0x10` for the TSS while inherited segment registers could still refer to `0x10` as a data selector. The fix added an explicit kernel data descriptor and reloaded `SS`, `DS`, and `ES` after installing the GDT and before loading the TSS. After that correction, timer interrupts could be delivered and return normally.

### Verification

The milestone passed formatting, host tests, applicable checks, Clippy with `-D warnings`, target kernel build, and QEMU/OVMF smoke validation.

Observed output included:

```text
NASTALLI OS v0.0.7
Interrupt init: enable complete.
Scheduler initialized: round-robin, 5 tick quantum.
Scheduler timer active: first PIT tick observed.
```

### Limits at that milestone

- scheduling policy rotated task states but did not switch CPU execution contexts;
- tasks did not own independent execution stacks;
- multiple task bodies were not preemptively executed.

---

## Maintenance after v0.0.6

### Runtime and CI foundation

Several maintenance changes prepared the codebase for later scheduler/userspace milestones:

- Cargo workspace updated to `resolver = "3"` for Edition 2024;
- `kernel::start()` reduced to high-level orchestration with smaller initialization functions;
- `TaskTable` ownership moved into the long-running runtime rather than being discarded;
- bootstrap task-table regression coverage added;
- framebuffer abstraction intentionally deferred until a second real consumer exists;
- HAL kept limited to concrete serial/keyboard consumers;
- `rust-src` and `llvm-tools-preview` included in the pinned toolchain;
- CI installs the LLVM tools required by `bootloader 0.11.10`.

The first public CI attempts exposed a missing LLVM-tools dependency:

```text
failed to get llvm tools: NotFound
```

After `llvm-tools-preview` was added, the corrected pipeline completed successfully.

### Documentation and project presentation

The public documentation was rewritten around implemented behavior, explicit experimental status, current architecture, build/run instructions, engineering principles, support tiers, security reporting, and evidence-based roadmap gates. `v1.0.0` is defined as a production-grade baseline only for explicitly documented supported configurations rather than a claim of universal hardware compatibility.

---

## v0.0.6 — Task structure

### Goal

Create a minimal contract for task identity and task state without introducing a scheduler or execution contexts yet.

### Implemented

- `TaskId`, `TaskState`, `Task`, and fixed-capacity `TaskTable`;
- capacity of 16 tasks;
- monotonic task IDs;
- explicit create, lookup, and state-update operations;
- bootstrap task marked `Running`;
- explicit runtime ownership rather than a global singleton;
- tests for identity, state, capacity, lookup/update, and bootstrap construction.

### Verification

Formatting, host tests, applicable checks, Clippy, target build, and QEMU/OVMF boot validation passed for the milestone. Runtime output included the bootstrap task-table count.

### Limits at that milestone

There was no scheduler, context switching, independent task stack, process model, or userspace execution.

---

## v0.0.5 — PS/2 keyboard input

### Goal

Capture basic keyboard input under QEMU through IRQ1 while keeping the interrupt handler small and separating hardware access from scancode decoding.

### Implemented

- `arch::keyboard` reads port `0x60` and stores the latest scancode atomically;
- keyboard handler installed on IRQ1 after PIC remapping;
- HAL Set 1 decoding for the initial supported keys;
- serial reporting from normal kernel context;
- regression tests for supported press and ignored release behavior.

The interrupt path avoids allocation, formatted output, and high-level policy work.

### Verification

HAL tests, applicable checks, Clippy, target build, and QEMU/OVMF runtime validation passed.

---

## v0.0.4 — Initial kernel heap

### Goal

Provide a small first kernel heap without depending on dynamic paging.

### Implemented

- statically allocated 64 KiB heap;
- `linked_list_allocator::LockedHeap` global allocator;
- explicit initialization and alignment regression coverage.

The static design deliberately kept physical-frame allocation separate from heap-growth policy.

### Verification

Heap tests, applicable checks, Clippy, target build, and QEMU/OVMF validation passed.

---

## v0.0.3 — Physical memory

### Goal

Interpret the boot memory map and expose aligned 4 KiB usable physical frames without taking control of paging yet.

### Implemented

- `FrameAllocator` and `PhysicalFrame`;
- allocation restricted to `MemoryRegionKind::Usable`;
- conservative alignment of region boundaries;
- usable-frame counting;
- tests for misaligned and reserved memory regions.

### Verification

Memory tests, applicable checks, Clippy, target build, and QEMU/OVMF validation passed.

---

## v0.0.2 — Exceptions and interrupts

### Goal

Install the minimum x86_64 exception and hardware-interrupt foundation.

### Implemented

- GDT/TSS;
- IDT with initial exception handlers;
- PIC 8259 remapping;
- PIT programmed for 100 Hz;
- atomic tick foundation;
- OVMF discovery and headless QEMU support in `xtask`;
- PIT divisor regression coverage.

### Verification

Pinned-toolchain formatting, applicable checks, Clippy, host tests, target build, and QEMU/OVMF boot validation passed.

---

## v0.0.1 — Boot

### Goal

Enter a Rust `no_std` kernel through UEFI, emit serial diagnostics, and access the boot framebuffer.

### Implemented

- initial Cargo workspace and justified crate boundaries;
- `bootloader` / `bootloader_api` pinned to `0.11.10`;
- `x86_64-unknown-none` build path;
- UEFI entry point;
- panic handler;
- COM1 serial output;
- initial framebuffer painting;
- `xtask` build/image/run/test commands.

The initial UEFI image path was subsequently validated under QEMU/OVMF as the interrupt work matured.

---

## Evidence policy going forward

As Nastalli grows, the verification bar must grow with it.

Early milestones can be supported by host tests, target builds, QEMU boot, and serial/runtime evidence. Later milestones should additionally require the tests appropriate to their risk: stress testing, fuzzing, fault injection, SMP race validation, real-hardware qualification, reproducible builds, security review, and upgrade testing.

The roadmap is not evidence. This file should record only verification that actually occurred.
