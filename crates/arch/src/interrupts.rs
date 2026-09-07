use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
const PIT_FREQUENCY_HZ: u32 = 1_193_182;
pub const TIMER_FREQUENCY_HZ: u32 = 100;

static TICKS: AtomicU64 = AtomicU64::new(0);

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_handler);
        idt
    };
    static ref PICS: Mutex<ChainedPics> =
        Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndex {
    const fn as_u8(self) -> u8 {
        self as u8
    }

    const fn as_usize(self) -> usize {
        self.as_u8() as usize
    }
}

pub fn init() {
    IDT.load();
    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();
        // Enable timer IRQ0 and keyboard IRQ1. The timer handler only records
        // ticks; scheduler policy remains outside interrupt context.
        pics.write_masks(0b1111_1100, 0xff);
    }
    configure_pit(TIMER_FREQUENCY_HZ);
    x86_64::instructions::interrupts::enable();
}

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub const fn pit_divisor(frequency_hz: u32) -> u16 {
    let divisor = PIT_FREQUENCY_HZ / frequency_hz;
    if divisor > u16::MAX as u32 {
        u16::MAX
    } else {
        divisor as u16
    }
}

fn configure_pit(frequency_hz: u32) {
    let divisor = pit_divisor(frequency_hz);
    unsafe {
        port_write(0x43, 0x36);
        port_write(0x40, (divisor & 0xff) as u8);
        port_write(0x40, (divisor >> 8) as u8);
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::serial::write_byte(b'!');
    let _ = stack_frame;
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    let _ = stack_frame;
    loop {
        core::hint::spin_loop();
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: PageFaultErrorCode,
) {
    let _ = stack_frame;
    loop {
        core::hint::spin_loop();
    }
}

extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    TICKS.fetch_add(1, Ordering::Relaxed);
    unsafe { send_end_of_interrupt(InterruptIndex::Timer.as_u8()) };
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    crate::keyboard::handle_interrupt();
    unsafe { send_end_of_interrupt(InterruptIndex::Keyboard.as_u8()) };
}

unsafe fn send_end_of_interrupt(interrupt_id: u8) {
    if interrupt_id >= PIC_2_OFFSET {
        unsafe { port_write(0xa0, 0x20) };
    }
    unsafe { port_write(0x20, 0x20) };
}

unsafe fn port_write(port: u16, value: u8) {
    unsafe { core::arch::asm!("out dx, al", in("dx") port, in("al") value) };
}
