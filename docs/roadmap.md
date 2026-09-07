# Roadmap

Cada etapa precisa compilar e ser observável antes da próxima começar. O roadmap é uma direção técnica; se uma etapa revelar uma mudança arquitetural necessária, a decisão deve ser registrada em `progress.md` e `architecture.md` antes da implementação.

## Concluído

- [x] `v0.0.1` — boot UEFI, kernel Rust, serial e framebuffer.
- [x] `v0.0.2` — GDT/TSS, IDT, exceções, PIC e PIT.
- [x] `v0.0.3` — mapa de memória física e allocator de frames de 4 KiB.
- [x] `v0.0.4` — heap inicial do kernel de 64 KiB.
- [x] `v0.0.5` — teclado PS/2 básico via IRQ1.

## Próximas etapas

- [ ] `v0.0.6` — estrutura de tarefas sem scheduler preemptivo completo.
- [ ] `v0.0.7` — scheduler inicial usando os ticks do PIT.
- [ ] `v0.0.8` — transição controlada para ring 3.
- [ ] `v0.0.9` — ABI e syscalls mínimas, em crate independente.
- [ ] `v0.1.0` — init e shell userspace mínimos.

## Fora do escopo imediato

GUI completa, rede, USB, áudio, filesystem complexo, criptografia de armazenamento e Secure Boot do proprietário não serão adicionados antes de memória, tarefas, isolamento e syscalls terem contratos estáveis.

## Critério de avanço

Uma versão só avança quando:

- o código compila para o target correto;
- fmt e clippy aplicáveis passam;
- testes automatizados relevantes passam;
- o comportamento previsto é observado no QEMU quando aplicável;
- a documentação registra arquivos, decisões, limitações e evidências.
