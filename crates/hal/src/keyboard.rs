#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    Enter,
    Space,
    Backspace,
}

pub fn decode_scancode(scancode: u8) -> Option<Key> {
    if scancode & 0x80 != 0 {
        return None;
    }
    match scancode {
        0x1e => Some(Key::A),
        0x30 => Some(Key::B),
        0x2e => Some(Key::C),
        0x20 => Some(Key::D),
        0x12 => Some(Key::E),
        0x1c => Some(Key::Enter),
        0x39 => Some(Key::Space),
        0x0e => Some(Key::Backspace),
        _ => None,
    }
}

pub fn take_key() -> Option<Key> {
    nastalli_arch::keyboard::take_scancode().and_then(decode_scancode)
}

#[cfg(test)]
mod tests {
    #[test]
    fn decodes_pressed_a() {
        assert_eq!(super::decode_scancode(0x1e), Some(super::Key::A));
    }

    #[test]
    fn ignores_key_release_scancodes() {
        assert_eq!(super::decode_scancode(0x9e), None);
    }
}
