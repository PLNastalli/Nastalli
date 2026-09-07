use core::fmt;

pub struct Serial;

impl Serial {
    pub fn init() -> Self {
        nastalli_arch::serial::init();
        Self
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            if byte == b'\n' {
                nastalli_arch::serial::write_byte(b'\r');
            }
            nastalli_arch::serial::write_byte(byte);
        }
        Ok(())
    }
}
