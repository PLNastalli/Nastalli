# Contributing to Nastalli

Thank you for your interest in Nastalli. The project is still in early kernel development, so small, verifiable changes are more valuable than large speculative abstractions.

## Before making a change

Read the relevant documentation first:

- [Architecture](docs/architecture.md)
- [`unsafe` policy](docs/unsafe-policy.md)
- [Security model](docs/security-model.md)
- [Roadmap](docs/roadmap.md)
- [Security reporting](SECURITY.md)

Every contribution should answer a concrete engineering need. Avoid adding empty modules, placeholder crates, generalized frameworks, or compatibility layers before there is real code that requires them.

## Architecture boundaries

The current workspace intentionally separates responsibilities:

- `crates/boot` — bootloader integration and kernel entry point;
- `crates/arch` — architecture-specific privileged operations and x86_64 hardware access;
- `crates/hal` — safe hardware-facing abstractions consumed by higher layers;
- `crates/kernel` — architecture-independent kernel policy and core state where practical;
- `tools/xtask` — host-side build, image, test, and run automation.

Do not bypass these boundaries for convenience. If the kernel needs a privileged operation, prefer extending the appropriate `arch` or HAL interface instead of scattering architecture-specific or `unsafe` code through higher layers.

## Documentation is part of the change

Any meaningful change to behavior, architecture, build requirements, security assumptions, supported hardware, or roadmap status must update the relevant documentation in the same development cycle.

At minimum, consider whether the change affects:

- `README.md` for public project status;
- `docs/architecture.md` for boundaries and design decisions;
- `docs/progress.md` for verified implementation evidence;
- `docs/roadmap.md` for milestone status;
- a subsystem-specific document such as `docs/memory.md`, `docs/tasks.md`, or `docs/input.md`.

Documentation must never present planned work as implemented.

## Local verification

Run the checks applicable to your change:

```bash
cargo fmt --all -- --check
cargo xtask test
cargo check -p nastalli-arch -p nastalli-hal -p nastalli-kernel -p xtask
cargo clippy -p nastalli-arch -p nastalli-hal -p nastalli-kernel -p xtask -- -D warnings
cargo xtask build
```

If the change affects boot, interrupts, memory initialization, devices, runtime behavior, or anything that can only be validated in the target environment, also run:

```bash
cargo xtask run
```

For headless QEMU validation:

```bash
NASTALLI_QEMU_DISPLAY=none cargo xtask run
```

Record relevant serial output and limitations in `docs/progress.md` when the result is part of a milestone claim.

## `unsafe` and privileged code

Safe Rust is the default.

New `unsafe`, inline assembly, raw pointer manipulation, port I/O, privileged register changes, interrupt-controller operations, or ABI boundary code must:

1. exist at the narrowest appropriate boundary;
2. document the invariant that makes the operation valid;
3. avoid exposing raw unsafety to callers when a safe interface is possible;
4. receive especially careful review.

Do not use `unsafe` merely to work around ownership or borrowing constraints.

## Commits

Use concise, descriptive commit messages. Examples:

```text
feat: add initial task state transition
fix: preserve task table ownership
arch: prepare timer interrupt routing
docs: clarify memory allocator invariants
ci: install required llvm tools
```

A commit should ideally represent one coherent change that can be reviewed and reasoned about independently.

## Pull requests

If you use a pull request, describe:

- the concrete problem;
- the chosen approach;
- architectural impact;
- safety or security impact;
- verification performed;
- known limitations;
- documentation updated.

Changes involving boot, memory, interrupts, privilege transitions, scheduling, synchronization, security boundaries, ABI design, persistent storage, or new `unsafe` code require stricter review than ordinary refactors or documentation changes.

## Scope discipline

Nastalli is a long-term project, but each milestone should remain small enough to verify. Do not combine unrelated subsystems simply because they are all part of the eventual operating system.

The roadmap is directional. Implementation evidence takes precedence over planned version numbers.
