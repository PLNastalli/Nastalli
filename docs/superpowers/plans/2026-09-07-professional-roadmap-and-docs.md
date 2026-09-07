# Professional Roadmap and Documentation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Professionalize Nastalli's public GitHub documentation in English and establish an evidence-based roadmap from the current v0.0.6 kernel to a production-grade v1.0 baseline and beyond.

**Architecture:** Keep the current kernel source structure unchanged. This work is documentation-only: it rewrites the public project surface, preserves verified historical facts, clearly separates current implementation from planned capabilities, and defines release maturity, support tiers, and stability criteria.

**Tech Stack:** Markdown, GitHub Actions status, Rust/Cargo command documentation.

**Spec:** `docs/superpowers/specs/2026-09-07-professional-roadmap-and-docs-design.md`

## Global Constraints

- Apply changes directly to `main` as explicitly requested by the project owner.
- Public documentation is English-first.
- Current implementation remains `v0.0.6`; planned features must never be described as implemented.
- `v1.0.0` means production-grade for explicitly supported configurations, not universal Windows/Linux hardware parity.
- Preserve verified implementation history and existing working build commands.
- Do not create empty future subsystem crates.
- Every meaningful architecture, build, security, or roadmap change must keep documentation synchronized.
- CI must remain green after the documentation changes.

---

### Task 1: Public project surface

**Files:**
- Modify: `README.md`
- Modify: `CONTRIBUTING.md`
- Modify: `SECURITY.md`

**Interfaces:**
- Consumes: current v0.0.6 capabilities and build requirements.
- Produces: concise public positioning, contributor expectations, and security reporting guidance used by all later documentation.

- [ ] **Step 1:** Rewrite `README.md` in professional English with explicit experimental status, current capabilities, design principles, architecture overview, roadmap summary, build/run commands, documentation links, contribution/security pointers, and licensing note.
- [ ] **Step 2:** Rewrite `CONTRIBUTING.md` in English with architecture boundaries, documentation synchronization requirements, local verification commands, commit/PR conventions, and extra review requirements for unsafe/security/boot/memory/interrupt changes.
- [ ] **Step 3:** Rewrite `SECURITY.md` in English, scope security guarantees to implemented mechanisms, preserve owner-controlled trust principles, and document responsible private reporting expectations.
- [ ] **Step 4:** Review all three files for claims that could be read as implemented future features; remove or mark them as planned.
- [ ] **Step 5:** Commit the public-surface changes.

### Task 2: Documentation index, architecture, and roadmap

**Files:**
- Modify: `docs/README.md`
- Modify: `docs/architecture.md`
- Modify: `docs/roadmap.md`

**Interfaces:**
- Consumes: approved professional roadmap spec and current crate layout.
- Produces: the canonical documentation navigation, architecture direction, release maturity model, hardware support tiers, and milestone roadmap.

- [ ] **Step 1:** Rewrite `docs/README.md` as an English documentation index grouped by project overview, current implementation, engineering policies, roadmap/history, and design records.
- [ ] **Step 2:** Rewrite `docs/architecture.md` to document current `boot → kernel → HAL → arch` boundaries, current v0.0.6 reality, owner-controlled design principles, and clearly labeled long-term architecture direction.
- [ ] **Step 3:** Replace `docs/roadmap.md` with the approved roadmap covering `v0.0.x`, `v0.1.0` through `v0.9.x`, `v1.0.0`, `v1.x`, and `v2.x+`; include objective exit criteria, support tiers, and stability rules.
- [ ] **Step 4:** Cross-check README roadmap summary against the canonical roadmap and ensure no milestone contradicts the spec.
- [ ] **Step 5:** Commit architecture and roadmap documentation.

### Task 3: Current subsystem documentation

**Files:**
- Modify: `docs/boot.md`
- Modify: `docs/heap.md`
- Modify: `docs/input.md`
- Modify: `docs/memory.md`
- Modify: `docs/tasks.md`
- Modify: `docs/unsafe-policy.md`
- Modify: `docs/security-model.md`

**Interfaces:**
- Consumes: existing verified subsystem behavior.
- Produces: English subsystem documentation that remains evidence-based and version-scoped.

- [ ] **Step 1:** Translate and professionally rewrite boot documentation without changing verified boot behavior.
- [ ] **Step 2:** Rewrite heap documentation preserving the current static 64 KiB kernel heap and its limitations.
- [ ] **Step 3:** Rewrite input documentation preserving current PS/2 IRQ1 behavior and explicitly excluding unimplemented USB/layout work.
- [ ] **Step 4:** Rewrite physical memory documentation preserving the BootInfo-derived 4 KiB frame allocator behavior and current paging limitations.
- [ ] **Step 5:** Rewrite task documentation preserving the fixed 16-entry table, persistent runtime ownership, and absence of scheduling/context switching in v0.0.6.
- [ ] **Step 6:** Rewrite unsafe policy in English with safe-Rust-by-default rules and documented hardware/boot boundaries.
- [ ] **Step 7:** Rewrite security model in English, separating current guarantees from long-term capability-oriented and owner-controlled security goals.
- [ ] **Step 8:** Commit subsystem documentation.

### Task 4: Evidence history and consistency pass

**Files:**
- Modify: `docs/progress.md`
- Review: all public Markdown files changed by Tasks 1-3

**Interfaces:**
- Consumes: verified historical entries from v0.0.1 through v0.0.6 and the maintenance work after v0.0.6.
- Produces: an English, evidence-oriented technical history and a consistent public documentation set.

- [ ] **Step 1:** Rewrite `docs/progress.md` in English while preserving version-by-version facts, validation evidence, observed serial output, and known limitations.
- [ ] **Step 2:** Add the documentation professionalization work as maintenance after v0.0.6 without presenting it as a functional kernel release.
- [ ] **Step 3:** Search the rewritten public documentation for Portuguese headings/sentences and remove remaining public-language inconsistencies.
- [ ] **Step 4:** Check internal Markdown links and filenames against the repository tree.
- [ ] **Step 5:** Check that every current-version claim matches v0.0.6 implementation and that future roadmap items are marked as planned.
- [ ] **Step 6:** Commit the progress/consistency pass.

### Task 5: Verification

**Files:**
- No production files expected unless verification reveals a documentation defect.

**Interfaces:**
- Consumes: completed documentation set.
- Produces: evidence that documentation-only changes did not break repository checks.

- [ ] **Step 1:** Confirm the final `main` tree contains the expected English public documentation files.
- [ ] **Step 2:** Confirm GitHub Actions completes successfully on the final documentation commit.
- [ ] **Step 3:** If CI fails, inspect the failing job log and fix only the demonstrated root cause.
- [ ] **Step 4:** Re-read `README.md`, `docs/roadmap.md`, and `docs/architecture.md` together to verify release semantics and current/planned boundaries are consistent.
- [ ] **Step 5:** Record the final documentation maintenance and verification result in `docs/progress.md` if any additional evidence needs to be added.
