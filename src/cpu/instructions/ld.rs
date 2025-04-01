use super::AddrMode;
use crate::cpu::cpu::LR35902CPU;

pub fn ldr(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let value: u16;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::R16_IMM16 => {
            value = cpu.bus.read16(cpu.pc());
            cpu.inc_pc(2);
        }
        AddrMode::R_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap()) as u16;
            cycles = 1;
        }
        AddrMode::R_IMM => {
            value = cpu.bus.read(cpu.pc()) as u16;
            cpu.inc_pc(1);
        }
        AddrMode::R_RADDR => {
            value = cpu
                .bus
                .read(cpu.get_register16(instr.reg2.as_ref().unwrap())) as u16;
        }
        AddrMode::R16_R16_IMM => {
            let sp = cpu.get_register16(instr.reg2.as_ref().unwrap());
            let offset = cpu.bus.read(cpu.pc()) as i8;
            cpu.inc_pc(1);

            value = sp.wrapping_add_signed(offset.into());
            let h = ((sp ^ (offset as u16) ^ ((value as u16) & 0xFFFF)) & 0x10) == 0x10;
            let c = ((sp ^ (offset as u16) ^ ((value as u16) & 0xFFFF)) & 0x100) == 0x100; // ????
            cpu.set_flags(0, 0, h as i8, c as i8);
            cycles = 3;
        }
        AddrMode::R_IMMADDR => {
            let addr = cpu.bus.read16(cpu.pc());
            cpu.inc_pc(2);
            value = cpu.bus.read(addr) as u16;
            cycles = 4;
        }
        AddrMode::R16_R16 => {
            value = cpu.get_register16(instr.reg2.as_ref().unwrap());
        }
        _ => panic!("Unhandled addr mode for ldr"),
    }

    cpu.set_register(instr.reg1.as_ref().unwrap(), value);
    cycles
}

pub fn ldm(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let addr: u16;
    let value: u8;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::RADDR_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
            addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
        }
        AddrMode::IMMADDR_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
            addr = cpu.bus.read16(cpu.pc());
            cpu.inc_pc(2);
            cycles = 4;
        }
        AddrMode::IMMADDR_R16 => {
            addr = cpu.bus.read16(cpu.pc());
            cpu.inc_pc(2);

            let reg = cpu.get_register16(instr.reg2.as_ref().unwrap());
            cpu.bus.write(addr + 1, (reg >> 8) as u8);
            value = (reg & 0xff) as u8;
            cycles = 5;
        }
        AddrMode::RADDR_IMM => {
            addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(cpu.pc());
            cpu.inc_pc(1);
            cycles = 3;
        }
        _ => panic!("Unhandled addr mode for ldm"),
    }

    cpu.bus.write(addr, value);
    cycles
}

pub fn ldh(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let mut cycles: u8 = 3;

    if (matches!(instr.addr_mode, AddrMode::IMM_R) || matches!(instr.addr_mode, AddrMode::RADDR_R))
    {
        let mut addr = 0xff00;
        if matches!(instr.addr_mode, AddrMode::IMM_R) {
            // opcode 0xe0
            addr += cpu.bus.read(cpu.pc()) as u16;
            cpu.inc_pc(1);
        } else {
            // opcode 0xe2
            addr += cpu.get_register(instr.reg1.as_ref().unwrap()) as u16;
            cycles = 2;
        }
        cpu.bus
            .write(addr, cpu.get_register(instr.reg2.as_ref().unwrap()));
    } else {
        let value: u16;
        if matches!(instr.addr_mode, AddrMode::R_IMMADDR) {
            // opcode 0xf0
            let addr = cpu.bus.read(cpu.pc()) as u16;
            cpu.inc_pc(1);
            value = cpu.bus.read(0xff00 + addr) as u16;
        } else {
            // opcode 0xf2
            let reg = cpu.get_register(instr.reg2.as_ref().unwrap()) as u16;
            value = cpu.bus.read(0xff00 + reg) as u16;
            cycles = 2;
        }
        cpu.set_register(instr.reg1.as_ref().unwrap(), value);
    }
    cycles
}

pub fn ldi(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;

    if matches!(instr.addr_mode, AddrMode::RADDR_R) {
        // opcode 0x22
        let hl = cpu.get_register16(instr.reg1.as_ref().unwrap());
        cpu.set_register(instr.reg1.as_ref().unwrap(), hl + 1);
        cpu.bus
            .write(hl, cpu.get_register(instr.reg2.as_ref().unwrap()));
    } else {
        // opcode 0x2a
        let hl = cpu.get_register16(instr.reg2.as_ref().unwrap());
        cpu.set_register(instr.reg2.as_ref().unwrap(), hl + 1);
        let value = cpu.bus.read(hl);
        cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
    }
    2
}

pub fn ldd(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;

    if matches!(instr.addr_mode, AddrMode::RADDR_R) {
        // opcode 0x32
        let hl = cpu.get_register16(instr.reg1.as_ref().unwrap());
        cpu.set_register(instr.reg1.as_ref().unwrap(), hl - 1);
        cpu.bus
            .write(hl, cpu.get_register(instr.reg2.as_ref().unwrap()));
    } else {
        //opcode 0x3a
        let hl = cpu.get_register16(instr.reg2.as_ref().unwrap());
        cpu.set_register(instr.reg2.as_ref().unwrap(), hl - 1);
        let value = cpu.bus.read(hl);
        cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
    }
    2
}
