use crate::core::cpu::cpu::LR35902CPU;

static mut SERIAL_BUFF: [u8; 0x100] = [0; 0x100];
static mut SERIAL_IDX: u8 = 0;

pub fn print_serial(cpu: &mut LR35902CPU) {
    let char = cpu.bus.io.serial.get_char();

    if char != 0 {
        unsafe {
            SERIAL_BUFF[SERIAL_IDX as usize] = char;
            SERIAL_IDX = SERIAL_IDX.wrapping_add(1);
            println!(
                "SERIAL DATA: {}",
                std::str::from_utf8(&SERIAL_BUFF[..SERIAL_IDX as usize])
                    .expect("invalid utf-8 sequence")
            );
        }
    }
}
