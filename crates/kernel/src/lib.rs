#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

use bootloader_api::BootInfo;
use core::fmt::Write;

pub mod heap;
pub mod memory;
pub mod scheduler;
pub mod task;

const USER_CODE_ADDRESS: u64 = 0x0000_0000_4000_0000;
const USER_STACK_ADDRESS: u64 = 0x0000_0000_4001_0000;

struct UserProbe {
    entry: u64,
    stack_top: u64,
}

pub fn start(boot_info: &'static mut BootInfo) -> ! {
    let mut serial = nastalli_hal::serial::Serial::init();
    write_banner(&mut serial);
    initialize_platform(&mut serial);
    initialize_memory(&mut serial, boot_info);
    let tasks = initialize_tasks(&mut serial);
    paint_framebuffer(boot_info);

    let physical_memory_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("physical memory mapping enabled by boot configuration");
    let mut allocator = memory::FrameAllocator::new(&boot_info.memory_regions);
    run(&mut serial, tasks, physical_memory_offset, &mut allocator);
}

fn write_banner(serial: &mut impl Write) {
    let _ = writeln!(serial, "NASTALLI OS v0.0.9");
    let _ = writeln!(serial, "Architecture: {}", nastalli_arch::NAME);
    let _ = writeln!(serial, "Boot: UEFI");
    let _ = writeln!(serial, "Kernel initialized successfully.");
}

fn initialize_platform(serial: &mut impl Write) {
    let _ = writeln!(serial, "Platform init: GDT start.");
    nastalli_arch::gdt::init();
    let _ = writeln!(serial, "Platform init: GDT complete.");
    let _ = writeln!(serial, "Platform init: interrupts start.");
    nastalli_arch::interrupts::init();
    let _ = writeln!(serial, "Platform init: interrupts complete.");
    let _ = writeln!(serial, "IDT, timer IRQ0 and keyboard IRQ1 initialized.");

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

fn prepare_scheduler_runtime(
    mut tasks: task::TaskTable,
    physical_memory_offset: u64,
    allocator: &mut memory::FrameAllocator<'_>,
) -> *mut scheduler::Scheduler {
    let scheduler_frame = allocator
        .allocate_frame()
        .expect("scheduler runtime frame");
    let worker_frame = allocator
        .allocate_frame()
        .expect("task worker stack frame");

    unsafe {
        nastalli_arch::paging::zero_frame(physical_memory_offset, scheduler_frame.start_address);
        nastalli_arch::paging::zero_frame(physical_memory_offset, worker_frame.start_address);
    }

    let scheduler_ptr = physical_memory_offset
        .checked_add(scheduler_frame.start_address)
        .expect("scheduler virtual address overflow")
        as *mut scheduler::Scheduler;
    let worker_stack = physical_memory_offset
        .checked_add(worker_frame.start_address)
        .expect("worker stack virtual address overflow")
        as *mut u8;

    let worker_task = tasks.create().expect("worker task slot");
    let worker_context = unsafe {
        nastalli_arch::context::prepare(
            worker_stack,
            memory::PAGE_SIZE as usize,
            task_switch_worker,
            scheduler_ptr.cast(),
        )
    };
    tasks
        .install_execution(
            worker_task,
            worker_context,
            task::KernelStack::new(worker_stack as u64, memory::PAGE_SIZE as usize),
        )
        .expect("worker task exists");

    unsafe {
        scheduler_ptr.write(scheduler::Scheduler::new(tasks));
    }
    scheduler_ptr
}

fn wait_for_prepared_switch(
    scheduler_ptr: *mut scheduler::Scheduler,
) -> scheduler::PreparedContextSwitch {
    let mut observed_ticks = nastalli_arch::interrupts::ticks();

    loop {
        let now = nastalli_arch::interrupts::ticks();
        while observed_ticks < now {
            observed_ticks = observed_ticks.saturating_add(1);
            let decision = unsafe { (&mut *scheduler_ptr).on_tick() };
            if matches!(decision, scheduler::ScheduleDecision::Switch { .. }) {
                return unsafe {
                    (&mut *scheduler_ptr)
                        .prepare_context_switch(decision)
                        .expect("scheduled tasks own contexts")
                };
            }
        }
        core::hint::spin_loop();
    }
}

unsafe fn execute_prepared_switch(prepared: scheduler::PreparedContextSwitch) {
    let current = unsafe { &mut *prepared.current };
    unsafe { nastalli_arch::context::switch(current, &prepared.next) };
}

fn run_task_switch_probe(serial: &mut impl Write, scheduler_ptr: *mut scheduler::Scheduler) {
    let first = wait_for_prepared_switch(scheduler_ptr);
    unsafe { execute_prepared_switch(first) };
    let _ = writeln!(serial, "Task switch: bootstrap resumed.");

    let second = wait_for_prepared_switch(scheduler_ptr);
    unsafe { execute_prepared_switch(second) };
    let _ = writeln!(serial, "Task switch: bootstrap resumed twice.");
}

extern "C" fn task_switch_worker(argument: *mut ()) -> ! {
    let scheduler_ptr = argument.cast::<scheduler::Scheduler>();
    let mut serial = nastalli_hal::serial::Serial::init();
    let _ = writeln!(serial, "Task switch: worker entered.");

    let first = wait_for_prepared_switch(scheduler_ptr);
    unsafe { execute_prepared_switch(first) };

    let _ = writeln!(serial, "Task switch: worker resumed.");
    let second = wait_for_prepared_switch(scheduler_ptr);
    unsafe { execute_prepared_switch(second) };

    loop {
        core::hint::spin_loop();
    }
}

fn prepare_ring3_probe(
    physical_memory_offset: u64,
    allocator: &mut memory::FrameAllocator<'_>,
) -> UserProbe {
    let code_frame = allocator.allocate_frame().expect("Ring 3 code frame");
    let stack_frame = allocator.allocate_frame().expect("Ring 3 stack frame");

    unsafe {
        nastalli_arch::paging::zero_frame(physical_memory_offset, code_frame.start_address);
        nastalli_arch::paging::zero_frame(physical_memory_offset, stack_frame.start_address);
        nastalli_arch::paging::write_frame_bytes(
            physical_memory_offset,
            code_frame.start_address,
            &[0xcd, nastalli_abi::SYSCALL_VECTOR, 0xcc],
        );
    }

    let mut allocate_page_table_frame =
        || allocator.allocate_frame().map(|frame| frame.start_address);

    unsafe {
        nastalli_arch::paging::map_user_stack_page(
            USER_STACK_ADDRESS,
            stack_frame.start_address,
            physical_memory_offset,
            &mut allocate_page_table_frame,
        )
        .expect("map Ring 3 stack page");
        nastalli_arch::paging::map_user_code_page(
            USER_CODE_ADDRESS,
            code_frame.start_address,
            physical_memory_offset,
            &mut allocate_page_table_frame,
        )
        .expect("map Ring 3 code page");
    }

    UserProbe {
        entry: USER_CODE_ADDRESS,
        stack_top: USER_STACK_ADDRESS + memory::PAGE_SIZE,
    }
}

fn run(
    serial: &mut impl Write,
    tasks: task::TaskTable,
    physical_memory_offset: u64,
    allocator: &mut memory::FrameAllocator<'_>,
) -> ! {
    let scheduler_ptr = prepare_scheduler_runtime(tasks, physical_memory_offset, allocator);
    let initial_ticks = nastalli_arch::interrupts::ticks();
    let _ = writeln!(
        serial,
        "Scheduler initialized: round-robin, {} tick quantum.",
        scheduler::DEFAULT_QUANTUM_TICKS
    );

    while nastalli_arch::interrupts::ticks() == initial_ticks {
        if let Some(key) = nastalli_hal::keyboard::take_key() {
            let _ = writeln!(serial, "Key pressed: {key:?}");
        }
        core::hint::spin_loop();
    }

    let _ = unsafe { (&mut *scheduler_ptr).on_tick() };
    let _ = writeln!(serial, "Scheduler timer active: first PIT tick observed.");

    run_task_switch_probe(serial, scheduler_ptr);
    let user_probe = prepare_ring3_probe(physical_memory_offset, allocator);
    unsafe { nastalli_arch::user::enter(user_probe.entry, user_probe.stack_top) }
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
