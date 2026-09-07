# Security and Ownership Model

## Purpose

Nastalli's long-term security model is owner-controlled: the machine owner should remain the final authority over the device.

Security does not mean the kernel should silently take ownership away from the user. It means authority boundaries should be explicit, auditable, and enforceable without depending on a mandatory remote project authority.

## Current reality

As of `v0.0.6`, Nastalli is **not** a production security platform.

The current codebase does not yet implement:

- separate userspace processes;
- capability enforcement;
- a stable syscall security boundary;
- disk encryption;
- owner-controlled Secure Boot;
- a secure key-management subsystem;
- hardened driver isolation;
- a production-grade cryptographic RNG path;
- broad real-hardware security qualification.

The current security contribution is structural rather than feature-complete: Safe Rust by default, concentrated `unsafe`, explicit layer boundaries, no mandatory remote service, and no project-controlled master key architecture.

## Long-term invariants

The project intends to preserve these invariants as the system matures:

1. **No Nastalli master key** should exist that can unlock every user's machine.
2. **No mandatory online account** should be required for local boot and ordinary supported local operation.
3. **No mandatory telemetry** should be required for the OS to function.
4. **Secure Boot trust should be owner-controlled** when Secure Boot support is implemented.
5. **Persistent data encryption should support owner-controlled keys** rather than mandatory project escrow.
6. **Processes should receive explicit resource authority**, preferably through rights-scoped handles/capabilities rather than ambient global privilege.
7. **Application-level end-to-end encryption belongs primarily in userspace.** The kernel should provide isolation, secure randomness, memory protection, transport primitives, and controlled key access—not inspect application message contents by default.
8. **A driver or service compromise should not automatically grant unrelated authority** once the architecture provides isolation strong enough to enforce that rule.

These are design constraints. Each becomes a real security guarantee only after the corresponding mechanism is implemented and validated.

## Trust and boot

Long-term boot/security work should support:

- a documented chain of trust;
- optional Secure Boot using keys controlled by the machine owner;
- local recovery without mandatory project escrow;
- signed update verification without making the project the user's unavoidable root of trust;
- an explicit way for the owner to replace or revoke trusted keys.

The exact Secure Boot/update architecture is not frozen yet and should be designed only when the required boot, key, and recovery primitives exist.

## Process and resource isolation

Future isolation is expected to include:

- per-process virtual address spaces;
- user/kernel privilege separation;
- explicit syscall validation;
- rights-scoped handles or capabilities;
- controlled IPC;
- resource ownership and lifetime rules;
- stronger driver/service isolation where practical.

The kernel should be able to answer a concrete question for every protected resource: **which principal has which authority, and why?**

## Keys and persistent data

Future persistent-data security should favor:

- keys generated locally from a suitable entropy source;
- owner-controlled recovery choices;
- encryption keys unavailable to unrelated processes by default;
- explicit key access rights;
- logs and diagnostics that avoid leaking secret material;
- documented behavior when key material is lost or deliberately destroyed.

Recovery is a product/policy choice, not an excuse to introduce an undeclared project backdoor.

## Updates and remote authority

A secure update mechanism may verify signatures, versions, hashes, and compatibility metadata. That does not require updates to be forced or require a single mandatory distribution server.

The long-term design should allow the owner to understand and control the system's trust roots while still receiving strong integrity guarantees.

## Threat-model boundaries

Even a mature Nastalli release cannot guarantee protection against every threat.

Examples that may fall partly or entirely outside OS guarantees include:

- malicious or compromised firmware below the kernel's trust boundary;
- physically modified hardware;
- DMA-capable devices when no effective isolation mechanism is available;
- a user intentionally granting full authority to malicious code;
- physical attackers with sufficient time and equipment against unprotected hardware;
- CPU/platform vulnerabilities outside the kernel's ability to mitigate fully.

These limits should be documented rather than obscured by broad security slogans.

## Security maturity and support matrix

Future security claims must be scoped to the hardware/software configurations listed in the release support matrix.

A mechanism working under QEMU does not by itself establish the same guarantee on arbitrary real hardware.

Before `v1.0.0`, security qualification should include negative testing, fuzzing of exposed parsers/interfaces, privilege-boundary tests, fault injection where appropriate, and review of supported configurations.

## Owner override and safety

The long-term philosophy is to distinguish **protection** from **ownership**.

Nastalli may warn, require explicit confirmation, preserve rollback/recovery data, or make dangerous operations difficult to perform accidentally. It should avoid treating a project-controlled remote authority as more legitimate than the machine owner.

Owner control does not mean every process receives unrestricted access. Ordinary applications should remain strongly isolated; owner authority should be exercised through explicit administrative/trust decisions.

## Related documents

- [`SECURITY.md`](../SECURITY.md) — vulnerability reporting and current support policy
- [`unsafe-policy.md`](unsafe-policy.md) — low-level Rust/privileged-code rules
- [`architecture.md`](architecture.md) — current and target architecture
- [`roadmap.md`](roadmap.md) — security and stabilization milestones
