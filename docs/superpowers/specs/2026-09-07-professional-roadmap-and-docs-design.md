# Nastalli professional roadmap and documentation design

Date: 2026-09-07
Status: Approved and implemented on `main`

## Purpose

Professionalize Nastalli's public GitHub presence and define a long-term engineering roadmap from the current experimental kernel to a production-grade 1.0 release.

The documentation must remain technically honest. It must never imply Windows/Linux-level hardware coverage or universal stability before evidence exists. The 1.0 target is therefore defined as stable within an explicit support matrix, with a stable userspace ABI, documented upgrade guarantees, reproducible builds, and release-quality validation.

## Public positioning

Nastalli is an experimental operating system and kernel written primarily in Rust. Its long-term design goals are:

- memory safety by default;
- modular kernel architecture;
- owner-controlled security and trust;
- no mandatory remote authority, account, telemetry, or project-controlled master key;
- explicit hardware support tiers;
- stable public interfaces only after sufficient implementation and validation;
- documentation that reflects actual behavior rather than planned behavior.

Suggested project statement:

> Nastalli is an experimental operating system and kernel written primarily in Rust, designed around memory safety, modularity, user sovereignty, and long-term architectural independence.

Suggested ownership principle:

> The machine belongs to its owner. Nastalli protects the user without taking ownership away from them.

## Documentation language

All public project documentation should be English-first.

Files to revise include:

- `README.md`
- `CONTRIBUTING.md`
- `SECURITY.md`
- `docs/README.md`
- `docs/architecture.md`
- `docs/boot.md`
- `docs/heap.md`
- `docs/input.md`
- `docs/memory.md`
- `docs/progress.md`
- `docs/roadmap.md`
- `docs/security-model.md`
- `docs/tasks.md`
- `docs/unsafe-policy.md`

Existing historical implementation facts must be preserved when translated or rewritten.

## README structure

The top-level README should be concise and project-oriented rather than serving as a full design document.

Recommended sections:

1. Project name and one-sentence description
2. CI/status badges
3. Current status warning
4. Current capabilities
5. Design principles
6. Architecture overview
7. Roadmap summary
8. Build and run instructions
9. Documentation index
10. Contributing
11. Security
12. Licensing/status note

The README must clearly distinguish current implementation from long-term plans.

## Architecture direction

The long-term documentation should describe the intended layering without claiming that all layers already exist:

```text
Applications / Services
        |
        v
Stable Userspace ABI
        |
        v
+----------------------------+
|      Nastalli Kernel       |
|                            |
| Scheduler                  |
| Virtual Memory             |
| IPC                        |
| Capabilities               |
| VFS                        |
| Process / Thread Model     |
| Device Management          |
+-------------+--------------+
              |
             HAL
              |
     +--------+--------+
    x86_64           ARM64
```

The current architecture remains intentionally small: `boot`, `arch`, `hal`, `kernel`, and `xtask`. New crates or subsystems are introduced only when a real implementation requires them.

## Release maturity model

Nastalli releases should communicate maturity explicitly:

- `0.0.x`: foundation experiments and kernel bring-up
- `0.1.x` to `0.8.x`: subsystem development and integration
- `0.9.x`: stabilization, ABI freeze candidates, hardware qualification, reliability work
- `1.0.0`: production-grade baseline for explicitly supported configurations
- `1.x`: compatibility, performance, driver coverage, and supported-platform expansion
- `2.x+`: broader desktop-class ecosystem and larger architectural evolution

A version number must not imply support beyond documented support tiers.

## Long-term roadmap

### Current foundation — `v0.0.x`

Implemented or currently in progress:

- UEFI boot
- Rust `no_std` kernel entry
- serial diagnostics
- framebuffer access
- GDT/TSS
- IDT and exception handling
- PIC/PIT foundation
- physical memory map handling
- initial kernel heap
- PS/2 keyboard input
- task identity/state model

### `v0.1.0` — First isolated userspace

Target capabilities:

- initial preemptive scheduler
- context switching
- kernel stacks
- controlled Ring 3 transition
- syscall entry/return path
- minimal independent ABI crate
- first userspace `init`
- minimal userspace shell

Exit criteria include repeatable multitasking tests and validated privilege separation.

### `v0.2.0` — Process and virtual-memory model

Target capabilities:

- processes and threads
- per-process address spaces
- virtual memory manager
- page mapping/unmapping APIs
- user/kernel stack lifecycle
- ELF executable loader
- process lifecycle and exit/reaping semantics
- IPC foundation where required

### `v0.3.0` — Storage and filesystem foundation

Target capabilities:

- VFS
- handles/file-descriptor model
- ramfs/devfs-like internal filesystems
- block-device abstraction
- persistent filesystem support
- buffered I/O and cache policy foundation
- filesystem error and corruption handling

### `v0.4.0` — Driver and platform framework

Target capabilities:

- explicit driver model
- PCI/PCIe enumeration
- ACPI integration
- VirtIO block/input/network/graphics foundations as applicable
- NVMe and/or AHCI storage
- HID foundation
- interrupt-routing improvements
- device discovery and lifecycle

### `v0.5.0` — Networking

Target capabilities:

- network-device interface
- Ethernet
- ARP/NDP as applicable
- IPv4 and IPv6 foundation
- ICMP
- UDP
- TCP
- DNS resolver
- socket API
- userspace networking tests

### `v0.6.0` — SMP and modern machine support

Target capabilities:

- multiprocessor startup
- APIC/IOAPIC
- multicore scheduler
- synchronization primitives suitable for SMP
- per-CPU state
- high-resolution timers
- power-management foundation
- race-condition and stress validation

### `v0.7.0` — Security and owner-controlled trust

Target capabilities:

- capability-oriented resource access
- process isolation hardening
- secure randomness
- privilege boundaries
- owner-controlled trust policy
- optional owner-controlled Secure Boot strategy
- secret/key access primitives
- security audit preparation

The kernel should provide primitives; higher-level cryptographic protocols remain in userspace unless kernel placement is justified.

### `v0.8.0` — General device and desktop foundations

Target capabilities:

- USB host stack foundation
- USB HID
- richer input model
- graphics/device abstraction
- audio foundation
- device hotplug
- power/suspend work where practical

This release does not imply a complete desktop environment.

### `v0.9.0` — Stabilization program

This phase is expected to be one of the most demanding.

Required work includes:

- fuzzing of parsers and kernel interfaces
- sustained stress tests
- scheduler and SMP race testing
- fault injection
- filesystem/storage recovery tests
- network robustness tests
- memory-leak and lifetime analysis
- panic/crash diagnostics
- performance profiling
- boot/reboot/shutdown reliability
- security review
- supported-hardware qualification
- reproducible-build verification
- upgrade/downgrade behavior documentation

### `v0.9.x` — Release candidates

Goals:

- freeze candidate userspace ABI
- freeze documented kernel/userspace contracts
- remove known release-blocking corruption or isolation bugs
- define supported hardware tiers
- validate release artifacts
- complete migration and upgrade guarantees

### `v1.0.0` — Production-grade baseline

`v1.0.0` means stable for documented supported configurations, not universal PC compatibility.

Minimum expectations:

- stable documented userspace ABI
- reliable process/thread isolation
- reliable virtual memory
- production-quality scheduler on supported CPU configurations
- persistent storage and filesystem support
- networking suitable for normal application use
- mature error handling and crash diagnostics
- owner-controlled security model
- reproducible release builds
- regression test suite
- hardware support matrix
- upgrade policy
- security reporting policy
- no release-blocking known data-corruption or privilege-isolation defects within the supported matrix

### `v1.x`

Focus:

- broader driver coverage
- additional filesystems
- performance
- power management
- security hardening
- compatibility guarantees
- ARM64 bring-up when justified
- broader real-hardware qualification

### `v2.x+`

Potential long-term work:

- desktop-class graphics/compositor ecosystem
- richer application runtime
- package/application distribution model
- decentralized identity/services aligned with owner sovereignty
- additional architectures
- compatibility layers where technically justified

These items are directional, not promises.

## Hardware support tiers

Before 1.0, the documentation should introduce a support matrix with explicit categories, for example:

- Tier 1: release-blocking, continuously tested configurations
- Tier 2: officially supported but less extensively exercised configurations
- Tier 3: experimental/community-supported configurations
- Unsupported: no compatibility guarantee

QEMU/OVMF x86_64 should remain the initial reference platform until real-hardware qualification exists.

## Stability criteria

The project must avoid subjective claims such as "100% stable". Stability claims should be based on evidence.

A milestone may advance only when the applicable criteria pass:

- formatting and lint checks
- automated unit/integration tests
- architecture-specific build
- QEMU boot validation where relevant
- subsystem stress tests where relevant
- documentation synchronized with implementation
- known limitations documented
- CI green on the release commit

For later milestones, add fuzzing, fault injection, supported-hardware tests, reproducible builds, security review, and upgrade tests.

## Documentation rules

1. Current behavior and planned behavior must always be visibly separated.
2. Planned subsystems must never be described as implemented.
3. Every meaningful architecture, build, security, or roadmap change updates the relevant documentation in the same development cycle.
4. `docs/progress.md` remains evidence-oriented and records what actually passed.
5. `docs/roadmap.md` remains directional and may change as implementation teaches new constraints.
6. `docs/architecture.md` explains boundaries and design decisions, not marketing claims.
7. Security guarantees must be scoped to implemented mechanisms and supported configurations.

## GitHub professionalism scope

This redesign intentionally includes only practices useful at the current project size:

- polished English README
- English technical documentation
- clear roadmap and release semantics
- contribution guidance
- security guidance
- CI status visibility
- explicit experimental warning
- architectural principles

It intentionally does not add heavyweight governance, committees, formal RFC bureaucracy, or enterprise process before there are enough contributors to justify them.

## Non-goals

This documentation project does not:

- implement the future roadmap;
- claim broad hardware compatibility;
- promise fixed release dates;
- create empty subsystem crates for future features;
- redesign the current working kernel architecture without an implementation need;
- claim parity with Windows or Linux hardware coverage.

## Acceptance criteria

The documentation redesign is complete when:

- public project documentation is consistently English-first;
- the README is concise and professional;
- the full roadmap through 1.0 and beyond is documented;
- release maturity and stability claims are objectively defined;
- support tiers are described;
- current v0.0.6 implementation facts remain accurate;
- build commands remain correct;
- links between documentation files are valid;
- contribution and security documents match the project's actual maturity;
- CI remains green after documentation changes.
