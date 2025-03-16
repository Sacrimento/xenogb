use super::CPURegisterId;
use crate::cpu::LR35902CPU;

pub fn _pop(cpu: &mut LR35902CPU) -> u8 {
    let v = cpu.bus.read(cpu.sp());
    cpu.inc_sp();
    v
}

pub fn pop(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let regs: [&CPURegisterId; 2];

    match instr.reg1.as_ref().unwrap() {
        CPURegisterId::AF => regs = [&CPURegisterId::F, &CPURegisterId::A],
        CPURegisterId::BC => regs = [&CPURegisterId::C, &CPURegisterId::B],
        CPURegisterId::DE => regs = [&CPURegisterId::E, &CPURegisterId::D],
        CPURegisterId::HL => regs = [&CPURegisterId::L, &CPURegisterId::H],
        _ => panic!("Invalid register for pop"),
    }

    for reg in regs {
        let mut v = _pop(cpu);
        if reg == &CPURegisterId::F {
            v = v & 0xf0;
        }
        cpu.set_register(reg, v as u16);
    }
    3
}

pub fn _push(cpu: &mut LR35902CPU, value: u8) -> () {
    cpu.dec_sp();
    cpu.bus.write(cpu.sp(), value);
}

pub fn push(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let regs: [&CPURegisterId; 2];

    match instr.reg1.as_ref().unwrap() {
        CPURegisterId::AF => regs = [&CPURegisterId::A, &CPURegisterId::F],
        CPURegisterId::BC => regs = [&CPURegisterId::B, &CPURegisterId::C],
        CPURegisterId::DE => regs = [&CPURegisterId::D, &CPURegisterId::E],
        CPURegisterId::HL => regs = [&CPURegisterId::H, &CPURegisterId::L],
        _ => panic!("Invalid register for pop"),
    }

    for reg in regs {
        let mut v = cpu.get_register(reg);
        if reg == &CPURegisterId::F {
            v = v & 0xf0;
        }
        _push(cpu, v);
    }
    4
}
