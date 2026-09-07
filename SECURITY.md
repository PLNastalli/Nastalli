# Security Policy

## Current security status

Nastalli is experimental kernel software and is **not yet suitable for production or security-sensitive systems**. The current v0.0.6 codebase does not provide complete user isolation, disk encryption, Secure Boot policy, capabilities, mature privilege separation, or a stable security boundary for real-world workloads.

Security claims in this repository must always be scoped to mechanisms that actually exist.

## Security principles

The long-term security model follows several project-level constraints:

- the machine owner should control the device's trust policy;
- Nastalli must not depend on a project-controlled master key capable of unlocking user systems;
- no mandatory online account or mandatory telemetry should be required to boot the system;
- owner-controlled Secure Boot should be possible when that subsystem is implemented;
- `unsafe` should remain small, documented, and concentrated at hardware, boot, ABI, and other unavoidable trust boundaries;
- failures in one service or driver should not automatically imply authority over unrelated resources once isolation mechanisms exist.

See [docs/security-model.md](docs/security-model.md) for the design direction and [docs/unsafe-policy.md](docs/unsafe-policy.md) for current unsafe-code rules.

## Reporting a vulnerability

For a vulnerability that could materially affect users or future supported configurations, **do not publish immediately exploitable details in a public issue before a fix or responsible coordination is possible**.

Use an available private GitHub security/contact channel for the repository or contact the maintainer privately and request a secure reporting path.

A useful report should include:

- affected version or commit;
- architecture and environment;
- a minimal, safe reproduction;
- observed impact;
- expected behavior;
- whether corruption, privilege escalation, information disclosure, denial of service, or persistence is involved;
- any constraints that reduce or increase exploitability.

Ordinary bugs, build failures, documentation issues, and non-sensitive correctness problems may be reported publicly.

## Scope of future security support

A future stable release will define its security guarantees only for configurations listed in the supported hardware/software matrix. Unsupported or experimental configurations may not receive the same guarantees or response priority.

Until Nastalli reaches a stable release, no compatibility window, security-support lifetime, or patch SLA is promised.

## Responsible disclosure

Please allow reasonable time for investigation, remediation, testing, and coordinated disclosure when a vulnerability could put users at risk. The project will prefer transparent technical disclosure after a fix is available rather than permanent secrecy.
