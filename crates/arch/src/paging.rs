use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

const PAGE_SIZE: usize = 4096;

struct CallbackFrameAllocator<'a, F> {
    allocate: &'a mut F,
}

unsafe impl<F> FrameAllocator<Size4KiB> for CallbackFrameAllocator<'_, F>
where
    F: FnMut() -> Option<u64>,
{
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        (self.allocate)().map(|address| PhysFrame::containing_address(PhysAddr::new(address)))
    }
}

unsafe fn active_mapper(physical_memory_offset: u64) -> OffsetPageTable<'static> {
    let physical_memory_offset = VirtAddr::new(physical_memory_offset);
    let (level_4_frame, _) = Cr3::read();
    let level_4_virtual = physical_memory_offset + level_4_frame.start_address().as_u64();
    let level_4_ptr: *mut PageTable = level_4_virtual.as_mut_ptr();
    let level_4_table = unsafe { &mut *level_4_ptr };
    unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset) }
}

pub unsafe fn zero_frame(physical_memory_offset: u64, physical_address: u64) {
    let pointer = (physical_memory_offset + physical_address) as *mut u8;
    unsafe { core::ptr::write_bytes(pointer, 0, PAGE_SIZE) };
}

pub unsafe fn write_frame_bytes(
    physical_memory_offset: u64,
    physical_address: u64,
    bytes: &[u8],
) {
    assert!(bytes.len() <= PAGE_SIZE);
    let pointer = (physical_memory_offset + physical_address) as *mut u8;
    unsafe { core::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer, bytes.len()) };
}

pub unsafe fn map_user_code_page<F>(
    virtual_address: u64,
    physical_address: u64,
    physical_memory_offset: u64,
    allocate_frame: &mut F,
) -> Result<(), ()>
where
    F: FnMut() -> Option<u64>,
{
    unsafe {
        map_page(
            virtual_address,
            physical_address,
            physical_memory_offset,
            PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE,
            allocate_frame,
        )
    }
}

pub unsafe fn map_user_stack_page<F>(
    virtual_address: u64,
    physical_address: u64,
    physical_memory_offset: u64,
    allocate_frame: &mut F,
) -> Result<(), ()>
where
    F: FnMut() -> Option<u64>,
{
    unsafe {
        map_page(
            virtual_address,
            physical_address,
            physical_memory_offset,
            PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE
                | PageTableFlags::NO_EXECUTE,
            allocate_frame,
        )
    }
}

unsafe fn map_page<F>(
    virtual_address: u64,
    physical_address: u64,
    physical_memory_offset: u64,
    flags: PageTableFlags,
    allocate_frame: &mut F,
) -> Result<(), ()>
where
    F: FnMut() -> Option<u64>,
{
    let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virtual_address));
    let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(physical_address));
    let parent_flags =
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
    let mut allocator = CallbackFrameAllocator {
        allocate: allocate_frame,
    };
    let mut mapper = unsafe { active_mapper(physical_memory_offset) };
    let flush = unsafe {
        mapper.map_to_with_table_flags(page, frame, flags, parent_flags, &mut allocator)
    }
    .map_err(|_| ())?;
    flush.flush();
    Ok(())
}
