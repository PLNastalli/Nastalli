use core::sync::atomic::{AtomicU8, Ordering};

static LAST_SCANCODE: AtomicU8 = AtomicU8::new(0);

pub(crate) fn handle_interrupt() {
    let scancode = unsafe { read_port(0x60) };
    LAST_SCANCODE.store(scancode, Ordering::Release);
}

pub fn take_scancode() -> Option<u8> {
    match LAST_SCANCODE.swap(0, Ordering::Acquire) {
        0 => None,
        scancode => Some(scancode),
    }
}

unsafe fn read_port(port: u16) -> u8 {
    let value: u8;
    unsafe { core::arch::asm!("in al, dx", out("al") value, in("dx") port) };
    value
}
