# Ring 3 Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove a controlled x86_64 CPL0 -> CPL3 -> CPL0 transition under QEMU without introducing the syscall ABI or a full process model.

**Architecture:** Extend the existing GDT/TSS with user code/data selectors and a valid Ring 0 interrupt stack. Enable the bootloader physical-memory mapping so the kernel can build two temporary user mappings: one executable code page and one writable NX stack page. Enter CPL3 with `iretq`; the user probe executes `int3`, whose DPL3 breakpoint gate returns control to a Ring 0 handler that emits the validation marker and stops.

**Tech Stack:** Rust `no_std`, `x86_64 = 0.14.13`, bootloader/bootloader_api `0.11.10`, QEMU/OVMF, GitHub Actions.

**Spec:** `docs/roadmap.md` milestone `v0.0.8`.

## Global Constraints

- Keep architecture-specific privileged code under `crates/arch`.
- Do not add syscall ABI, process lifecycle, ELF loading, or general-purpose virtual memory yet.
- User code and user stack must be backed by distinct 4 KiB frames.
- User mappings must carry `USER_ACCESSIBLE`; the stack must be non-executable.
- The transition must be runtime-verified under QEMU and CI.

---

### Task 1: Define Ring 3 descriptor contract

**Files:**
- Modify: `crates/arch/src/gdt.rs`
- Test: `crates/arch/src/gdt.rs`

**Interfaces:**
- Produces: `pub fn user_selectors() -> (SegmentSelector, SegmentSelector)` and `pub fn set_kernel_stack(stack_top: VirtAddr)`.

- [ ] Add a failing unit test asserting both returned user selectors have RPL3.
- [ ] Run `cargo xtask test` and confirm the test fails because the Ring 3 selector API does not exist.
- [ ] Add user code/data descriptors and the selector API.
- [ ] Add TSS `privilege_stack_table[0]` update API.
- [ ] Re-run host tests.

### Task 2: Add minimal user-page mapping boundary

**Files:**
- Create: `crates/arch/src/paging.rs`
- Modify: `crates/arch/src/lib.rs`
- Modify: `crates/boot/src/main.rs`

**Interfaces:**
- Produces: architecture helper to map one 4 KiB physical frame at a virtual address with caller-provided flags and a callback-backed frame allocator for intermediate page tables.
- Consumes: bootloader-provided physical-memory offset.

- [ ] Enable `BootloaderConfig.mappings.physical_memory = Some(Mapping::Dynamic)`.
- [ ] Implement active `OffsetPageTable` construction from CR3 and the physical-memory offset.
- [ ] Map user code with `PRESENT | USER_ACCESSIBLE` and user stack with `PRESENT | WRITABLE | USER_ACCESSIBLE | NO_EXECUTE`.
- [ ] Flush each mapping after creation.

### Task 3: Enter and prove Ring 3

**Files:**
- Create: `crates/arch/src/user.rs`
- Modify: `crates/arch/src/lib.rs`
- Modify: `crates/arch/src/interrupts.rs`
- Modify: `crates/kernel/src/lib.rs`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Produces: `pub unsafe fn enter(entry: VirtAddr, stack_top: VirtAddr) -> !`.

- [ ] Change the QEMU smoke test first to require `Ring 3 probe reached kernel breakpoint.` and confirm CI/runtime is red.
- [ ] Allow the breakpoint IDT gate to be invoked from Ring 3.
- [ ] Write a tiny user probe (`int3`) into a fresh physical frame through the physical-memory mapping.
- [ ] Map code and stack into low canonical user virtual addresses.
- [ ] Set TSS RSP0 to the bootloader kernel-stack top.
- [ ] Execute `iretq` into the user probe.
- [ ] In the breakpoint handler, detect RPL3 origin, print `Ring 3 probe reached kernel breakpoint.`, then spin in Ring 0.
- [ ] Run format, tests, check, Clippy, target build, and QEMU smoke.

### Task 4: Close v0.0.8 documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/progress.md`

- [ ] Mark `v0.0.8` complete only after the final runtime marker is observed in green CI.
- [ ] Document that this is a transition probe only: no syscall ABI, user process model, loader, or stable userspace contract yet.
- [ ] Run final CI on the documentation commit and require the QEMU smoke to remain green.
