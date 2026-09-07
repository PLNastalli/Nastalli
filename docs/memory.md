# Memória física — v0.0.3

## Escopo

A v0.0.3 introduz o primeiro componente de gerenciamento de memória: um allocator linear de frames físicos de 4 KiB. Ele não é ainda um allocator de heap, não configura paginação e não modifica as tabelas criadas pelo bootloader.

## Fonte de autoridade

O bootloader fornece `BootInfo.memory_regions`. Cada região tem endereço inicial inclusivo, endereço final exclusivo e um `MemoryRegionKind`.

Somente regiões `Usable` podem ser entregues ao kernel. Regiões `Bootloader`, `UnknownUefi` e `UnknownBios` permanecem reservadas.

## Invariantes

- Todo frame começa em endereço múltiplo de `4096`.
- O fim da região é tratado como exclusivo.
- Bordas desalinhadas são descartadas, nunca arredondadas para dentro de uma região reservada.
- Nenhum frame é retornado duas vezes pelo mesmo `FrameAllocator`.
- Uma região não utilizável nunca produz um frame.
- A ausência de frames retorna `None`; não há panic por exaustão.

## Implementação atual

`kernel::memory::FrameAllocator` mantém uma referência ao slice do bootloader, o índice da região atual e o próximo endereço. O custo de estado é constante; a busca atravessa regiões em ordem e não requer heap ou bitmap.

```rust
let mut allocator = FrameAllocator::new(&boot_info.memory_regions);
let frame = allocator.allocate_frame();
```

O tipo `PhysicalFrame` contém somente o endereço físico inicial. Ele ainda não é convertido em `x86_64::PhysFrame`, preservando a independência do kernel em relação a uma API específica de arquitetura.

## Limitações

- O kernel apenas conta frames durante a inicialização nesta versão.
- Ainda não há reserva explícita das páginas ocupadas pelo próprio kernel.
- Ainda não há bitmap persistente, desalocação, sincronização entre CPUs ou accounting por processo.
- Essas extensões só devem entrar quando houver um consumidor real, como paginação ou heap.

## Verificação

Os testes host cobrem alinhamento de bordas e rejeição de regiões do bootloader. O QEMU deve imprimir a quantidade de frames utilizáveis e continuar inicializando sem modificar a paginação fornecida pelo bootloader.
