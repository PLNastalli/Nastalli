#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![feature(abi_x86_interrupt)]

pub const NAME: &str = "x86_64";

#[cfg(test)]
mod tests {
    #[test]
    fn pit_divisor_for_100_hz_is_11931() {
        assert_eq!(super::interrupts::pit_divisor(100), 11_931);
    }
}

pub mod serial {
    const COM1: u16 = 0x3f8;

    pub fn init() {
        unsafe {
            write(COM1 + 1, 0x00);
            write(COM1 + 3, 0x80);
            write(COM1, 0x03);
            write(COM1 + 1, 0x00);
            write(COM1 + 3, 0x03);
            write(COM1 + 2, 0xc7);
            write(COM1 + 4, 0x0b);
        }
    }

    pub fn write_byte(byte: u8) {
        unsafe {
            while read(COM1 + 5) & 0x20 == 0 {
                core::hint::spin_loop();
            }
            write(COM1, byte);
        }
    }

    unsafe fn write(port: u16, value: u8) {
        unsafe { core::arch::asm!("out dx, al", in("dx") port, in("al") value) };
    }

    unsafe fn read(port: u16) -> u8 {
        let value: u8;
        unsafe { core::arch::asm!("in al, dx", out("al") value, in("dx") port) };
        value
    }
}

pub mod context;
pub mod gdt;
pub mod interrupts;
pub mod keyboard;
pub mod paging;
pub mod preemption;
pub mod user;
