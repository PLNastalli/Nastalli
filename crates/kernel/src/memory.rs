use bootloader_api::info::{MemoryRegion, MemoryRegionKind};

pub const PAGE_SIZE: u64 = 4096;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PhysicalFrame {
    pub start_address: u64,
}

pub struct FrameAllocator<'a> {
    regions: &'a [MemoryRegion],
    region_index: usize,
    next_address: u64,
}

impl<'a> FrameAllocator<'a> {
    pub fn new(regions: &'a [MemoryRegion]) -> Self {
        Self {
            regions,
            region_index: 0,
            next_address: 0,
        }
    }

    pub fn total_usable_frames(&self) -> u64 {
        self.regions
            .iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable)
            .map(|region| usable_frame_count(region.start, region.end))
            .sum()
    }

    pub fn allocate_frame(&mut self) -> Option<PhysicalFrame> {
        while self.region_index < self.regions.len() {
            let region = self.regions[self.region_index];
            if region.kind != MemoryRegionKind::Usable {
                self.region_index += 1;
                self.next_address = 0;
                continue;
            }

            let first_frame = align_up(region.start);
            let last_address = align_down(region.end);
            let address = self.next_address.max(first_frame);
            if address < last_address {
                self.next_address = address + PAGE_SIZE;
                return Some(PhysicalFrame {
                    start_address: address,
                });
            }

            self.region_index += 1;
            self.next_address = 0;
        }

        None
    }
}

const fn align_up(address: u64) -> u64 {
    (address + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

const fn align_down(address: u64) -> u64 {
    address & !(PAGE_SIZE - 1)
}

const fn usable_frame_count(start: u64, end: u64) -> u64 {
    let first_frame = align_up(start);
    let last_address = align_down(end);
    if last_address <= first_frame {
        0
    } else {
        (last_address - first_frame) / PAGE_SIZE
    }
}

#[cfg(test)]
mod tests {
    use bootloader_api::info::{MemoryRegion, MemoryRegionKind};

    #[test]
    fn frame_count_ignores_unaligned_edges() {
        assert_eq!(super::usable_frame_count(0x1003, 0x4001), 2);
    }

    #[test]
    fn allocator_returns_only_usable_aligned_frames() {
        let regions = [
            MemoryRegion {
                start: 0x1003,
                end: 0x3001,
                kind: MemoryRegionKind::Usable,
            },
            MemoryRegion {
                start: 0x3001,
                end: 0x5000,
                kind: MemoryRegionKind::Bootloader,
            },
        ];
        let mut allocator = super::FrameAllocator::new(&regions);

        assert_eq!(allocator.allocate_frame().unwrap().start_address, 0x2000);
        assert_eq!(allocator.allocate_frame(), None);
    }
}
