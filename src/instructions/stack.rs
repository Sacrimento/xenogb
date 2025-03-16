use super::CPURegister;
use crate::cpu::LR35902CPU;

pub fn _pop(cpu: &mut LR35902CPU) -> u8 {
    let v = cpu.bus.read(cpu.sp());
    cpu.inc_sp();
    v
}

pub fn pop(cpu: &mut LR35902CPU) -> u8 {
    let instr = cpu.current_instruction;
    let regs: [&CPURegister; 2];

    match instr.reg1.as_ref().unwrap() {
        CPURegister::AF => regs = [&CPURegister::F, &CPURegister::A],
        CPURegister::BC => regs = [&CPURegister::C, &CPURegister::B],
        CPURegister::DE => regs = [&CPURegister::E, &CPURegister::D],
        CPURegister::HL => regs = [&CPURegister::L, &CPURegister::H],
        _ => panic!("Invalid register for pop"),
    }

    for reg in regs {
        let mut v = _pop(cpu);
        if reg == &CPURegister::F {
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
    let regs: [&CPURegister; 2];

    match instr.reg1.as_ref().unwrap() {
        CPURegister::AF => regs = [&CPURegister::A, &CPURegister::F],
        CPURegister::BC => regs = [&CPURegister::B, &CPURegister::C],
        CPURegister::DE => regs = [&CPURegister::D, &CPURegister::E],
        CPURegister::HL => regs = [&CPURegister::H, &CPURegister::L],
        _ => panic!("Invalid register for pop"),
    }

    for reg in regs {
        let mut v = cpu.get_register(reg);
        if reg == &CPURegister::F {
            v = v & 0xf0;
        }
        _push(cpu, v);
    }
    4
}
