# Roadmap

Nastalli's roadmap is organized by **engineering maturity**, not fixed dates. A milestone is complete only when the relevant behavior is implemented, verified, documented, and supported by green CI.

The roadmap is directional. Implementation evidence may change scope, but planned work must never be presented as already implemented.

## Release maturity model

| Version range | Meaning |
|---|---|
| `0.0.x` | Kernel bring-up and foundational experiments |
| `0.1.x`–`0.8.x` | Major subsystem development and integration |
| `0.9.x` | Stabilization, ABI freeze candidates, hardware qualification, reliability work |
| `1.0.0` | Production-grade baseline for explicitly supported configurations |
| `1.x` | Compatibility, performance, driver coverage, and supported-platform expansion |
| `2.x+` | Broader ecosystem and major architectural evolution |

A version number does not imply support outside the documented hardware/software support matrix.

## Completed foundation — `v0.0.x`

- [x] `v0.0.1` — UEFI boot, Rust `no_std` kernel entry, serial diagnostics, framebuffer access.
- [x] `v0.0.2` — GDT/TSS, IDT, exceptions, PIC 8259, PIT setup and tick foundation.
- [x] `v0.0.3` — physical memory map handling and 4 KiB frame allocator model.
- [x] `v0.0.4` — initial static 64 KiB kernel heap.
- [x] `v0.0.5` — basic PS/2 keyboard input through IRQ1.
- [x] `v0.0.6` — task identity/state model with persistent runtime ownership.
- [x] `v0.0.7` — initial round-robin scheduler policy driven by real 100 Hz PIT ticks with a 5-tick quantum.
- [x] `v0.0.8` — controlled Ring 3 transition with dedicated user code/stack mappings and verified privilege return through the TSS stack.
- [x] `v0.0.9` — independent `no_std` ABI crate and experimental Ring 3 syscall entry/return through vector `0x80`.

The `v0.0.x` series establishes kernel bring-up and the first userspace boundary. It does not promise a stable ABI, process isolation, broad hardware support, or production security.

---

## `v0.1.0` — First isolated userspace

### Verified during development

- [x] architecture-level cooperative save/restore of `rsp` and x86_64 callee-saved registers;
- [x] fresh kernel contexts prepared on independent physical-frame-backed stacks;
- [x] repeated bootstrap/worker context-switch round trips under QEMU;
- [x] task objects own saved `Context` and kernel-stack metadata;
- [x] scheduler `Switch { from, to }` decisions resolve to task-owned contexts;
- [x] real PIT tick accounting drives repeated scheduler-selected execution switches from normal kernel context;
- [x] Ring 3 syscall entry/return remains functional after scheduler-managed task switching.

These are verified foundations, not completion of `v0.1.0`.

### Remaining target capabilities

- [ ] architecture trap/interrupt context capable of representing the full interrupted task state;
- [ ] PIT IRQ-driven preemption that switches live tasks through interrupt return rather than cooperative polling;
- [ ] robust task stack allocation and teardown lifecycle;
- [ ] repeated preemptive execution of multiple task bodies without corruption;
- [ ] first persistent userspace `init`;
- [ ] minimal userspace shell;
- [ ] syscall dispatch required by `init`/shell;
- [ ] basic task/process termination and resource reaping path.

### Exit criteria

`v0.1.0` is complete only when:

- multiple tasks execute repeatedly without corrupting kernel state;
- timer interrupts can cause repeatable execution-context changes under QEMU;
- task-owned kernel stack/context lifetime is explicit and teardown behavior is defined for the implemented path;
- Ring 3 code cannot directly perform privileged kernel operations;
- syscall entry/return survives repeated calls from the first userspace program;
- kernel and userspace stacks remain clearly separated;
- first userspace `init` and minimal shell execute through documented kernel interfaces;
- basic termination/reaping works for the supported initial process path;
- tests document scheduler, trap-frame, context, stack, and privilege invariants;
- README, architecture, task, ABI, progress and roadmap documentation match implementation;
- full CI, target build and QEMU smoke/stress coverage for the milestone are green;
- a dedicated **v0.1.0 review pass** finds and fixes release-blocking architecture, dependency, `unsafe`, memory, scheduler, ABI, security and documentation defects before the version is declared complete.

### Mandatory v0.1.0 review gate

Before promoting to the next milestone, the entire repository must be reviewed for:

- crate/module boundaries and dependency direction;
- unnecessary or poorly chosen dependencies;
- all `unsafe` blocks and assembly contracts;
- GDT/TSS/IDT/PIC/PIT correctness;
- physical memory, paging and heap invariants;
- task/context/stack ownership and lifecycle;
- scheduler correctness, races, interrupt safety and preemption behavior;
- Ring 3 isolation and syscall/ABI validation;
- panic paths, deadlocks, undefined behavior and corruption risks;
- tests, QEMU runtime evidence and CI coverage;
- dead code, warnings, naming and API quality;
- documentation claims versus actual implementation.

Findings that affect correctness, security, isolation or maintainability must be fixed before `v0.1.0` is considered complete.

---

## `v0.2.0` — Process and virtual-memory model

Target capabilities:

- explicit process/thread model;
- per-process address spaces;
- virtual memory manager and map/unmap APIs;
- page permissions and user/kernel separation;
- process/thread lifecycle and exit/reaping semantics;
- ELF executable loader;
- user/kernel stack allocation lifecycle;
- IPC foundation when required by real consumers.

Exit criteria include tested process isolation, malformed executable handling, safe address-space teardown, and process-local fault handling.

---

## `v0.3.0` — Storage and filesystem foundation

Target capabilities:

- VFS;
- handle/file-descriptor model;
- in-memory bootstrap filesystem;
- device namespace;
- block-device abstraction;
- at least one persistent filesystem;
- buffered I/O/cache foundation;
- explicit filesystem error and recovery paths.

A supported persistent filesystem must pass create/read/write/reopen/delete plus malformed/corruption/error scenarios before the milestone is complete.

---

## `v0.4.0` — Driver and platform framework

Target capabilities:

- driver registration/lifecycle;
- PCI/PCIe enumeration;
- ACPI platform discovery;
- VirtIO support for the reference platform where useful;
- NVMe and/or AHCI storage;
- HID foundation;
- interrupt-routing improvements;
- DMA ownership rules.

Supported devices/classes must be explicitly documented rather than implied globally.

---

## `v0.5.0` — Networking

Target capabilities:

- network-device interface;
- Ethernet;
- ARP / IPv6 neighbor-discovery foundation;
- IPv4 and IPv6 foundation;
- ICMP, UDP, TCP;
- DNS resolver;
- socket ABI;
- timeout/error handling.

Userspace must perform basic TCP/UDP communication on a supported device, and packet-facing code must receive fuzz/stress coverage appropriate to its risk.

---

## `v0.6.0` — SMP and modern machine support

Target capabilities:

- multiprocessor startup;
- local APIC / IOAPIC;
- multicore scheduler;
- per-CPU state;
- SMP synchronization;
- multicore interrupt routing;
- higher-resolution timer foundation;
- CPU topology and power-management foundations.

The milestone requires multicore scheduler stress testing, documented synchronization/lock rules, and working single-core fallback.

---

## `v0.7.0` — Security and owner-controlled trust

Target capabilities:

- capability-oriented or rights-scoped resource access;
- stronger process isolation;
- secure random source for supported platforms;
- explicit privilege boundaries;
- owner-controlled trust and optional owner-controlled Secure Boot strategy;
- hardened validation of userspace-controlled inputs;
- security-audit preparation.

No project-controlled master key or mandatory remote authority may be required for supported local operation.

---

## `v0.8.0` — General device and desktop foundations

Target capabilities:

- USB host foundation and HID;
- richer keyboard/mouse/input model;
- hotplug;
- graphics-device/display foundation beyond boot framebuffer painting;
- audio foundation;
- power/suspend work where practical;
- userspace device interfaces where justified.

This milestone does not imply a complete desktop environment.

---

## `v0.9.0` — Stabilization program

Required work expands from feature implementation to sustained evidence:

- fuzzing of parsers and syscall-facing inputs;
- scheduler and SMP stress tests;
- fault injection;
- storage/filesystem recovery testing;
- network robustness testing;
- lifetime/leak analysis;
- crash diagnostics;
- performance profiling;
- boot/reboot/shutdown reliability;
- reproducible builds;
- security review;
- supported-hardware qualification;
- upgrade/migration testing;
- explicit known-issues tracking.

Known data-corruption or privilege-isolation defects block promotion.

---

## `v0.9.x` — Release candidates

Goals:

- freeze a candidate userspace ABI;
- define compatibility/deprecation rules;
- remove known release-blocking corruption, isolation and boot-reliability bugs;
- finalize hardware support tiers;
- validate upgrades, release artifacts and reproducibility;
- complete pre-1.0 security review work.

Stability guarantees begin only when explicitly documented.

---

## `v1.0.0` — Production-grade baseline

`v1.0.0` means **stable for documented supported configurations**. It does not mean universal PC compatibility or Windows/Linux-level hardware coverage.

Minimum expectations:

- stable documented userspace ABI for supported use cases;
- reliable process/thread isolation and virtual memory;
- production-quality scheduler on supported CPU configurations;
- persistent storage/filesystem support;
- networking suitable for supported normal applications;
- mature error handling and crash diagnostics;
- owner-controlled security/trust model;
- reproducible releases and regression suite;
- documented hardware support matrix;
- documented upgrade, compatibility and security-reporting policies;
- no known release-blocking data-corruption or privilege-isolation defects inside the supported matrix.

A release is not promoted merely because roadmap features exist; stabilization evidence must support the claim.

---

## `v1.x` and `v2.x+`

`v1.x` expands driver/filesystem coverage, performance, power management, security hardening, real-hardware qualification and potentially ARM64 while preserving documented stable interfaces.

`v2.x+` may expand toward a desktop-class compositor/application ecosystem, package/application distribution, decentralized owner-controlled services, additional architectures and compatibility layers where technically justified.

These are directions, not promises or fixed schedules.

---

# Hardware support tiers

## Tier 1 — Reference / release-blocking

Continuously or routinely validated; regressions can block releases. x86_64 QEMU/OVMF is the current reference environment.

## Tier 2 — Officially supported

Expected to work for documented features and receives compatibility attention, but may receive less continuous testing than Tier 1.

## Tier 3 — Experimental / community-supported

Useful development targets with no release-blocking compatibility guarantee.

## Unsupported

No compatibility/reliability guarantee. Unsupported means insufficient evidence, not intentional breakage.

---

# General milestone advancement criteria

Every applicable milestone requires:

- correct-target build;
- formatting and Clippy under project warning policy;
- automated tests;
- QEMU/runtime evidence where behavior requires it;
- documentation synchronized with implementation;
- known limitations recorded;
- green CI on the milestone/release commit.

As maturity grows, the gate expands to fuzzing, fault injection, sustained stress testing, SMP race testing, supported-hardware testing, security review, reproducible builds, upgrades/migrations and performance regression checks.

## No "100% stable" claim

Nastalli will not use subjective claims such as "100% stable" as an engineering criterion. Stability must be scoped to documented supported configurations and backed by repeatable evidence.
