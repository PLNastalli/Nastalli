#[cfg(not(test))]
use linked_list_allocator::LockedHeap;

pub const SIZE: usize = 64 * 1024;

#[repr(align(8))]
#[cfg(not(test))]
struct AlignedHeap([u8; SIZE]);

#[cfg(not(test))]
static mut HEAP: AlignedHeap = AlignedHeap([0; SIZE]);

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[cfg(not(test))]
pub fn init() {
    let heap_start = unsafe { align_up(core::ptr::addr_of_mut!(HEAP.0) as usize, 8) as *mut u8 };
    unsafe {
        ALLOCATOR.lock().init(heap_start, SIZE);
    }
}

#[cfg(test)]
pub fn init() {}

const fn align_up(address: usize, alignment: usize) -> usize {
    (address + alignment - 1) & !(alignment - 1)
}

#[cfg(test)]
mod tests {
    #[test]
    fn heap_alignment_rounds_up_to_eight_bytes() {
        assert_eq!(super::align_up(9, 8), 16);
    }
}
