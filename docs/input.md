# Keyboard Input — v0.0.5

## Scope

`v0.0.5` introduced minimal PS/2 keyboard input sufficient for the current QEMU reference environment.

The PIC routes IRQ1 to the keyboard interrupt handler. Architecture-specific port access remains in `arch`, while scancode decoding is exposed through the HAL.

The PIT is configured as part of the interrupt foundation, but IRQ0 remains masked in the current `v0.0.6` runtime until scheduling work begins.

## Current flow

```text
PS/2 keyboard
    |
    v
IRQ1
    |
    v
arch::keyboard::handle_interrupt()
    |
    v
atomic scancode storage
    |
    v
hal::keyboard::take_key()
    |
    v
Key
    |
    v
kernel serial diagnostic
```

The current storage model retains only the most recent scancode. It is not yet an event queue and does not provide backpressure or multi-consumer semantics.

## Supported keys

The current Set 1 decoder recognizes:

- `A`
- `B`
- `C`
- `D`
- `E`
- `Enter`
- `Space`
- `Backspace`

Release scancodes and unsupported keys are ignored.

## Interrupt-handler constraints

The IRQ1 handler intentionally remains small. It should not perform allocation, formatting, complex decoding, or high-level policy work.

Its current responsibilities are limited to the hardware-facing path, including reading port `0x60`, recording the scancode, and completing interrupt-controller handling as required.

Higher-level conversion into `Key` occurs outside the interrupt context.

## Current limitations

As of `v0.0.6`, there is no support for:

- a queued input event model;
- USB keyboards;
- mouse input;
- international keyboard layouts;
- Shift/Ctrl/Alt modifier state;
- key repeat;
- full Set 1 coverage;
- userspace input delivery;
- device hotplug.

These should be added when the scheduler, userspace, and device model provide concrete consumers for them.

## Verification

Host tests verify a supported key press and that a release scancode is ignored. QEMU validation verifies the IRQ1 path and serial-visible decoded input behavior.

See [`progress.md`](progress.md) for the recorded version evidence.
