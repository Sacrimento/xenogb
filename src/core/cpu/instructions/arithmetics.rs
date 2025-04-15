use super::AddrMode;
use crate::core::cpu::cpu::{CPUFlags, LR35902CPU};

pub fn add(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let result: u32;
    let c: bool;
    let h: bool;
    let op1;
    let op2;
    let mut z: i8 = -1;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::R_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            op1 = cpu.get_register(instr.reg1.as_ref().unwrap()) as u32;
            op2 = cpu.bus.read(addr) as u32;
            result = op1 + op2;
            z = if (result as u8) == 0 { 1 } else { 0 };
            c = result > 0xff;
            h = (op1 & 0xf) + (op2 & 0xf) > 0xf;
        }
        AddrMode::R_R => {
            op1 = cpu.get_register(instr.reg1.as_ref().unwrap()) as u32;
            op2 = cpu.get_register(instr.reg2.as_ref().unwrap()) as u32;
            result = op1 + op2;
            z = if (result as u8) == 0 { 1 } else { 0 };
            c = result > 0xff;
            h = (op1 & 0xf) + (op2 & 0xf) > 0xf;
            cycles = 1;
        }
        AddrMode::R_IMM => {
            op1 = cpu.get_register(instr.reg1.as_ref().unwrap()) as u32;
            op2 = cpu.bus.read(cpu.pc()) as u32;
            result = op1 + op2;
            cpu.inc_pc(1);
            z = if (result as u8) == 0 { 1 } else { 0 };
            c = result > 0xff;
            h = (op1 & 0xf) + (op2 & 0xf) > 0xf;
        }
        AddrMode::R16_IMM16 => {
            op1 = cpu.get_register16(instr.reg1.as_ref().unwrap()) as u32;
            op2 = cpu.bus.read16(cpu.pc()) as u32;
            result = op1 + op2;
            cpu.inc_pc(2);
            c = result > 0xffff;
            h = (op1 & 0xfff) + (op2 & 0xfff) > 0xfff;
        }
        AddrMode::R16_R16 => {
            op1 = cpu.get_register16(instr.reg1.as_ref().unwrap()) as u32;
            op2 = cpu.get_register16(instr.reg2.as_ref().unwrap()) as u32;
            result = op1 + op2;
            c = result > 0xffff;
            h = (op1 & 0xfff) + (op2 & 0xfff) > 0xfff;
        }
        AddrMode::R16_SIMM => {
            let op1 = cpu.get_register16(instr.reg1.as_ref().unwrap());
            let op2 = cpu.bus.read(cpu.pc()) as i8;
            result = op1.wrapping_add_signed(op2.into()) as u32;
            cpu.inc_pc(1);
            z = 0;
            c = ((op1 ^ (op2 as u16) ^ ((result as u16) & 0xFFFF)) & 0x100) == 0x100; // ????
            h = ((op1 ^ (op2 as u16) ^ ((result as u16) & 0xFFFF)) & 0x10) == 0x10;
            cycles = 4;
        }
        _ => unreachable!(),
    }

    cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
    cpu.set_flags(z as i8, 0, h as i8, c as i8);
    cycles
}

pub fn adc(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let c = cpu.get_flag(CPUFlags::C);
    let op1: u32;
    let op2: u32;
    let result: u32;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::R_IMM => {
            op1 = cpu.get_register(instr.reg1.as_ref().unwrap()) as u32;
            op2 = cpu.bus.read(cpu.pc()) as u32;
            result = op1 + op2 + c as u32;
            cpu.inc_pc(1);
        }
        AddrMode::R_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            op1 = cpu.get_register(instr.reg1.as_ref().unwrap()) as u32;
            op2 = cpu.bus.read(addr) as u32;
            result = op1 + op2 + c as u32;
        }
        AddrMode::R_R => {
            op1 = cpu.get_register(instr.reg1.as_ref().unwrap()) as u32;
            op2 = cpu.get_register(instr.reg2.as_ref().unwrap()) as u32;
            result = op1 + op2 + c as u32;
            cycles = 1;
        }
        _ => unreachable!(),
    }

    cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
    cpu.set_flags(
        ((result as u8) == 0) as i8,
        0,
        (((op1 & 0xf) + (op2 & 0xf) + c as u32) > 0xf) as i8,
        (result > 0xff) as i8,
    );
    cycles
}

pub fn sub(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let a = cpu.get_register(instr.reg1.as_ref().unwrap());
    let value: u8;
    let result: i16;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::R_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
            result = a as i16 - value as i16;
            cycles = 1;
        }
        AddrMode::R_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
            result = a as i16 - value as i16;
        }
        AddrMode::R_IMM => {
            value = cpu.bus.read(cpu.pc());
            cpu.inc_pc(1);
            result = a as i16 - value as i16;
        }
        _ => unreachable!(),
    }

    let h = ((a & 0xf) as i16 - (value & 0xf) as i16) < 0;
    cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
    cpu.set_flags(((result as u8) == 0) as i8, 1, h as i8, (result < 0) as i8);
    cycles
}

pub fn sbc(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let a = cpu.get_register(instr.reg1.as_ref().unwrap());
    let c = cpu.get_flag(CPUFlags::C);
    let value: u8;
    let mut cycles = 2;

    match instr.addr_mode {
        AddrMode::R_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
            cycles = 1;
        }
        AddrMode::R_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
        }
        AddrMode::R_IMM => {
            value = cpu.bus.read(cpu.pc());
            cpu.inc_pc(1);
        }
        _ => unreachable!(),
    }

    let result = a as i16 - value as i16 - c as i16;
    let h = ((a & 0xf) as i16 - (value & 0xf) as i16 - c as i16) < 0;
    cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
    cpu.set_flags(((result as u8) == 0) as i8, 1, h as i8, (result < 0) as i8);
    cycles
}

pub fn inc(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let result: u32;
    let mut cycles: u8 = 1;

    match instr.addr_mode {
        AddrMode::R => {
            result = cpu.get_register(instr.reg1.as_ref().unwrap()) as u32 + 1;
            cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
        }
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            result = cpu.bus.read(addr) as u32 + 1;
            cpu.bus.write(addr, result as u8);
            cycles = 3;
        }
        AddrMode::R16 => {
            result = cpu.get_register16(instr.reg1.as_ref().unwrap()) as u32 + 1;
            cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
            cycles = 2;
        }
        _ => unreachable!(),
    }

    if !matches!(instr.addr_mode, AddrMode::R16) {
        cpu.set_flags(
            ((result as u8) == 0) as i8,
            0,
            (result & 0xf == 0) as i8,
            -1,
        );
    }
    cycles
}

pub fn dec(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let result: i32;
    let mut cycles: u8 = 1;

    match instr.addr_mode {
        AddrMode::R => {
            result = cpu.get_register(instr.reg1.as_ref().unwrap()) as i32 - 1;
            cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
        }
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            result = cpu.bus.read(addr) as i32 - 1;
            cpu.bus.write(addr, result as u8);
            cycles = 3;
        }
        AddrMode::R16 => {
            result = cpu.get_register16(instr.reg1.as_ref().unwrap()) as i32 - 1;
            cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
            cycles = 2;
        }
        _ => unreachable!(),
    }

    if !matches!(instr.addr_mode, AddrMode::R16) {
        cpu.set_flags(
            ((result as u8) == 0) as i8,
            1,
            ((result as u8) & 0x0f == 0x0f) as i8,
            -1,
        );
    }
    cycles
}

pub fn cp(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let a = cpu.get_register(instr.reg1.as_ref().unwrap()) as i16;
    let value: i16;
    let mut cycles = 2;

    match instr.addr_mode {
        AddrMode::R_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap()) as i16;
            cycles = 1;
        }
        AddrMode::R_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr) as i16;
        }
        AddrMode::R_IMM => {
            value = cpu.bus.read(cpu.pc()) as i16;
            cpu.inc_pc(1);
        }
        _ => unreachable!(),
    }

    let result = a - value;
    let h = ((a & 0xf) as i16 - (value & 0xf) as i16) < 0;
    cpu.set_flags(((result as u8) == 0) as i8, 1, h as i8, (result < 0) as i8);
    cycles
}
