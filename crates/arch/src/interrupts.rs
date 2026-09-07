use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::PrivilegeLevel;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
const PIT_FREQUENCY_HZ: u32 = 1_193_182;
pub const TIMER_FREQUENCY_HZ: u32 = 100;

static TICKS: AtomicU64 = AtomicU64::new(0);

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint
            .set_handler_fn(breakpoint_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_handler);
        idt[nastalli_abi::SYSCALL_VECTOR as usize]
            .set_handler_fn(syscall_handler)
            .set_privilege_level(PrivilegeLevel::Ring3);
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
    trace(b"Interrupt init: IDT start.\r\n");
    IDT.load();
    trace(b"Interrupt init: IDT complete.\r\n");

    trace(b"Interrupt init: PIC start.\r\n");
    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();
        pics.write_masks(0b1111_1100, 0xff);
    }
    trace(b"Interrupt init: PIC complete.\r\n");

    trace(b"Interrupt init: PIT start.\r\n");
    configure_pit(TIMER_FREQUENCY_HZ);
    trace(b"Interrupt init: PIT complete.\r\n");

    trace(b"Interrupt init: enable start.\r\n");
    x86_64::instructions::interrupts::enable();
    trace(b"Interrupt init: enable complete.\r\n");
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

fn trace(message: &[u8]) {
    for &byte in message {
        crate::serial::write_byte(byte);
    }
}

fn trace_hex_u64(value: u64) {
    for shift in (0..16).rev() {
        let nibble = ((value >> (shift * 4)) & 0x0f) as u8;
        crate::serial::write_byte(if nibble < 10 {
            b'0' + nibble
        } else {
            b'a' + nibble - 10
        });
    }
}

fn fault(marker: &[u8]) -> ! {
    trace(marker);
    loop {
        core::hint::spin_loop();
    }
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    trace(b"Ring 3 probe reached kernel breakpoint.\r\n");
    loop {
        core::hint::spin_loop();
    }
}

extern "x86-interrupt" fn syscall_handler(_stack_frame: InterruptStackFrame) {
    trace(b"Ring 3 syscall entered kernel and returned.\r\n");
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    let _ = stack_frame;
    fault(b"FAULT: UD\r\n");
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    let _ = stack_frame;
    fault(b"FAULT: DF\r\n");
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
    let _ = stack_frame;
    fault(b"FAULT: TS\r\n");
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    let _ = stack_frame;
    fault(b"FAULT: NP\r\n");
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    let _ = stack_frame;
    fault(b"FAULT: SS\r\n");
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    trace(b"FAULT: GP error=0x");
    trace_hex_u64(error_code);
    trace(b" rip=0x");
    trace_hex_u64(stack_frame.instruction_pointer.as_u64());
    trace(b"\r\n");
    loop {
        core::hint::spin_loop();
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: PageFaultErrorCode,
) {
    let _ = stack_frame;
    fault(b"FAULT: PF\r\n");
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
