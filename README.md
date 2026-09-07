# Nastalli OS

Sistema operacional próprio, escrito principalmente em Rust, começando por x86_64 + UEFI + QEMU. O projeto não é uma distribuição Linux nem uma modificação de outro kernel.

## Estado atual

**Versão:** `v0.0.5`  
**Status:** boot validado em QEMU + OVMF  
**Próxima versão:** `v0.0.6`, estrutura inicial de tarefas

A versão atual entra no kernel Rust `no_std`, inicializa GDT/TSS, IDT, PIC 8259 e PIT a 100 Hz, captura teclado PS/2 pela IRQ1, conta frames físicos utilizáveis a partir do `BootInfo`, inicializa uma heap estática de 64 KiB, acessa o framebuffer e escreve diagnóstico pela serial COM1.

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

## Princípios

- Safe Rust por padrão; `unsafe` concentrado em `arch`, `hal` e integração de boot.
- Separação entre kernel, HAL, arquitetura e ferramentas.
- Sem autoridade remota, chave mestra do projeto ou telemetria obrigatória.
- Novos módulos só entram quando houver código real e um problema concreto para resolver.
- Toda mudança relevante deve atualizar a documentação e registrar sua verificação.
