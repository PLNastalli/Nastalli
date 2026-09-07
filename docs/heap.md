# Heap do kernel — v0.0.4

## Escopo

A v0.0.4 adiciona uma heap inicial de 64 KiB para permitir as primeiras alocações do kernel. A área é uma região estática alinhada a 8 bytes e já pertence à imagem do kernel; por isso esta etapa não precisa alterar paginação.

## Contrato

`kernel::heap::init()` deve ser chamado uma única vez durante a inicialização do kernel, depois de a memória fornecida pelo bootloader estar disponível. O allocator global é um `LockedHeap`, permitindo uso futuro por código que importe `alloc`.

```rust
heap::init();
```

## Segurança

A única operação unsafe é a entrega do endereço da área estática ao allocator. O intervalo e o tamanho são constantes do kernel (`SIZE = 64 KiB`), e a inicialização não aceita ponteiros arbitrários externos.

## Limitações

- A heap não cresce.
- Não há heap por processo.
- Não há mapeamento de páginas sob demanda.
- A capacidade é pequena e não deve ser usada como solução definitiva.
- A política de desalocação pertence à crate `linked_list_allocator` nesta primeira versão.

Quando o kernel tiver paginação própria, a heap deverá ser migrada para páginas obtidas pelo `FrameAllocator`, com limites e accounting explícitos.

## Verificação

Os testes cobrem o alinhamento de endereços. O boot no QEMU confirma a inicialização da heap sem alterar o fluxo de memória física.
