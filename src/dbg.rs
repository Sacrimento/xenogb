use super::cpu::{CPUFlags, LR35902CPU};
use super::instructions::{AddrMode, CPURegister};

static mut serial_buff: [u8; 0x100] = [0; 0x100];
static mut serial_idx: usize = 0;

pub fn print_state(cpu: &LR35902CPU) {
    let header = format!("0x{:04X} {}", cpu.pc() - 1, cpu.current_instruction.name);

    let args = match cpu.current_instruction.addr_mode {
        AddrMode::R => {
            if cpu.current_instruction.reg1.is_some() {
                &format!("{}", cpu.current_instruction.reg1.as_ref().unwrap())
            } else {
                ""
            }
        }
        AddrMode::R_R => {
            if cpu.current_instruction.reg1.is_some() {
                &format!(
                    "{}, {}",
                    cpu.current_instruction.reg1.as_ref().unwrap(),
                    cpu.current_instruction.reg2.as_ref().unwrap()
                )
            } else {
                ""
            }
        }
        AddrMode::R_IMM => &format!(
            "{}, $0x{:02X}",
            cpu.current_instruction.reg1.as_ref().unwrap(),
            cpu.bus.read(cpu.pc())
        ),
        AddrMode::R_RADDR => &format!(
            "{}, [{}]",
            cpu.current_instruction.reg1.as_ref().unwrap(),
            cpu.current_instruction.reg2.as_ref().unwrap()
        ),
        AddrMode::R_IMMADDR => &format!(
            "{}, [0x{:04X}]",
            cpu.current_instruction.reg1.as_ref().unwrap(),
            cpu.bus.read16(cpu.pc())
        ),
        AddrMode::RADDR => &format!("[{}]", cpu.current_instruction.reg1.as_ref().unwrap()),
        AddrMode::RADDR_R => &format!(
            "[{}], {}",
            cpu.current_instruction.reg1.as_ref().unwrap(),
            cpu.current_instruction.reg2.as_ref().unwrap()
        ),
        AddrMode::RADDR_IMM => &format!(
            "[{}], $0x{:02X}",
            cpu.current_instruction.reg1.as_ref().unwrap(),
            cpu.bus.read(cpu.pc())
        ),
        AddrMode::R16 => &format!("{}", cpu.current_instruction.reg1.as_ref().unwrap()),
        AddrMode::R16_R16 => &format!(
            "{}, {}",
            cpu.current_instruction.reg1.as_ref().unwrap(),
            cpu.current_instruction.reg2.as_ref().unwrap()
        ),
        AddrMode::R16_SIMM => &format!(
            "{}, $0x{:02X}",
            cpu.current_instruction.reg1.as_ref().unwrap(),
            cpu.bus.read(cpu.pc()) as i8
        ),
        AddrMode::R16_IMM16 => &format!(
            "{}, $0x{:04X}",
            cpu.current_instruction.reg1.as_ref().unwrap(),
            cpu.bus.read16(cpu.pc())
        ),
        AddrMode::R16_R16_IMM => &format!(
            "{}, {}, $0x{:02X}",
            cpu.current_instruction.reg1.as_ref().unwrap(),
            cpu.current_instruction.reg2.as_ref().unwrap(),
            cpu.bus.read(cpu.pc()) as i8
        ),
        AddrMode::IMM_R => &format!(
            "$0x{:02X}, {}",
            cpu.bus.read(cpu.pc()),
            cpu.current_instruction.reg2.as_ref().unwrap()
        ),
        AddrMode::IMMADDR => &format!("[0x{:04X}]", cpu.bus.read16(cpu.pc())),
        AddrMode::IMM_RADDR => &format!(
            "$0x{:02X}, [{}]",
            cpu.bus.read(cpu.pc()),
            cpu.current_instruction.reg2.as_ref().unwrap()
        ),
        AddrMode::IMMADDR_R => &format!(
            "[0x{:04X}], {}",
            cpu.bus.read16(cpu.pc()),
            cpu.current_instruction.reg2.as_ref().unwrap()
        ),
        AddrMode::IMMADDR_R16 => &format!(
            "[{}], {}",
            cpu.bus.read16(cpu.pc()),
            cpu.current_instruction.reg2.as_ref().unwrap()
        ),
        AddrMode::SIMM => &format!("$0x{:02X}", cpu.bus.read(cpu.pc()) as i8),
    };

    let mem = format!(
        "({:02X} {:02X} {:02X})",
        cpu.bus.read(cpu.pc() - 1),
        cpu.bus.read(cpu.pc()),
        cpu.bus.read(cpu.pc() + 1)
    );

    let registers = format!(
        "A:0x{:02X} B:0x{:02X} C:0x{:02X} D:0x{:02X} E:0x{:02X} H:0x{:02X} L:0x{:02X} SP:0x{:02X}",
        cpu.get_register(&CPURegister::A),
        cpu.get_register(&CPURegister::B),
        cpu.get_register(&CPURegister::C),
        cpu.get_register(&CPURegister::D),
        cpu.get_register(&CPURegister::E),
        cpu.get_register(&CPURegister::H),
        cpu.get_register(&CPURegister::L),
        cpu.get_register16(&CPURegister::SP),
    );

    let flags = format!(
        "Z:{} N:{} H:{} C:{}",
        cpu.get_flag(CPUFlags::Z as u8),
        cpu.get_flag(CPUFlags::N as u8),
        cpu.get_flag(CPUFlags::H as u8),
        cpu.get_flag(CPUFlags::C as u8),
    );

    println!(
        "{} {} {} {} {}",
        format!("{:11}", header),
        format!("{:14}", args),
        mem,
        registers,
        flags,
    );
}

pub fn print_state_doctor(cpu: &mut LR35902CPU) {
    let registers = format!(
        "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X}",
        cpu.get_register(&CPURegister::A),
        cpu.get_register(&CPURegister::F),
        cpu.get_register(&CPURegister::B),
        cpu.get_register(&CPURegister::C),
        cpu.get_register(&CPURegister::D),
        cpu.get_register(&CPURegister::E),
        cpu.get_register(&CPURegister::H),
        cpu.get_register(&CPURegister::L),
        cpu.get_register16(&CPURegister::SP),
        cpu.get_register16(&CPURegister::PC),
    );

    let mem = format!(
        "PCMEM:{:02X},{:02X},{:02X},{:02X}",
        cpu.bus.read(cpu.pc()),
        cpu.bus.read(cpu.pc() + 1),
        cpu.bus.read(cpu.pc() + 2),
        cpu.bus.read(cpu.pc() + 3),
    );

    println!("{} {}", registers, mem);
}

pub fn print_serial(cpu: &mut LR35902CPU) {
    let char = cpu.bus.io.serial.get_char();

    if char != 0 {
        unsafe {
            serial_buff[serial_idx % 0x100] = char;
            serial_idx += 1;
            println!(
                "SERIAL DATA: {}",
                std::str::from_utf8(&serial_buff[..serial_idx]).expect("invalid utf-8 sequence")
            );
        }
    }
}
