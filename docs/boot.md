# Boot Flow

## Execution chain

```text
UEFI firmware
    |
    v
OVMF under QEMU
    |
    v
bootloader 0.11.10 UEFI image
    |
    v
entry point in crates/boot
    |
    v
nastalli_kernel::start(BootInfo)
    |
    +--> serial diagnostics
    +--> framebuffer access
    +--> GDT/TSS
    +--> IDT + PIC/PIT infrastructure
    +--> memory/heap/task initialization
```

Nastalli does not currently implement its own bootloader. The `boot` crate contains the integration with `bootloader_api`, the kernel binary entry point, and binary-level panic integration. Boot policy and ordinary hardware logic should remain outside this crate.

## Build

`cargo xtask build` performs a dedicated kernel build for:

```text
x86_64-unknown-none
```

The current build uses Rust `build-std` support for `core` and `compiler_builtins`. The host-side `xtask` binary itself is compiled for the development host and uses `std`.

The pinned toolchain is declared in `rust-toolchain.toml`. The current bootloader build also requires `rust-src` and `llvm-tools-preview`.

## Image creation

```bash
cargo xtask image
```

The `xtask` tool uses `bootloader::UefiBoot` to package the kernel ELF into:

```text
target/nastalli-uefi.img
```

The image format and boot chain currently depend on `bootloader 0.11.10`.

## Running under QEMU

```bash
cargo xtask run
```

`xtask` searches common OVMF locations, including:

- `/usr/share/edk2/x64/OVMF.4m.fd`
- `/usr/share/edk2/x64/OVMF_CODE.4m.fd`
- `/usr/share/edk2-ovmf/x64/OVMF_CODE.fd`
- `/usr/share/OVMF/OVMF_CODE.fd`

A custom OVMF code image can be selected with:

```bash
NASTALLI_OVMF_CODE=/path/to/OVMF_CODE.fd cargo xtask run
```

For serial-only/headless validation:

```bash
NASTALLI_QEMU_DISPLAY=none cargo xtask run
```

The default display mode is GTK when available through the current QEMU invocation.

## `BootInfo` contract

The bootloader provides `BootInfo`, which currently gives the kernel access to information including the memory map and framebuffer.

In the current kernel:

- the framebuffer is used directly for minimal boot-time drawing;
- `BootInfo.memory_regions` is inspected by the physical-memory allocator model;
- the kernel does not replace the bootloader-provided page tables with a full self-managed virtual-memory subsystem yet.

The lifetime and ownership assumptions of bootloader-provided structures must remain explicit whenever future memory-management code begins taking stronger control of address spaces.

## Current limitations

As of `v0.0.6`:

- there is no owner-controlled Secure Boot implementation;
- there is no initramfs or general boot filesystem;
- there is no userspace loader in the boot path;
- the UEFI image depends on the existing Rust bootloader crate and OVMF in the reference environment;
- the kernel does not return to firmware after taking control;
- real-hardware boot is not yet a production support claim.

## Verification

Boot-affecting changes should be validated with the applicable host checks plus a target run under QEMU/OVMF. Relevant serial output should be recorded in [`progress.md`](progress.md) when it supports a milestone claim.
