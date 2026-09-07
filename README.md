# Nastalli

[![CI](https://github.com/PLNastalli/Nastalli/actions/workflows/ci.yml/badge.svg)](https://github.com/PLNastalli/Nastalli/actions/workflows/ci.yml)

**Nastalli is an experimental operating system kernel written primarily in Rust, designed around memory safety, modularity, user sovereignty, and long-term architectural independence.**

> [!WARNING]
> Nastalli is under active development and is not yet suitable for production systems, real user data, or security-sensitive workloads.

## Project status

- **Current version:** `v0.0.9`
- **Reference platform:** x86_64 + UEFI + QEMU/OVMF
- **Next milestone:** `v0.1.0` — first isolated userspace foundation
- **Maturity:** early kernel bring-up

The current kernel boots through UEFI, enters Rust `no_std`, initializes GDT/TSS, IDT, PIC 8259 and a 100 Hz PIT timer, handles basic PS/2 keyboard input through IRQ1, reads the bootloader memory map, exposes a 4 KiB physical-frame allocator model, initializes a static 64 KiB kernel heap, owns a persistent task table, applies an initial round-robin scheduler policy with a 5-tick quantum, maps a minimal user code page and user stack, performs a controlled Ring 0 to Ring 3 transition, and validates an experimental userspace-to-kernel software-interrupt entry and return path.

The current userspace probe runs from mapped user pages, invokes the experimental x86_64 syscall entry through software interrupt vector `0x80`, returns to Ring 3, and then triggers a Ring 3 breakpoint used by QEMU smoke validation. The shared contract lives in the independent `nastalli-abi` crate. This is intentionally an early ABI experiment, not a stable application ABI.

The current scheduler milestone still validates timer-driven scheduling policy and task-state rotation rather than full CPU context switching. Nastalli does **not** yet provide independent preemptive task execution, a process model, a general syscall dispatcher, executable loading, per-process address spaces, or a userspace shell.

Nastalli is **not** a Linux distribution and does not reuse the Linux kernel.

## Design principles

- **Safe Rust by default.** `unsafe` is restricted to boundaries where hardware, privileged CPU state, boot integration, or invariants not expressible in the type system require it.
- **Small, explicit boundaries.** Architecture-specific code, hardware abstractions, kernel policy, boot integration, ABI contracts, and development tooling remain separated.
- **Owner-controlled trust.** The long-term design does not depend on a project-controlled master key, mandatory remote authority, mandatory account, or mandatory telemetry.
- **Evidence before claims.** Documentation must distinguish implemented behavior from planned behavior.
- **No speculative structure.** New crates and abstractions are introduced when real implementation needs justify them.

> **The machine belongs to its owner. Nastalli protects the user without taking ownership away from them.**

## Architecture

Current workspace direction:

```text
experimental userspace probe
          |
          v
    nastalli-abi
          |
          v
        kernel
          |
          +------> HAL
          |         |
          +-------->arch
                    |
                    v
          hardware / privileged CPU state
```

`crates/abi` contains the small shared userspace contract. `crates/arch` owns x86_64 privilege-transition, paging, interrupt, and hardware mechanisms. `crates/kernel` owns kernel policy and orchestrates the current Ring 3 probe.

The long-term direction adds processes, virtual memory ownership, a real syscall dispatcher, executable loading, IPC, capabilities, VFS, device management, networking, and broader hardware support. These are roadmap goals and must not be interpreted as currently implemented features.

See [docs/architecture.md](docs/architecture.md) for the current boundaries and long-term direction.

## Roadmap

The roadmap is organized by engineering maturity rather than fixed dates:

| Milestone | Focus |
|---|---|
| `v0.0.x` | Kernel bring-up: boot, interrupts, memory, heap, input, task model, scheduler policy, Ring 3 and initial ABI entry |
| `v0.1.0` | Context switching, syscall dispatch/return hardening, first isolated userspace and minimal shell |
| `v0.2.0` | Processes, threads, virtual memory, ELF loading |
| `v0.3.0` | VFS, persistent storage, block I/O |
| `v0.4.0` | Driver/platform framework, PCIe, ACPI, VirtIO, storage drivers |
| `v0.5.0` | Networking stack and sockets |
| `v0.6.0` | SMP, APIC/IOAPIC, multicore scheduling, synchronization |
| `v0.7.0` | Capability-oriented security and owner-controlled trust |
| `v0.8.0` | USB, richer input, graphics/audio foundations, hotplug |
| `v0.9.x` | Stabilization, fuzzing, fault injection, ABI freeze candidates, hardware qualification |
| `v1.0.0` | Production-grade baseline for explicitly supported configurations |
| `v1.x+` | Broader hardware, performance, additional filesystems and architectures |

`v1.0.0` will mean **stable within a documented support matrix**, not universal Windows/Linux-level hardware compatibility.

See the full [roadmap](docs/roadmap.md).

## Build and run

The repository pins its Rust nightly toolchain in [`rust-toolchain.toml`](rust-toolchain.toml), including the `rust-src` and `llvm-tools-preview` components required by the current bootloader toolchain.

Typical development commands:

```bash
cargo fmt --all -- --check
cargo xtask test
cargo xtask build
cargo xtask image
cargo xtask run
```

Useful environment variables:

```bash
NASTALLI_QEMU_DISPLAY=none cargo xtask run
NASTALLI_OVMF_CODE=/path/to/OVMF_CODE.fd cargo xtask run
NASTALLI_OVMF_VARS=/path/to/OVMF_VARS.fd cargo xtask run
```

`cargo xtask build` targets `x86_64-unknown-none`. `cargo xtask image` creates the UEFI disk image, and `cargo xtask run` starts QEMU using OVMF. For pflash-based OVMF, the writable VARS template is copied into `target/` before QEMU starts so the system firmware template is not modified in place.

## Documentation

- [Documentation index](docs/README.md)
- [Architecture](docs/architecture.md)
- [Roadmap](docs/roadmap.md)
- [Technical progress and validation history](docs/progress.md)
- [Boot flow](docs/boot.md)
- [Physical memory](docs/memory.md)
- [Kernel heap](docs/heap.md)
- [Input](docs/input.md)
- [Task model](docs/tasks.md)
- [Experimental userspace ABI](docs/abi.md)
- [`unsafe` policy](docs/unsafe-policy.md)
- [Security and ownership model](docs/security-model.md)

## Contributing

Contributions should be small, testable, architecture-aware, and documented. Any meaningful change to architecture, build behavior, security assumptions, or roadmap state must update the relevant documentation in the same development cycle.

Read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting changes.

## Security

Nastalli currently provides **no production security guarantee**. Security-sensitive issues should be reported responsibly rather than published with immediately usable exploitation details.

See [SECURITY.md](SECURITY.md) and [docs/security-model.md](docs/security-model.md).

## License status

A final redistribution license has not yet been selected for the first public stable release. Until a license is added, no broad permission to copy, modify, or redistribute the code should be assumed.
