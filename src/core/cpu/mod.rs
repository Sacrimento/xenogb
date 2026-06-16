mod clock;
pub mod cpu;
pub mod instructions;
pub mod interrupts;

pub use clock::{CPUSpeed, CLOCK_SPEED};
pub use cpu::LR35902CPU;
