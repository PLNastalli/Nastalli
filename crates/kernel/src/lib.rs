#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

use bootloader_api::BootInfo;
use core::fmt::Write;

pub mod heap;
pub mod memory;
pub mod task;

pub fn start(boot_info: &'static mut BootInfo) -> ! {
    let mut serial = nastalli_hal::serial::Serial::init();
    let _ = writeln!(serial, "NASTALLI OS v0.0.6");
    let _ = writeln!(serial, "Architecture: {}", nastalli_arch::NAME);
    let _ = writeln!(serial, "Boot: UEFI");
    let _ = writeln!(serial, "Kernel initialized successfully.");

    nastalli_arch::gdt::init();
    nastalli_arch::interrupts::init();
    let _ = writeln!(serial, "IDT and keyboard IRQ1 initialized.");

    heap::init();
    let _ = writeln!(
        serial,
        "Kernel heap initialized: {} KiB.",
        heap::SIZE / 1024
    );
    let _ = writeln!(serial, "Keyboard input initialized on IRQ1.");

    let usable_frames = {
        let allocator = memory::FrameAllocator::new(&boot_info.memory_regions);
        allocator.total_usable_frames()
    };
    let _ = writeln!(serial, "Physical memory: {usable_frames} usable frames.");

    let mut tasks = task::TaskTable::new();
    let bootstrap_task = tasks.create().expect("bootstrap task slot");
    tasks
        .set_state(bootstrap_task, task::TaskState::Running)
        .expect("bootstrap task exists");
    let _ = writeln!(serial, "Task table initialized: {} task.", tasks.len());

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();
        let buffer = framebuffer.buffer_mut();
        let width = info.width.min(320);
        let height = info.height.min(80);
        for y in 0..height {
            for x in 0..width {
                let byte_offset = y * info.stride * info.bytes_per_pixel + x * info.bytes_per_pixel;
                if byte_offset + 3 <= buffer.len() {
                    buffer[byte_offset] = 0x20;
                    buffer[byte_offset + 1] = 0x10;
                    buffer[byte_offset + 2] = 0x40;
                }
            }
        }
    }

    loop {
        if let Some(key) = nastalli_hal::keyboard::take_key() {
            let _ = writeln!(serial, "Key pressed: {key:?}");
        }
        core::hint::spin_loop();
    }
}

pub fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut serial = nastalli_hal::serial::Serial::init();
    let _ = writeln!(serial, "kernel panic: {info}");
    loop {
        core::hint::spin_loop();
    }
}
