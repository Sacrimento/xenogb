use crate::cpu::{LR35902CPU, CPUFlags};
use super::CPURegister;
use crate::utils::flip_bit;

pub fn ccf(cpu: &mut LR35902CPU) -> () {
    let c = cpu.get_flag(CPUFlags::C as u8);
    cpu.set_flags(-1, -1, -1, flip_bit(c, 0) as i8);
}

pub fn cpl(cpu: &mut LR35902CPU) -> () {
    let a = cpu.get_register(&CPURegister::A);
    cpu.set_register(&CPURegister::A, (!a) as u16);
}

pub fn scf(cpu: &mut LR35902CPU) -> () {
    cpu.set_flags(-1, 0, 0, 1);
}

pub fn daa(cpu: &mut LR35902CPU) -> () {
    let c = cpu.get_flag(CPUFlags::C as u8);
    let h = cpu.get_flag(CPUFlags::H as u8);
    let n = cpu.get_flag(CPUFlags::N as u8);
    let mut a = cpu.get_register(&CPURegister::A);

    let mut correction: u16 = if c != 0 {0x60} else {0x0};

    if h != 0 || (n == 0 && ((a & 0xf)) > 9) {
        correction |= 0x6;
    }

    if c != 0 || (n == 0 && (a > 0x99)) {
        correction |= 0x60;
    }
    
    if n != 0 {
        a = (a as u16 - correction) as u8;
    } else {
        a = (a as u16 + correction) as u8;
    }


    cpu.set_flags((a == 0) as i8, -1, 0, ((correction << 2) & 0x100 != 0) as i8);
    cpu.set_register(&CPURegister::A, a as u16);
}

pub fn halt(cpu: &mut LR35902CPU) -> () {
    cpu.halt = true;
}

pub fn stop(cpu: &mut LR35902CPU) -> () {
    todo!();
}

pub fn ei(cpu: &mut LR35902CPU) -> () {
    cpu.enabling_ints = true;
}

pub fn di(cpu: &mut LR35902CPU) -> () {
    cpu.int_master = false;
}
