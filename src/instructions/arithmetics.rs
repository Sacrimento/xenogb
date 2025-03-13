use crate::cpu::{LR35902CPU, CPUFlags};
use super::AddrMode;

pub fn add(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let result: u32;
    let c: bool;
    let h: bool;
    let z: bool;

    match instr.addr_mode {
        AddrMode::R_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            result = (
                cpu.get_register(instr.reg1.as_ref().unwrap())
                + cpu.bus.read(addr)
            ) as u32;
            z = (result as u8) == 0;
            c = result > 0xff;
            h = result > 0xf;
        },
        AddrMode::R_R => {
            result = (
                cpu.get_register(instr.reg1.as_ref().unwrap())
                + cpu.get_register(instr.reg2.as_ref().unwrap())
            ) as u32;
            z = (result as u8) == 0;
            c = result > 0xff;
            h = result > 0xf;
        },
        AddrMode::R_IMM => {
            result = (
                cpu.get_register(instr.reg1.as_ref().unwrap())
                + cpu.bus.read(cpu.pc())
            ) as u32;
            cpu.inc_pc(1);
            z = (result as u8) == 0;
            c = result > 0xff;
            h = result > 0xf;
        },
        AddrMode::R16_IMM16 => {
            result = (
                cpu.get_register16(instr.reg1.as_ref().unwrap())
                + cpu.bus.read16(cpu.pc())
            ) as u32;
            cpu.inc_pc(2);
            z = (result as u8) == 0;
            c = result > 0xffff;
            h = result > 0xfff;
        },
        AddrMode::R16_R16 => {
            result = (
                cpu.get_register16(instr.reg1.as_ref().unwrap())
                + cpu.get_register16(instr.reg2.as_ref().unwrap())
            ) as u32;
            z = (result as u16) == 0;
            c = result > 0xffff;
            h = result > 0xfff;
        },
        AddrMode::R16_SIMM => {
            result = (
                cpu.get_register16(instr.reg1.as_ref().unwrap())
                .wrapping_add_signed(cpu.bus.read(cpu.pc()) as i16)
            ) as u32;
            cpu.inc_pc(1);
            z = (result as u16) == 0;
            c = result > 0xff;
            h = result > 0xf;
        },
        _ => panic!("Unhandled addr mode for add")
    }

    cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
    cpu.set_flags(z as i8, 0, h as i8, c as i8);
}

pub fn adc(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let old_carry = cpu.get_flag(CPUFlags::C as u8);
    let result: u32;
    let new_c: bool;
    let h: bool;

    match instr.addr_mode {
        AddrMode::R_IMM => {
            result = (
                cpu.get_register(instr.reg1.as_ref().unwrap())
                + cpu.bus.read(cpu.pc())
                + old_carry
            ) as u32;
            cpu.inc_pc(1);
            new_c = result > 0xff;
            h = result > 0xf;
        },
        AddrMode::R_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            result = (
                cpu.get_register(instr.reg1.as_ref().unwrap())
                + cpu.bus.read(addr)
                + old_carry
            ) as u32;
            new_c = result > 0xff;
            h = result > 0xf;
        },
        AddrMode::R_R => {
            result = (
                cpu.get_register(instr.reg1.as_ref().unwrap())
                + cpu.get_register(instr.reg2.as_ref().unwrap())
                + old_carry
            ) as u32;
            new_c = result > 0xff;
            h = result > 0xf;
        },
        _ => panic!("Unhandled addr mode for adc")
    }

    cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
    cpu.set_flags(((result as u8) == 0) as i8, 0, h as i8, new_c as i8);
}

pub fn sub(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let a = cpu.get_register(instr.reg1.as_ref().unwrap());
    let value: u8;
    let result: i16;

    match instr.addr_mode {
        AddrMode::R_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
            result = (a - value) as i16;
        },
        AddrMode::R_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
            result = (a - value) as i16;
        },
        AddrMode::R_IMM => {
            value = cpu.bus.read(cpu.pc());
            cpu.inc_pc(1);
            result = (a - value) as i16;
        },
        _ => panic!("Unhandled addr mode for sub")
    }

    let h = ((a & 0xf) as i16 - (value & 0xf) as i16) < 0;
    cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
    cpu.set_flags(((result as u8) == 0) as i8, 1, h as i8, (result < 0) as i8);
}

pub fn sbc(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let a = cpu.get_register(instr.reg1.as_ref().unwrap());
    let c = cpu.get_flag(CPUFlags::C as u8);
    let value: u8;
    let result: i16;

    match instr.addr_mode {
        AddrMode::R_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
            result = (a - value - c) as i16;
        },
        AddrMode::R_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
            result = (a - value - c) as i16;
        },
        AddrMode::R_IMM => {
            value = cpu.bus.read(cpu.pc());
            cpu.inc_pc(1);
            result = (a - value - c) as i16;
        },
        _ => panic!("Unhandled addr mode for sub")
    }

    let h = ((a & 0xf) as i16 - (value & 0xf) as i16 - c as i16) < 0;
    cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
    cpu.set_flags(((result as u8) == 0) as i8, 1, h as i8, (result < 0) as i8);
}

pub fn inc(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let result: u32;

    match instr.addr_mode {
        AddrMode::R => {
            result = cpu.get_register(instr.reg1.as_ref().unwrap()) as u32 + 1;
            cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
        },
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            result = cpu.bus.read(addr) as u32 + 1;
            cpu.bus.write(addr, result as u8);
        },
        AddrMode::R16 => {
            result = cpu.get_register16(instr.reg1.as_ref().unwrap()) as u32 + 1;
            cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
        },
        _ => panic!("Unhandled addr mode for inc")
    }

    if !matches!(instr.addr_mode, AddrMode::R16) {
        cpu.set_flags(((result as u8) == 0) as i8, 0, (result > 0xf) as i8, -1);
    }
}

pub fn dec(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let result: u16;

    match instr.addr_mode {
        AddrMode::R => {
            result = cpu.get_register(instr.reg1.as_ref().unwrap()) as u16 - 1;
            cpu.set_register(instr.reg1.as_ref().unwrap(), result);
        },
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            result = cpu.bus.read(addr) as u16 - 1;
            cpu.bus.write(addr, result as u8);
        },
        AddrMode::R16 => {
            result = cpu.get_register16(instr.reg1.as_ref().unwrap()) as u16 - 1;
            cpu.set_register(instr.reg1.as_ref().unwrap(), result as u16);
        },
        _ => panic!("Unhandled addr mode for inc")
    }

    if !matches!(instr.addr_mode, AddrMode::R16) {
        cpu.set_flags(((result as u8) == 0) as i8, 1, ((result as u8) & 0x0f == 0x0f) as i8, -1);
    }
}

pub fn cp(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let a = cpu.get_register(instr.reg1.as_ref().unwrap());
    let value: u8;
    let result: i16;

    match instr.addr_mode {
        AddrMode::R_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
            result = (a - value) as i16;
        },
        AddrMode::R_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
            result = (a - value) as i16;
        },
        AddrMode::R_IMM => {
            value = cpu.bus.read(cpu.pc());
            cpu.inc_pc(1);
            result = (a - value) as i16;
        },
        _ => panic!("Unhandled addr mode for sub")
    }

    let h = ((a & 0xf) as i16 - (value & 0xf) as i16) < 0;
    cpu.set_flags(((result as u8) == 0) as i8, 1, h as i8, (result < 0) as i8);
}