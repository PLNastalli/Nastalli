# Roadmap

Nastalli's roadmap is organized by **engineering maturity**, not fixed dates. A milestone is complete only when the relevant behavior is implemented, verified, documented, and supported by green CI.

The roadmap is directional. Implementation evidence may require milestone scope to change, but planned work must never be presented as already implemented.

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

## Current foundation — `v0.0.x`

### Completed

- [x] `v0.0.1` — UEFI boot, Rust `no_std` kernel entry, serial diagnostics, framebuffer access.
- [x] `v0.0.2` — GDT/TSS, IDT, exceptions, PIC 8259, PIT setup, tick foundation.
- [x] `v0.0.3` — physical memory map handling and 4 KiB frame allocator model.
- [x] `v0.0.4` — initial static 64 KiB kernel heap.
- [x] `v0.0.5` — basic PS/2 keyboard input through IRQ1.
- [x] `v0.0.6` — task identity/state model with persistent runtime ownership.
- [x] `v0.0.7` — initial round-robin scheduler policy driven by real 100 Hz PIT ticks, with a 5-tick quantum and runtime tick validation under QEMU.
- [x] `v0.0.8` — controlled Ring 3 transition foundation with dedicated user code/stack mappings and verified privilege return through the kernel TSS stack.
- [x] `v0.0.9` — minimal independent `no_std` ABI crate and experimental Ring 3 syscall entry/return path through vector `0x80`.

The `v0.0.x` series established the first userspace boundary but remains kernel bring-up. Its userspace ABI is experimental and does not promise compatibility, a general syscall surface, process isolation, broad hardware support, or production security.

---

## `v0.1.0` — First isolated userspace

### Work already verified during `v0.1.0` development

- [x] architecture-level cooperative context save/restore for `rsp` and x86_64 callee-saved registers;
- [x] fresh kernel-context preparation through an independent physical-frame-backed stack;
- [x] two bootstrap/worker context-switch round trips under QEMU;
- [x] existing Ring 3 syscall entry/return remains functional after the context-switch probe.

These checks are **foundational evidence**, not completion of `v0.1.0`. The current `Task` model does not yet own saved contexts/stacks, and the PIT-driven scheduler does not yet perform CPU context switches.

### Remaining target capabilities

- preemptive scheduler foundation that actually switches execution contexts;
- context/stack lifecycle integrated into kernel tasks;
- repeated execution of multiple kernel task bodies;
- first userspace `init` process;
- minimal userspace shell;
- basic process termination path;
- syscall dispatch beyond the current entry/return probe where required by the first userspace program.

Controlled Ring 3 transition, the experimental ABI crate, and the first syscall entry/return path were validated in `v0.0.8` and `v0.0.9`; `v0.1.0` must integrate them into a real task/process lifecycle rather than merely repeat the probes.

### Exit criteria

- multiple tasks execute repeatedly without corrupting kernel state;
- timer-driven scheduling performs repeatable execution-context changes under QEMU;
- task-owned kernel stack/context lifecycle is explicit;
- Ring 3 code cannot directly perform privileged kernel operations;
- syscall entry/return survives repeated calls from the first userspace program;
- kernel and userspace stacks are clearly separated;
- first userspace `init` and minimal shell execute through documented kernel interfaces;
- tests and documentation describe scheduler, context, stack, and privilege-transition invariants;
- CI and target build are green.

---

## `v0.2.0` — Process and virtual-memory model

### Target capabilities

- explicit process and thread model;
- per-process address spaces;
- virtual memory manager;
- page mapping and unmapping APIs;
- page permissions and user/kernel separation;
- process and thread lifecycle;
- exit/reaping semantics;
- ELF executable loader;
- user and kernel stack allocation lifecycle;
- IPC foundation where required by actual consumers.

### Exit criteria

- independent processes cannot access each other's private mappings under supported test scenarios;
- executable loading is validated with malformed-input tests;
- address-space teardown does not leak or double-release owned mappings in tested paths;
- fault handling identifies process-local faults without silently corrupting unrelated state;
- lifecycle and mapping invariants are documented and tested.

---

## `v0.3.0` — Storage and filesystem foundation

### Target capabilities

- virtual filesystem layer;
- handle/file-descriptor model;
- in-memory filesystem suitable for tests and bootstrapping;
- device filesystem or equivalent device namespace;
- block-device abstraction;
- persistent filesystem support;
- buffered I/O foundation;
- cache policy foundation;
- explicit filesystem error propagation;
- recovery behavior for expected storage failures.

### Exit criteria

- files can be created, read, written, reopened, and deleted on at least one supported persistent filesystem;
- invalid paths and malformed filesystem structures fail safely;
- storage tests include abrupt-error and corruption scenarios appropriate to the implemented filesystem;
- user/kernel access boundaries for file operations are documented.

---

## `v0.4.0` — Driver and platform framework

### Target capabilities

- explicit driver registration/lifecycle model;
- PCI/PCIe enumeration;
- ACPI integration sufficient for supported platform discovery;
- VirtIO support where useful for the reference QEMU platform;
- NVMe and/or AHCI storage support;
- HID foundation;
- interrupt-routing improvements;
- device discovery and lifecycle events;
- DMA ownership rules suitable for supported drivers.

### Exit criteria

- reference devices are discovered deterministically;
- at least one supported storage path works beyond synthetic in-memory storage;
- driver failure paths are observable and documented;
- hardware access remains behind explicit architecture/HAL/driver boundaries;
- supported device IDs/classes are documented rather than implied globally.

---

## `v0.5.0` — Networking

### Target capabilities

- network-device interface;
- Ethernet framing;
- ARP and IPv6 neighbor-discovery foundation as applicable;
- IPv4;
- IPv6 foundation;
- ICMP;
- UDP;
- TCP;
- DNS resolver;
- socket API exposed through userspace ABI;
- network error and timeout handling.

### Exit criteria

- userspace can perform basic TCP and UDP communication on a supported network device;
- malformed packet parsing is fuzzed or equivalently stress-tested;
- network state survives repeated connect/disconnect/error scenarios;
- API behavior and unsupported protocol features are documented.

---

## `v0.6.0` — SMP and modern machine support

### Target capabilities

- multiprocessor startup;
- local APIC and IOAPIC support;
- multicore scheduler;
- per-CPU state;
- synchronization primitives suitable for SMP;
- interrupt routing on multicore systems;
- high-resolution timer foundation;
- CPU topology discovery;
- power-management foundation where practical.

### Exit criteria

- scheduler stress tests run across multiple virtual CPUs;
- shared kernel structures have documented synchronization rules;
- repeated multicore boot/shutdown tests do not expose known race-related corruption;
- lock ordering or equivalent deadlock-avoidance policy is documented;
- single-core fallback remains functional.

---

## `v0.7.0` — Security and owner-controlled trust

### Target capabilities

- capability-oriented or rights-scoped resource access;
- stronger process isolation boundaries;
- secure random-number source suitable for key generation on supported hardware;
- privilege boundaries for sensitive kernel resources;
- owner-controlled trust policy;
- optional owner-controlled Secure Boot strategy;
- secret/key access primitives;
- hardened validation of userspace-provided kernel inputs;
- security audit preparation.

The kernel should provide security primitives. Higher-level cryptographic protocols should remain in userspace unless kernel placement is justified by a concrete requirement.

### Exit criteria

- authority is explicit enough to test which resources a process may access;
- privilege failures are deterministic and auditable;
- random-number assumptions are documented per supported platform;
- security-sensitive interfaces have negative tests, not only success tests;
- no project-controlled master key or mandatory remote authority is required for supported local operation.

---

## `v0.8.0` — General device and desktop foundations

### Target capabilities

- USB host-stack foundation;
- USB HID;
- richer keyboard/mouse/input model;
- device hotplug;
- graphics-device abstraction;
- framebuffer/display foundation beyond boot-time painting;
- audio foundation;
- power/suspend work where practical;
- device-facing userspace interfaces where appropriate.

This milestone does **not** imply a complete desktop environment.

### Exit criteria

- supported USB/input devices can be added/removed without kernel corruption;
- graphics and audio foundations have explicit supported-device scope;
- device lifecycle and hotplug failure paths are tested;
- no claim of generic PC peripheral compatibility is made without qualification.

---

## `v0.9.0` — Stabilization program

This phase is expected to be one of the most demanding in the project.

### Required work

- fuzzing of parsers and syscall-facing inputs;
- sustained scheduler stress tests;
- SMP race testing;
- fault injection;
- storage/filesystem recovery testing;
- network robustness testing;
- leak and lifetime analysis;
- panic/crash diagnostics;
- performance profiling;
- boot/reboot/shutdown reliability testing;
- reproducible build verification;
- security review;
- supported-hardware qualification;
- upgrade and migration testing;
- explicit known-issues tracking.

### Exit criteria

- release-blocking correctness/security categories are defined;
- supported configurations pass repeatable long-duration test suites;
- known data-corruption or privilege-isolation defects block promotion;
- crash diagnostics are sufficient to investigate failures;
- release artifacts can be reproduced according to documented procedure.

---

## `v0.9.x` — Release candidates

### Goals

- freeze a candidate stable userspace ABI;
- freeze documented kernel/userspace contracts needed by supported applications;
- define compatibility and deprecation rules;
- remove known release-blocking corruption, isolation, and boot-reliability bugs;
- finalize supported hardware tiers;
- validate upgrade behavior;
- validate release artifacts and reproducibility;
- complete pre-1.0 security review work.

An ABI freeze candidate may still change if a release-blocking design defect is discovered. Stability guarantees begin only when explicitly documented.

---

## `v1.0.0` — Production-grade baseline

`v1.0.0` means **stable for documented supported configurations**. It does not mean universal PC compatibility and does not claim Windows/Linux-level hardware coverage.

### Minimum expectations

- stable, documented userspace ABI for supported use cases;
- reliable process/thread isolation;
- reliable virtual memory management;
- production-quality scheduler on supported CPU configurations;
- persistent storage and filesystem support;
- networking suitable for normal supported applications;
- mature error handling and crash diagnostics;
- owner-controlled security/trust model implemented for supported scenarios;
- reproducible release builds;
- regression test suite;
- documented hardware support matrix;
- documented upgrade and compatibility policy;
- documented security reporting policy;
- no known release-blocking data-corruption or privilege-isolation defects within the supported matrix.

### Release gate

A `v1.0.0` release must not be cut solely because roadmap features exist. The stabilization evidence must support the release claim.

---

## `v1.x` — Expansion without breaking the stable baseline

Likely focus areas:

- broader driver coverage;
- additional filesystems;
- performance and latency work;
- power management;
- security hardening;
- compatibility guarantees and deprecation processes;
- broader real-hardware qualification;
- ARM64 bring-up when implementation resources and architecture maturity justify it.

Minor releases should preserve documented stable interfaces unless a security or correctness emergency requires an explicitly managed exception.

---

## `v2.x+` — Long-term direction

Potential work includes:

- desktop-class graphics/compositor ecosystem;
- richer application runtime;
- application/package distribution model;
- decentralized identity and services aligned with owner sovereignty;
- additional CPU architectures;
- compatibility layers where technically justified;
- stronger userspace driver/service isolation where IPC and capabilities make it practical.

These are directional goals, not promises or release commitments.

---

# Hardware support tiers

Before `v1.0.0`, Nastalli should publish a support matrix using explicit tiers.

## Tier 1 — Reference / release-blocking

- continuously or routinely exercised by project CI or release validation;
- regressions may block a release;
- full documented feature expectations for that release apply.

Initially, x86_64 QEMU/OVMF is the reference environment. It should not automatically remain Tier 1 forever if the stable release definition requires real hardware.

## Tier 2 — Officially supported

- expected to work for documented features;
- receives compatibility attention;
- may receive less continuous testing than Tier 1.

## Tier 3 — Experimental / community-supported

- useful development targets;
- compatibility may regress;
- no release-blocking guarantee unless promoted.

## Unsupported

No compatibility or reliability guarantee. Unsupported does not mean intentionally broken; it means the project has insufficient evidence to make a support claim.

---

# General milestone advancement criteria

Every milestone must satisfy the checks applicable to its scope:

- correct-target build succeeds;
- formatting checks pass;
- applicable Clippy checks pass with project warning policy;
- automated unit/integration tests pass;
- QEMU validation passes when target behavior requires it;
- relevant serial/runtime behavior is observed when applicable;
- documentation is synchronized with implementation;
- known limitations are documented;
- CI is green on the milestone/release commit.

As maturity increases, the gate expands to include:

- fuzzing;
- fault injection;
- sustained stress testing;
- SMP race testing;
- supported-hardware testing;
- security review;
- reproducible builds;
- upgrade/migration tests;
- performance regression checks.

## No "100% stable" claim

Nastalli will not use subjective claims such as "100% stable" as an engineering criterion. Stability must be scoped to documented supported configurations and backed by repeatable evidence.
