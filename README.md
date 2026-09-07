# Nastalli OS

[![CI](https://github.com/PLNastalli/Nastalli/actions/workflows/ci.yml/badge.svg)](https://github.com/PLNastalli/Nastalli/actions/workflows/ci.yml)

> Um sistema operacional próprio, modular e seguro, escrito principalmente em Rust.

Nastalli não é uma distribuição Linux nem uma modificação de outro kernel. É um projeto experimental de longo prazo, iniciado com x86_64, UEFI e QEMU, com uma arquitetura preparada para evoluir para múltiplas arquiteturas.

> **Status:** projeto em desenvolvimento ativo. Ainda não é adequado para uso em hardware ou dados reais.

Sistema operacional próprio, escrito principalmente em Rust, começando por x86_64 + UEFI + QEMU. O projeto não é uma distribuição Linux nem uma modificação de outro kernel.

## Estado atual

**Versão:** `v0.0.6`
**Status:** boot validado em QEMU + OVMF
**Próxima versão:** `v0.0.7`, scheduler inicial

A versão atual entra no kernel Rust `no_std`, inicializa GDT/TSS, IDT, PIC 8259 e PIT a 100 Hz, captura teclado PS/2 pela IRQ1, conta frames físicos utilizáveis a partir do `BootInfo`, inicializa uma heap estática de 64 KiB, cria a tabela inicial de tarefas e mantém sua posse no runtime do kernel, acessa o framebuffer e escreve diagnóstico pela serial COM1.

## Arquitetura

```text
Aplicações → runtime/userspace → ABI → syscalls → kernel → HAL → arch → hardware
```

O workspace começa pequeno de propósito: `arch`, `hal`, `kernel`, `boot` e `xtask` só recebem novas responsabilidades quando existe código real que as justifique. Detalhes das fronteiras estão em [docs/architecture.md](docs/architecture.md).

## Documentação

- [Índice da documentação](docs/README.md)
- [Arquitetura](docs/architecture.md)
- [Histórico e decisões por versão](docs/progress.md)
- [Fluxo de boot](docs/boot.md)
- [Política de unsafe](docs/unsafe-policy.md)
- [Modelo de segurança e posse](docs/security-model.md)
- [Roadmap](docs/roadmap.md)

## Comandos

```bash
cargo xtask build   # compila o kernel para x86_64-unknown-none
cargo xtask image   # cria target/nastalli-uefi.img
cargo xtask run     # cria a imagem e inicia QEMU
cargo xtask test    # executa testes host dos crates verificáveis
```

Para ambientes headless, use `NASTALLI_QEMU_DISPLAY=none cargo xtask run`.

## Desenvolvimento

Requisitos: Rust nightly definido em [`rust-toolchain.toml`](rust-toolchain.toml), target `x86_64-unknown-none`, componentes `rust-src` e `llvm-tools-preview`, QEMU e OVMF.

```bash
rustup component add rust-src llvm-tools-preview --toolchain nightly-2025-01-01
cargo fmt --all -- --check
cargo xtask test
cargo xtask build
cargo xtask run
```

Contribuições devem manter a documentação sincronizada e explicar qualquer novo bloco `unsafe`. Consulte [CONTRIBUTING.md](CONTRIBUTING.md) e [SECURITY.md](SECURITY.md).

## Princípios

- Safe Rust por padrão; `unsafe` concentrado em `arch`, `hal` e integração de boot.
- Separação entre kernel, HAL, arquitetura e ferramentas.
- Sem autoridade remota, chave mestra do projeto ou telemetria obrigatória.
- Novos módulos só entram quando houver código real e um problema concreto para resolver.
- Toda mudança relevante deve atualizar a documentação e registrar sua verificação.

## Licença

A licença definitiva ainda será escolhida antes do primeiro release público. Até lá, o repositório deve ser tratado como código experimental sem autorização implícita para redistribuição comercial.
