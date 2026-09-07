# Kernel Heap — v0.0.4

## Scope

`v0.0.4` introduced Nastalli's first kernel heap: a static 64 KiB region used to enable initial dynamic allocations in kernel code.

The region is aligned to 8 bytes and is part of the kernel image, so this milestone did not require Nastalli to take control of page tables or implement dynamic virtual-memory growth.

## Current contract

`kernel::heap::init()` is called once during kernel initialization.

```rust
heap::init();
```

The global allocator is based on `linked_list_allocator::LockedHeap`, allowing code that uses `alloc` to allocate from the initial kernel heap after initialization.

## Safety boundary

The unsafe operation in this path provides the allocator with the address and size of the kernel-owned static region.

The validity argument is intentionally narrow:

- the storage is statically allocated by the kernel;
- its size is fixed by the kernel (`64 KiB`);
- the initialization path does not accept an arbitrary external pointer;
- initialization is expected to occur once during boot.

Future changes must preserve or explicitly revise these invariants.

## Current limitations

- the heap does not grow;
- there is no per-process heap;
- there is no demand paging;
- the kernel does not yet obtain heap growth pages from its physical frame allocator;
- 64 KiB is a bring-up capacity, not a production design;
- free-list behavior is delegated to `linked_list_allocator` in the current implementation.

## Long-term direction

Once Nastalli controls virtual memory and page mappings, kernel heap growth should be backed by explicitly owned frames and mapped virtual pages, with clear accounting, exhaustion behavior, and synchronization rules.

That future design should not be implemented merely to replace the static heap before there is a real consumer that needs additional capacity.

## Verification

Host tests cover the relevant alignment rule. QEMU boot validation confirms that heap initialization integrates with the rest of kernel startup without breaking the existing physical-memory path.

See [`progress.md`](progress.md) for version-specific validation evidence.
