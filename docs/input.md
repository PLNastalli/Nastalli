# Input de teclado — v0.0.5

## Escopo

A v0.0.5 adiciona entrada mínima de teclado PS/2, suficiente para QEMU. O PIC entrega IRQ1 ao handler de teclado; o acesso à porta fica em `arch` e a decodificação de scancodes fica no HAL. O PIT permanece configurado, mas sua IRQ fica mascarada até a etapa de tarefas.

## Fluxo

```text
Teclado PS/2 → IRQ1 → arch::keyboard::handle_interrupt()
             → scancode atômico → hal::keyboard::take_key()
             → Key → kernel registra na serial
```

O buffer atual guarda somente o último scancode. Isso não é ainda uma fila de eventos; uma fila será criada quando tarefas e consumidores concorrentes existirem.

## Teclas suportadas

Scancodes Set 1 para `A`, `B`, `C`, `D`, `E`, `Enter`, `Space` e `Backspace`. Scancodes de liberação e teclas não mapeadas são ignorados.

## Segurança e limites

O handler não faz alocação nem formatação: lê a porta `0x60`, grava um byte atômico e envia EOI ao PIC. A conversão para `Key` ocorre fora do contexto da interrupção.

Ainda não há suporte a USB, layouts internacionais, Shift/Ctrl/Alt, repetição, fila, mouse ou input de userspace.
