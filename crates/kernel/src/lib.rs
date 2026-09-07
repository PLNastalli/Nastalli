#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

use bootloader_api::BootInfo;
use core::fmt::Write;

pub mod heap;
pub mod memory;
pub mod task;

pub fn start(boot_info: &'static mut BootInfo) -> ! {
    let mut serial = nastalli_hal::serial::Serial::init();
    write_banner(&mut serial);
    initialize_platform(&mut serial);
    initialize_memory(&mut serial, boot_info);
    let tasks = initialize_tasks(&mut serial);
    paint_framebuffer(boot_info);
    run(&mut serial, tasks);
}

fn write_banner(serial: &mut impl Write) {
    let _ = writeln!(serial, "NASTALLI OS v0.0.6");
    let _ = writeln!(serial, "Architecture: {}", nastalli_arch::NAME);
    let _ = writeln!(serial, "Boot: UEFI");
    let _ = writeln!(serial, "Kernel initialized successfully.");
}

fn initialize_platform(serial: &mut impl Write) {
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
}

fn initialize_memory(serial: &mut impl Write, boot_info: &mut BootInfo) {
    let usable_frames = {
        let allocator = memory::FrameAllocator::new(&boot_info.memory_regions);
        allocator.total_usable_frames()
    };
    let _ = writeln!(serial, "Physical memory: {usable_frames} usable frames.");
}

fn initialize_tasks(serial: &mut impl Write) -> task::TaskTable {
    let tasks = bootstrap_task_table();
    let _ = writeln!(serial, "Task table initialized: {} task.", tasks.len());
    tasks
}

fn bootstrap_task_table() -> task::TaskTable {
    let mut tasks = task::TaskTable::new();
    let bootstrap_task = tasks.create().expect("bootstrap task slot");
    tasks
        .set_state(bootstrap_task, task::TaskState::Running)
        .expect("bootstrap task exists");
    tasks
}

fn paint_framebuffer(boot_info: &mut BootInfo) {
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
}

fn run(serial: &mut impl Write, tasks: task::TaskTable) -> ! {
    // Keep ownership of the task table in the long-lived kernel runtime. The
    // scheduler introduced later can consume this same state instead of
    // reconstructing tasks after boot.
    let _tasks = tasks;

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

#[cfg(test)]
mod tests {
    #[test]
    fn bootstrap_task_table_contains_one_task() {
        let tasks = super::bootstrap_task_table();
        assert_eq!(tasks.len(), 1);
    }
}
