# Nastalli v0.0.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Boot a Rust `no_std` kernel on x86_64/UEFI and validate serial plus framebuffer output under QEMU.

**Architecture:** `boot` receives `BootInfo` and calls `kernel`; `kernel` depends on architecture information through `arch` and safe serial access through `hal`. `xtask` produces the UEFI image and starts QEMU.

**Tech Stack:** Rust Edition 2024, Cargo workspace, `bootloader`/`bootloader_api` 0.11.10, x86_64, UEFI, QEMU, and OVMF.

**Spec:** `docs/superpowers/specs/2026-09-06-nastalli-v001-design.md`

## Global Constraints

- Safe Rust by default; assembly belongs only in `crates/arch`.
- Do not create ABI, userspace, scheduler, syscalls, filesystem, or complex driver subsystems in this milestone.
- Keep only the minimal crates justified by working code.
- Verify through formatting, checks, Clippy, image creation, and QEMU boot.

### Task 1: Minimal workspace and kernel bring-up

- [ ] Create the manifests, pinned toolchain, and target configuration.
- [ ] Implement the entry point, panic handler, serial output, and minimal framebuffer behavior.
- [ ] Implement `xtask build`, `image`, `run`, and `test` commands.
- [ ] Document layer boundaries and success criteria.

### Task 2: End-to-end verification

- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo check` and `cargo clippy`.
- [ ] Run `cargo xtask build`, `cargo xtask image`, and `cargo xtask test`.
- [ ] Start QEMU and confirm serial diagnostics and framebuffer output.
