# Fluxo de boot

## Cadeia de execução

```text
Firmware UEFI
    ↓
OVMF no QEMU
    ↓
Imagem criada por bootloader 0.11.10
    ↓
Entry point em crates/boot
    ↓
novaos_kernel::start(BootInfo)
    ↓
Serial, framebuffer, GDT/TSS, IDT, PIC e PIT
```

O Nova OS não possui bootloader próprio. O crate `boot` contém apenas a integração com `bootloader_api` e o ponto de entrada que entrega o `BootInfo` ao kernel.

## Build

`cargo xtask build` executa um build separado para `x86_64-unknown-none` e usa `-Zbuild-std=core,compiler_builtins`. O `xtask` em si continua sendo compilado para o host e usa `std`.

`cargo xtask image` usa `bootloader::UefiBoot` para empacotar o ELF do kernel em `target/novaos-uefi.img`.

## Execução

O `xtask` procura OVMF nestes caminhos:

- `/usr/share/edk2/x64/OVMF.4m.fd`;
- `/usr/share/edk2/x64/OVMF_CODE.4m.fd`;
- `/usr/share/edk2-ovmf/x64/OVMF_CODE.fd`;
- `/usr/share/OVMF/OVMF_CODE.fd`.

Também é possível definir `NOVAOS_OVMF_CODE`. O display padrão é GTK; para validar apenas a serial, use `NOVAOS_QEMU_DISPLAY=none`.

## Contrato BootInfo

O bootloader fornece memória, framebuffer e informações de carregamento. Na v0.0.3, o kernel usa o framebuffer diretamente e lê o mapa de memória para contar frames físicos utilizáveis.

## Limitações atuais

- Não há Secure Boot com chaves do proprietário ainda.
- Não há ramdisk, filesystem ou userspace.
- A imagem depende do bootloader existente e do firmware OVMF.
- O kernel não retorna ao firmware depois de assumir o controle.
