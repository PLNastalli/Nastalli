# Physical Memory — v0.0.3

## Scope

`v0.0.3` introduced Nastalli's first physical-memory management component: a linear allocator model for 4 KiB physical frames.

It is **not** the kernel heap, does not configure page tables, and does not yet replace or rebuild the mappings established by the bootloader.

## Source of authority

The bootloader provides `BootInfo.memory_regions`. Each memory region has:

- an inclusive start address;
- an exclusive end address;
- a `MemoryRegionKind`.

Only regions marked `Usable` are eligible to produce frames for the allocator.

Regions such as `Bootloader`, `UnknownUefi`, and `UnknownBios` remain unavailable to the allocator.

## Invariants

The current allocator is designed around these invariants:

- every returned frame begins at an address aligned to `4096` bytes;
- region ends are treated as exclusive;
- misaligned boundaries are discarded conservatively rather than rounded into reserved memory;
- one `FrameAllocator` instance does not return the same frame twice;
- non-usable regions never produce allocatable frames;
- exhaustion returns `None` rather than panicking.

These invariants are more important than the current internal representation and should remain covered as the allocator evolves.

## Current implementation

`kernel::memory::FrameAllocator` holds a reference to the bootloader-provided memory-region slice, tracks the current region, and advances through eligible frame addresses.

```rust
let mut allocator = FrameAllocator::new(&boot_info.memory_regions);
let frame = allocator.allocate_frame();
```

`PhysicalFrame` currently contains only the physical start address. It is intentionally not exposed as an `x86_64::PhysFrame`, keeping the generic kernel model from depending directly on an architecture-specific library type.

## Current usage

In the current boot flow, the kernel primarily uses the allocator model to inspect/count usable physical frames and validate the memory-management foundation.

The project does not yet rely on this allocator to manage a complete dynamic paging system.

## Current limitations

As of `v0.0.6`:

- there is no complete self-managed virtual-memory subsystem;
- there is no frame deallocation API;
- there is no persistent bitmap/buddy allocator;
- there is no per-process memory accounting;
- there is no SMP synchronization for frame allocation;
- kernel/user address-space ownership does not yet exist;
- explicit reservations and lifecycle rules will need to become stronger when Nastalli begins controlling page mappings itself.

The allocator should evolve in response to real paging, process, heap-growth, and DMA consumers rather than being generalized prematurely.

## Future direction

Later memory milestones are expected to add:

- virtual address-space management;
- mapping/unmapping APIs;
- page permissions;
- process-owned mappings;
- frame reclamation;
- synchronization suitable for multicore systems;
- explicit kernel/device/DMA ownership rules;
- leak and lifetime validation.

These are roadmap targets, not current features.

## Verification

Host tests cover boundary alignment and rejection of non-usable/bootloader regions. QEMU validation checks that the kernel reports usable physical-frame information and continues booting without replacing the bootloader-provided paging setup.

See [`progress.md`](progress.md) for recorded version evidence.
