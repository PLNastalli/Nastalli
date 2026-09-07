# Política de `unsafe`

## Regra geral

Safe Rust é o padrão. `unsafe` só pode existir quando representa uma fronteira real com hardware, ABI, memória fornecida pelo bootloader ou uma garantia que o compilador não consegue expressar.

Todos os crates usam:

```rust
#![deny(unsafe_op_in_unsafe_fn)]
```

## Localização atual

### `crates/arch`

Contém os únicos acessos atuais a `asm!`, portas x86 I/O, carregamento de GDT/TSS, habilitação de interrupções e inicialização do PIC. Cada função unsafe deve documentar a pré-condição do hardware.

### `crates/hal`

Expõe APIs seguras para o restante do sistema. O escritor serial não expõe portas ou assembly ao kernel.

### `crates/boot`

Usa a macro de entry point do bootloader e fornece o panic handler do binário. Não deve conter lógica de hardware.

### `crates/kernel`

Não deve usar `unsafe` diretamente para hardware. Quando precisar de uma operação privilegiada, deve ampliar primeiro a API do HAL ou de `arch`.

## Justificativa dos blocos atuais

- `arch::serial`: `out`/`in` acessam as portas COM1, assumindo execução em x86_64 com a UART disponível.
- `arch::gdt`: `CS::set_reg` e `load_tss` alteram registradores privilegiados após a GDT estática ter sido carregada.
- `arch::interrupts`: configuração do PIC/PIT acessa portas I/O e notifica o controlador após uma IRQ.
- `bootloader_api::Framebuffer::buffer_mut`: a segurança da região de memória é garantida pelo `BootInfo` fornecido pelo bootloader; a API encapsula os ponteiros crus.

## Proibições

- Não usar `unsafe` para contornar borrow checker.
- Não espalhar `asm!` pelo kernel.
- Não expor ponteiros crus como API pública quando uma referência segura for possível.
- Não aceitar `unsafe` sem comentário explicando a garantia necessária.
