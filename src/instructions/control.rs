use crate::cpu::{LR35902CPU, CPUFlags};
use super::{AddrMode, CPURegister, CondType};
use super::stack::{_pop, _push};

fn check_cond(cpu: &mut LR35902CPU, cond: Option<&CondType>) -> bool {
    if cond.is_none() { return true }
    
    let z = cpu.get_flag(CPUFlags::Z as u8);
    let c = cpu.get_flag(CPUFlags::C as u8);
    
    match cond.unwrap() {
        CondType::Nz => { z == 0 },
        CondType::Z => { z == 1 },
        CondType::Nc => { c == 0 },
        CondType::C => { c == 1 },
    }
}

pub fn jr(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    
    if !check_cond(cpu, instr.condition.as_ref()) {
        cpu.inc_pc(1);
        return
    }

    let mut pc = cpu.pc();

    let offset = cpu.bus.read(pc) as i8;
    pc = u16::try_from((pc as i32) + (offset as i32) + 1).expect("Could not convert for jr");

    cpu.set_register(&CPURegister::PC, pc);
}

pub fn jp(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let pc: u16;

    if !check_cond(cpu, instr.condition.as_ref()) {
        if matches!(instr.addr_mode, AddrMode::IMMADDR) {
            cpu.inc_pc(2);
        }
        return
    }

    match instr.addr_mode {
        AddrMode::R16 => { pc = cpu.get_register16(instr.reg1.as_ref().unwrap()) },
        AddrMode::IMMADDR => { pc = cpu.bus.read16(cpu.pc()); }
        _ => panic!("Unhandled addr mode for jp")
    }

    cpu.set_register(&CPURegister::PC, pc);
}

pub fn call(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;

    if !check_cond(cpu, instr.condition.as_ref()) {
        cpu.inc_pc(2);
        return
    }

    let v = cpu.pc() + 2;
    _push(cpu, (v >> 8) as u8);
    _push(cpu, (v & 0xff) as u8);

    let pc = cpu.bus.read16(cpu.pc());


    cpu.set_register(&CPURegister::PC, pc);
}

pub fn ret(cpu: &mut LR35902CPU) -> () {
    let instr = cpu.current_instruction;
    let pc: u16;

    if !check_cond(cpu, instr.condition.as_ref()) {
        return
    }

    pc = (_pop(cpu) as u16) | ((_pop(cpu) as u16) << 8);
    cpu.set_register(&CPURegister::PC, pc);
}

pub fn reti(cpu: &mut LR35902CPU) -> () {
    cpu.enabling_ints = true;

    let pc: u16 = ((_pop(cpu) as u16) << 8) | (_pop(cpu) as u16);
    cpu.set_register(&CPURegister::PC, pc);
}

pub fn rst(cpu: &mut LR35902CPU, addr: u8) -> () {
    let v = cpu.pc() + 1;
    _push(cpu, (v & 0xff) as u8);
    _push(cpu, (v >> 8) as u8);

    cpu.set_register(&CPURegister::PC, addr as u16);
}
