use crate::cpu::{LR35902CPU, CPUFlags};
use super::{AddrMode, CPURegister};
use crate::utils::{get_bit, set_bit};

pub fn and(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let a = cpu.get_register(instr.reg1.as_ref().unwrap());
    let mut value: u8;

    match instr.addr_mode {
        AddrMode::R_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
        },
        AddrMode::R_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
        },
        AddrMode::R_IMM => {
            value = cpu.bus.read(cpu.pc());
            cpu.inc_pc(1);
        },
        _ => panic!("Unhandled addr mode for and")
    }

    value &= a;
    cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
    cpu.set_flags((value == 0) as i8, 0, 1, 0);
}

pub fn or(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let a = cpu.get_register(instr.reg1.as_ref().unwrap());
    let mut value: u8;

    match instr.addr_mode {
        AddrMode::R_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
        },
        AddrMode::R_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
        },
        AddrMode::R_IMM => {
            value = cpu.bus.read(cpu.pc());
            cpu.inc_pc(1);
        },
        _ => panic!("Unhandled addr mode for or")
    }

    value |= a;
    cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
    cpu.set_flags((value == 0) as i8, 0, 0, 0);
}

pub fn xor(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let a = cpu.get_register(instr.reg1.as_ref().unwrap());
    let mut value: u8;

    match instr.addr_mode {
        AddrMode::R_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
        },
        AddrMode::R_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
        },
        AddrMode::R_IMM => {
            value = cpu.bus.read(cpu.pc());
            cpu.inc_pc(1);
        },
        _ => panic!("Unhandled addr mode for xor")
    }

    value ^= a;
    cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
    cpu.set_flags((value == 0) as i8, 0, 0, 0);
}

pub fn bit(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let value: u8;

    match instr.addr_mode {
        AddrMode::IMM_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
        },
        AddrMode::IMM_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
        },
        _ => panic!("Unhandled addr mode for bit")
    }

    let bit = cpu.bus.read(cpu.pc());
    cpu.inc_pc(1);

    cpu.set_flags(get_bit(value, bit) as i8, 0, 1, -1);
}

pub fn res(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let mut value: u8;
    let bit: u8;

    match instr.addr_mode {
        AddrMode::IMM_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
            bit = cpu.bus.read(cpu.pc());
            cpu.inc_pc(1);
            value = set_bit(value, bit, 0);
            cpu.set_register(instr.reg2.as_ref().unwrap(), value as u16);
        },
        AddrMode::IMM_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
            bit = cpu.bus.read(cpu.pc());
            cpu.inc_pc(1);
            value = set_bit(value, bit, 0);
            cpu.bus.write(addr, value);
        },
        _ => panic!("Unhandled addr mode for res")
    }
}

pub fn set(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let mut value: u8;
    let bit: u8;

    match instr.addr_mode {
        AddrMode::IMM_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
            bit = cpu.bus.read(cpu.pc());
            cpu.inc_pc(1);
            value = set_bit(value, bit, 1);
            cpu.set_register(instr.reg2.as_ref().unwrap(), value as u16);
        },
        AddrMode::IMM_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
            bit = cpu.bus.read(cpu.pc());
            cpu.inc_pc(1);
            value = set_bit(value, bit, 1);
            cpu.bus.write(addr, value);
        },
        _ => panic!("Unhandled addr mode for set")
    }
}

pub fn swap(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let mut value: u8;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            value = ((value & 0xf) << 4) | ((value & 0xf0) >> 4);
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        },
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            value = ((value & 0xf) << 4) | ((value & 0xf0) >> 4);
            cpu.bus.write(addr, value);
        },
        _ => panic!("Unhandled addr mode for set")
    }

    cpu.set_flags((value == 0) as i8, 0, 0, 0);
}

pub fn rl(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let old_carry = cpu.get_flag(CPUFlags::C as u8);
    let new_carry: u8;
    let mut value: u8;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = value >> 7;
            value = (value << 1) | old_carry;
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        },
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = value >> 7;
            value = (value << 1) | old_carry;
            cpu.bus.write(addr, value);
        },
        _ => panic!("Unhandled addr mode for set")
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
}

pub fn rla(cpu: &mut LR35902CPU) -> () {
    let mut value = cpu.get_register(&CPURegister::A);
    let new_carry = value >> 7;

    value = (value << 1) | cpu.get_flag(CPUFlags::C as u8);
    cpu.set_register(&CPURegister::A, value as u16);

    cpu.set_flags(0, 0, 0, new_carry as i8);
}

pub fn rlc(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let new_carry: u8;
    let mut value: u8;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = value >> 7;
            value = (value << 1) | new_carry;
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        },
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = value >> 7;
            value = (value << 1) | new_carry;
            cpu.bus.write(addr, value);
        },
        _ => panic!("Unhandled addr mode for set")
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
}

pub fn rlca(cpu: &mut LR35902CPU) -> () { 
    let mut value = cpu.get_register(&CPURegister::A);
    let new_carry = value >> 7;
    value = (value << 1) | new_carry;
    cpu.set_register(&CPURegister::A, value as u16);

    cpu.set_flags(0, 0, 0, new_carry as i8);
}

pub fn rr(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let old_carry = cpu.get_flag(CPUFlags::C as u8);
    let new_carry: u8;
    let mut value: u8;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = value & 1;
            value = (value >> 1) | (old_carry << 7);
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        },
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = value & 1;
            value = (value >> 1) | (old_carry << 7);
            cpu.bus.write(addr, value);
        },
        _ => panic!("Unhandled addr mode for set")
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
}

pub fn rra(cpu: &mut LR35902CPU) -> () {
    let mut value = cpu.get_register(&CPURegister::A);
    let new_carry = value & 1;

    value = (value >> 1) | (cpu.get_flag(CPUFlags::C as u8) << 7);
    cpu.set_register(&CPURegister::A, value as u16);

    cpu.set_flags(0, 0, 0, new_carry as i8);
}

pub fn rrc(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let new_carry: u8;
    let mut value: u8;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = value & 1;
            value >>= 1;
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        },
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = value & 1;
            value >>= 1;
            cpu.bus.write(addr, value);
        },
        _ => panic!("Unhandled addr mode for set")
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
}

pub fn rrca(cpu: &mut LR35902CPU) -> () { 
    let mut value = cpu.get_register(&CPURegister::A);
    let new_carry = value & 1;
    value >>= 1;
    cpu.set_register(&CPURegister::A, value as u16);

    cpu.set_flags(0, 0, 0, new_carry as i8);
}

pub fn sla(cpu: &mut LR35902CPU) -> () { 
    let instr = cpu.current_instruction;
    let new_carry: u8;
    let mut value: u8;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = get_bit(value, 7);
            value <<= 1;
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        },
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = get_bit(value, 7);
            value <<= 1;
            cpu.bus.write(addr, value);
        },
        _ => panic!("Unhandled addr mode for set")
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
}

pub fn sra(cpu: &mut LR35902CPU) -> () { 
    let instr = cpu.current_instruction;
    let new_carry: u8;
    let mut value: u8;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = value & 1;
            value >>= 1;
            value = set_bit(value, 7, (value >> 7) & 1);
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        },
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = value & 1;
            value >>= 1;
            value = set_bit(value, 7, (value >> 7) & 1);
            cpu.bus.write(addr, value);
        },
        _ => panic!("Unhandled addr mode for set")
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
}

pub fn srl(cpu: &mut LR35902CPU) -> () { 
    let instr = cpu.current_instruction;
    let new_carry: u8;
    let mut value: u8;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = value & 1;
            value >>= 1;
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        },
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = value & 1;
            value >>= 1;
            cpu.bus.write(addr, value);
        },
        _ => panic!("Unhandled addr mode for set")
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
}
