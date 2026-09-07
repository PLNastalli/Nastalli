# Histórico técnico

Este arquivo registra o que foi implementado, por que foi implementado e quais evidências existem. Cada versão deve ser concluída antes do início da seguinte.

## v0.0.6 — Estrutura de tarefas

### Objetivo

Criar um contrato mínimo para identidade e estado de tarefas sem iniciar ainda scheduler, troca de contexto ou userspace.

### Implementado

- `crates/kernel/src/task.rs` com `TaskId`, `TaskState`, `Task` e `TaskTable`.
- Tabela fixa de 16 tarefas, sem alocação dinâmica.
- IDs monotônicos e operações explícitas de criação, consulta e mudança de estado.
- Tarefa bootstrap criada durante a inicialização e marcada como `Running` apenas de forma descritiva.
- Testes para criação, transição de estado, capacidade e tarefas ausentes.
- Documentação dedicada em `docs/tasks.md`.

### Limites

- O PIT continua configurado, mas a IRQ0 permanece mascarada.
- Ainda não há scheduler, preempção, contexto salvo, stack própria ou processos.

### Verificação

- `cargo fmt --all -- --check`: aprovado.
- `cargo xtask test`: 1 teste de `arch`, 2 de `hal` e 6 do `kernel` passaram.
- `cargo check` dos crates aplicáveis: aprovado.
- `cargo clippy ... -- -D warnings`: aprovado.
- `cargo xtask build`: kernel x86_64 compilado.
- `cargo xtask run` com QEMU + OVMF: boot aprovado.
- Saída observada: `NASTALLI OS v0.0.6`, `Physical memory: 30269 usable frames.` e `Task table initialized: 1 task.`
- O QEMU foi encerrado por timeout controlado após a validação, pois o kernel permanece em loop infinito.

## v0.0.5 — Teclado PS/2

### Objetivo

Capturar teclas básicas no QEMU por IRQ1, mantendo o handler curto e separando leitura de hardware da decodificação.

### Implementado

- `arch::keyboard` com leitura da porta `0x60` e armazenamento atômico do último scancode.
- Handler de teclado no vetor IRQ1 após remapeamento do PIC.
- IRQ1 (teclado) desbloqueada no PIC; IRQ0 permanece mascarada até o scheduler.
- `hal::keyboard` com decodificação de scancodes Set 1 para oito teclas básicas.
- Kernel registra teclas decodificadas pela serial.
- Testes de tecla pressionada e liberação ignorada.
- Documentação dedicada em `docs/input.md`.

### Decisões

- O contexto da interrupção não aloca, formata nem chama código de alto nível.
- O buffer de um único evento é suficiente para validar o hardware; fila e backpressure ficam para quando houver tarefas.
- USB e layouts complexos não entram nesta versão.

### Verificação

- Testes do HAL: 2 testes de scancode passaram.
- `cargo check` dos crates aplicáveis: aprovado.
- `cargo clippy` com `-D warnings`: aprovado.
- `cargo xtask build`: kernel x86_64 compilado.
- `cargo xtask run` no QEMU + OVMF: boot aprovado.
- Saída observada: `NASTALLI OS v0.0.5`, `IDT and keyboard IRQ1 initialized.`, `Keyboard input initialized on IRQ1.` e `Physical memory: 30296 usable frames.`
- O QEMU foi encerrado por timeout controlado após a validação, pois o kernel permanece em loop infinito.

## v0.0.4 — Heap do kernel

### Objetivo

Disponibilizar uma heap inicial, pequena e estável para futuras APIs que precisem de alocação, sem introduzir paginação dinâmica ou heap de userspace.

### Implementado

- `crates/kernel/src/heap.rs` com heap estática alinhada de 64 KiB.
- `linked_list_allocator::LockedHeap` como allocator global.
- Inicialização explícita no fluxo do kernel.
- Teste unitário da regra de alinhamento.
- Documentação dedicada em `docs/heap.md`.

### Decisões

- A heap usa memória estática para não depender de paginação própria nesta etapa.
- O `FrameAllocator` continua separado; ele será usado para crescimento da heap somente quando o kernel controlar suas próprias tabelas de páginas.
- Não há API de alocação customizada nem abstração de ownership prematura.

### Verificação

- Teste unitário da heap: passou.
- `cargo check` e `cargo clippy` dos crates aplicáveis: passaram.
- Build x86_64 e boot QEMU serão registrados após a validação final.

Validação final:

- `cargo xtask build`: kernel x86_64 compilado.
- `cargo xtask run` no QEMU + OVMF: boot aprovado.
- Saída observada: `NASTALLI OS v0.0.4`, `Kernel heap initialized: 64 KiB.` e `Physical memory: 30298 usable frames.`
- O QEMU foi encerrado por timeout controlado após a validação, pois o kernel permanece em loop infinito.

## v0.0.3 — Memória física

### Objetivo

Interpretar o mapa de memória do `BootInfo` e disponibilizar frames físicos alinhados de 4 KiB, sem ainda criar heap ou paginação própria.

### Implementado

- `crates/kernel/src/memory.rs` com `FrameAllocator` e `PhysicalFrame`.
- Filtragem exclusiva de `MemoryRegionKind::Usable`.
- Alinhamento seguro de início inclusivo e fim exclusivo.
- Contagem de frames utilizáveis na inicialização.
- Testes para bordas desalinhadas e regiões reservadas.
- Documentação dedicada em `docs/memory.md`.

### Verificação

- Testes unitários do kernel: 2 testes de memória passaram.
- `cargo check` dos crates aplicáveis: aprovado.
- `cargo clippy` com `-D warnings`: aprovado.
- `cargo xtask build`: kernel x86_64 compilado.
- `cargo xtask run` no QEMU + OVMF: boot aprovado; o kernel reportou `30340 usable frames`.

Saída relevante:

```text
NASTALLI OS v0.0.3
IDT, PIC and PIT initialized at 100 Hz.
Physical memory: 30340 usable frames.
```

O QEMU foi encerrado por timeout controlado após a validação, pois o kernel permanece em loop infinito.

## v0.0.2 — Exceções e interrupções

### Objetivo

Instalar a infraestrutura mínima de exceções x86_64 e interrupções de hardware sem introduzir scheduler, processos ou userspace.

### Implementado

- `crates/arch/src/gdt.rs`: GDT com segmento de código do kernel e TSS.
- `crates/arch/src/interrupts.rs`: IDT e handlers de breakpoint, page fault e double fault.
- PIC 8259 remapeado para os vetores 32–47.
- PIT programado para 100 Hz.
- Contador atômico de ticks, ainda sem consumidor de scheduling.
- API mínima `nastalli_arch::gdt::init()` e `nastalli_arch::interrupts::init()`.
- Teste do divisor PIT para 100 Hz.
- `xtask run` com descoberta de OVMF e modo headless via `NASTALLI_QEMU_DISPLAY`.

### Decisões

- O assembly continua restrito ao crate `arch`.
- O kernel não manipula portas, registradores ou instruções diretamente.
- O page fault e o double fault param a CPU após o diagnóstico; recuperação será tratada quando existir gerenciamento de memória e estado de tarefas.
- O timer apenas incrementa ticks; não há scheduler nesta versão.

### Verificação

Executado com nightly fixado em `nightly-2025-01-01`:

- `rustfmt --check`: aprovado.
- `cargo check` para `arch`, `hal`, `kernel` e `xtask`: aprovado.
- `cargo clippy ... -- -D warnings`: aprovado.
- `cargo xtask test`: 1 teste específico passou; demais crates sem testes ainda.
- `cargo xtask build`: kernel x86_64 compilado.
- `cargo xtask run` no QEMU + OVMF: boot aprovado e serial mostrou:

```text
NASTALLI OS v0.0.2
Architecture: x86_64
Boot: UEFI
Kernel initialized successfully.
IDT, PIC and PIT initialized at 100 Hz.
```

O QEMU foi encerrado por timeout controlado após a validação, pois o kernel permanece em loop infinito.

## v0.0.1 — Boot

### Objetivo

Entrar em um kernel Rust `no_std` por UEFI, escrever pela serial e acessar o framebuffer.

### Implementado

- Cargo workspace com cinco crates mínimos.
- `bootloader`/`bootloader_api` fixados em `0.11.10`.
- Target `x86_64-unknown-none` com `build-std` no comando de build.
- Entry point UEFI, `panic_handler`, serial COM1 e pintura inicial do framebuffer.
- `tools/xtask` com build, imagem, execução e testes.

### Resultado

A imagem UEFI foi criada e posteriormente validada no QEMU durante a implementação da v0.0.2.
