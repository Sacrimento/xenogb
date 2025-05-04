use std::cell::Cell;

thread_local! {
    pub static INTERRUPT_FLAGS: Cell<u8> = const { Cell::new(0) };
    pub static INTERRUPT_ENABLE: Cell<u8> = const { Cell::new(0) };
}

#[allow(nonstandard_style)]
pub mod InterruptFlags {
    pub const VBLANK: u8 = 0x1;
    pub const STAT: u8 = 0x2;
    pub const TIMER: u8 = 0x4;
    pub const SERIAL: u8 = 0x8;
    pub const JOYPAD: u8 = 0x10;
}

pub fn request_interrupt(int: u8) {
    INTERRUPT_FLAGS.set(INTERRUPT_FLAGS.get() | int);
}
