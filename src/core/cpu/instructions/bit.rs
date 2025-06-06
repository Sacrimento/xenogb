use super::{AddrMode, CPURegisterId};
use crate::core::cpu::cpu::{CPUFlags, LR35902CPU};

pub fn and(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let a = cpu.get_register(instr.reg1.as_ref().unwrap());
    let mut value: u8;
    let mut cycles: u8 = 2;

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

    value &= a;
    cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
    cpu.set_flags((value == 0) as i8, 0, 1, 0);
    cycles
}

pub fn or(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let a = cpu.get_register(instr.reg1.as_ref().unwrap());
    let mut value: u8;
    let mut cycles: u8 = 2;

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

    value |= a;
    cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
    cpu.set_flags((value == 0) as i8, 0, 0, 0);
    cycles
}

pub fn xor(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let a = cpu.get_register(instr.reg1.as_ref().unwrap());
    let mut value: u8;
    let mut cycles: u8 = 2;

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

    value ^= a;
    cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
    cpu.set_flags((value == 0) as i8, 0, 0, 0);
    cycles
}

pub fn bit(cpu: &mut LR35902CPU, nth_bit: u8) -> u8 {
    let instr = cpu.current_instruction;
    let value: u8;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::IMM_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
        }
        AddrMode::IMM_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
            cycles = 3;
        }
        _ => unreachable!(),
    }

    cpu.set_flags((((value >> nth_bit) & 1) == 0) as i8, 0, 1, -1);
    cycles
}

pub fn res(cpu: &mut LR35902CPU, nth_bit: u8) -> u8 {
    let instr = cpu.current_instruction;
    let mut value: u8;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::IMM_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
            value &= !(1 << nth_bit);
            cpu.set_register(instr.reg2.as_ref().unwrap(), value as u16);
        }
        AddrMode::IMM_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
            value &= !(1 << nth_bit);
            cpu.bus.write(addr, value);
            cycles = 4;
        }
        _ => unreachable!(),
    }

    cycles
}

pub fn set(cpu: &mut LR35902CPU, nth_bit: u8) -> u8 {
    let instr = cpu.current_instruction;
    let mut value: u8;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::IMM_R => {
            value = cpu.get_register(instr.reg2.as_ref().unwrap());
            value |= 1 << nth_bit;
            cpu.set_register(instr.reg2.as_ref().unwrap(), value as u16);
        }
        AddrMode::IMM_RADDR => {
            let addr = cpu.get_register16(instr.reg2.as_ref().unwrap());
            value = cpu.bus.read(addr);
            value |= 1 << nth_bit;
            cpu.bus.write(addr, value);
            cycles = 4;
        }
        _ => unreachable!(),
    }
    cycles
}

pub fn swap(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let mut value: u8;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            value = ((value & 0xf) << 4) | ((value & 0xf0) >> 4);
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        }
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            value = ((value & 0xf) << 4) | ((value & 0xf0) >> 4);
            cpu.bus.write(addr, value);
            cycles = 4;
        }
        _ => unreachable!(),
    }

    cpu.set_flags((value == 0) as i8, 0, 0, 0);
    cycles
}

pub fn rl(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let old_carry = cpu.get_flag(CPUFlags::C);
    let new_carry: u8;
    let mut value: u8;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = value >> 7;
            value = (value << 1) | old_carry;
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        }
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = value >> 7;
            value = (value << 1) | old_carry;
            cpu.bus.write(addr, value);
            cycles = 4;
        }
        _ => unreachable!(),
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
    cycles
}

pub fn rla(cpu: &mut LR35902CPU) -> u8 {
    let mut value = cpu.get_register(&CPURegisterId::A);
    let new_carry = value >> 7;

    value = (value << 1) | cpu.get_flag(CPUFlags::C);
    cpu.set_register(&CPURegisterId::A, value as u16);

    cpu.set_flags(0, 0, 0, new_carry as i8);
    1
}

pub fn rlc(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let new_carry: u8;
    let mut value: u8;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = value >> 7;
            value = (value << 1) | new_carry;
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        }
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = value >> 7;
            value = (value << 1) | new_carry;
            cpu.bus.write(addr, value);
            cycles = 4;
        }
        _ => unreachable!(),
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
    cycles
}

pub fn rlca(cpu: &mut LR35902CPU) -> u8 {
    let mut value = cpu.get_register(&CPURegisterId::A);
    let new_carry = value >> 7;
    value = (value << 1) | new_carry;
    cpu.set_register(&CPURegisterId::A, value as u16);

    cpu.set_flags(0, 0, 0, new_carry as i8);
    1
}

pub fn rr(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let old_carry = cpu.get_flag(CPUFlags::C);
    let new_carry: u8;
    let mut value: u8;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = value & 1;
            value = (value >> 1) | (old_carry << 7);
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        }
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = value & 1;
            value = (value >> 1) | (old_carry << 7);
            cpu.bus.write(addr, value);
            cycles = 4;
        }
        _ => unreachable!(),
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
    cycles
}

pub fn rra(cpu: &mut LR35902CPU) -> u8 {
    let mut value = cpu.get_register(&CPURegisterId::A);
    let new_carry = value & 1;

    value = (value >> 1) | (cpu.get_flag(CPUFlags::C) << 7);
    cpu.set_register(&CPURegisterId::A, value as u16);

    cpu.set_flags(0, 0, 0, new_carry as i8);
    1
}

pub fn rrc(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let new_carry: u8;
    let mut value: u8;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = value & 1;
            value = (value >> 1) | ((value & 1) << 7);
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        }
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = value & 1;
            value = (value >> 1) | ((value & 1) << 7);
            cpu.bus.write(addr, value);
            cycles = 4;
        }
        _ => unreachable!(),
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
    cycles
}

pub fn rrca(cpu: &mut LR35902CPU) -> u8 {
    let mut value = cpu.get_register(&CPURegisterId::A);
    let new_carry = value & 1;
    value = (value >> 1) | ((value & 1) << 7);
    cpu.set_register(&CPURegisterId::A, value as u16);

    cpu.set_flags(0, 0, 0, new_carry as i8);
    1
}

pub fn sla(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let new_carry: u8;
    let mut value: u8;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = (value >> 7) & 1;
            value <<= 1;
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        }
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = (value >> 7) & 1;
            value <<= 1;
            cpu.bus.write(addr, value);
            cycles = 4;
        }
        _ => unreachable!(),
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
    cycles
}

pub fn sra(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let new_carry: u8;
    let mut value: u8;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = value & 1;
            value = (value >> 1) | (value & 0x80);
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        }
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = value & 1;
            value = (value >> 1) | (value & 0x80);
            cpu.bus.write(addr, value);
            cycles = 4;
        }
        _ => unreachable!(),
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
    cycles
}

pub fn srl(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let new_carry: u8;
    let mut value: u8;
    let mut cycles: u8 = 2;

    match instr.addr_mode {
        AddrMode::R => {
            value = cpu.get_register(instr.reg1.as_ref().unwrap());
            new_carry = value & 1;
            value >>= 1;
            cpu.set_register(instr.reg1.as_ref().unwrap(), value as u16);
        }
        AddrMode::RADDR => {
            let addr = cpu.get_register16(instr.reg1.as_ref().unwrap());
            value = cpu.bus.read(addr);
            new_carry = value & 1;
            value >>= 1;
            cpu.bus.write(addr, value);
            cycles = 4;
        }
        _ => unreachable!(),
    }

    cpu.set_flags((value == 0) as i8, 0, 0, new_carry as i8);
    cycles
}
