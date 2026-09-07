# Nastalli Documentation

Nastalli treats documentation as part of the engineering artifact. Public documentation must describe what the code actually does, why important decisions were made, what remains unsupported, and how claims were verified.

## Project overview

| Document | Purpose |
|---|---|
| [Architecture](architecture.md) | Current layer boundaries, dependency direction, and long-term architectural direction |
| [Roadmap](roadmap.md) | Engineering milestones, maturity model, hardware support tiers, and release criteria |
| [Technical progress](progress.md) | Version-by-version implementation history and validation evidence |
| [Security and ownership model](security-model.md) | Owner-controlled trust goals, current security state, and threat-model boundaries |
| [`unsafe` policy](unsafe-policy.md) | Rules for unsafe Rust, assembly, privileged operations, and trust boundaries |

## Current implementation

| Document | Purpose |
|---|---|
| [Boot flow](boot.md) | UEFI → bootloader → kernel chain, image creation, OVMF discovery, and run commands |
| [Physical memory](memory.md) | Current 4 KiB physical-frame allocator model and invariants |
| [Kernel heap](heap.md) | Current static 64 KiB kernel heap and its limitations |
| [Input](input.md) | Current PS/2 keyboard IRQ1 path and supported scancodes |
| [Task model](tasks.md) | v0.0.6 task identity/state model and persistent runtime ownership |

## Design and implementation records

Longer design specifications and implementation plans live under:

```text
docs/superpowers/specs/
docs/superpowers/plans/
```

The current documentation professionalization design is:

- [Professional roadmap and documentation design](superpowers/specs/2026-09-07-professional-roadmap-and-docs-design.md)

These records explain intent. They do **not** override the current implementation state described by the main documentation.

## Documentation rules

Every meaningful change should consider whether it affects:

1. `README.md` — public status and high-level capabilities;
2. `architecture.md` — boundaries, dependency direction, and architectural decisions;
3. `progress.md` — verified implementation evidence;
4. `roadmap.md` — milestone state or long-term direction;
5. the relevant subsystem document;
6. `SECURITY.md` or `security-model.md` when trust or security assumptions change.

A feature must not be marked complete merely because code was written. Claims should be supported by the checks appropriate to that feature: compilation, automated tests, target build, QEMU validation, stress testing, hardware qualification, or security review as maturity increases.

## Current maturity

Nastalli is currently at `v0.0.6`, in early kernel bring-up. The reference environment is x86_64 under QEMU/OVMF. Features listed for later milestones are design direction, not current capabilities.
