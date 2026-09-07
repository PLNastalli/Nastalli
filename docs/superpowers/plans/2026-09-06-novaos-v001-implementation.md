# Nastalli OS v0.0.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bootar um kernel Rust `no_std` em x86_64/UEFI e validar serial e framebuffer no QEMU.

**Architecture:** `boot` recebe `BootInfo` e chama `kernel`; `kernel` depende de `arch` somente para o nome da arquitetura e de `hal` para serial. `xtask` gera a imagem UEFI e inicia QEMU.

**Tech Stack:** Rust Edition 2024, Cargo workspace, `bootloader`/`bootloader_api` 0.11.10, x86_64, UEFI, QEMU e OVMF.

**Spec:** `docs/superpowers/specs/2026-09-06-novaos-v001-design.md`

## Global Constraints

- Safe Rust por padrão; assembly somente em `crates/arch`.
- Não criar ABI, userspace, scheduler, syscalls, filesystem ou drivers complexos.
- Manter apenas os cinco crates já justificados por código real.
- Verificar com fmt, check, clippy, image e QEMU.

### Task 1: Workspace e crates mínimos

- [ ] Criar os manifests, toolchain e configuração de target.
- [ ] Implementar entry point, panic handler, serial e framebuffer mínimo.
- [ ] Implementar `xtask build`, `image`, `run` e `test`.
- [ ] Documentar as fronteiras e o critério de sucesso.

### Task 2: Verificação ponta a ponta

- [ ] Rodar `cargo fmt --check`.
- [ ] Rodar `cargo check` e `cargo clippy`.
- [ ] Rodar `cargo xtask build`, `cargo xtask image` e `cargo xtask test`.
- [ ] Iniciar QEMU e confirmar as mensagens seriais e o framebuffer.
