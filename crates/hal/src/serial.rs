use core::fmt;

pub struct Serial;

impl Serial {
    pub fn init() -> Self {
        novaos_arch::serial::init();
        Self
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            if byte == b'\n' {
                novaos_arch::serial::write_byte(b'\r');
            }
            novaos_arch::serial::write_byte(byte);
        }
        Ok(())
    }
}
