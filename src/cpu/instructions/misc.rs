use super::CPURegisterId;
use crate::cpu::cpu::{CPUFlags, LR35902CPU};

pub fn ccf(cpu: &mut LR35902CPU) -> u8 {
    let c = cpu.get_flag(CPUFlags::C);
    cpu.set_flags(-1, 0, 0, (c ^ 1) as i8);
    1
}

pub fn cpl(cpu: &mut LR35902CPU) -> u8 {
    let a = cpu.get_register(&CPURegisterId::A);
    cpu.set_register(&CPURegisterId::A, (!a) as u16);
    cpu.set_flags(-1, 1, 1, -1);
    1
}

pub fn scf(cpu: &mut LR35902CPU) -> u8 {
    cpu.set_flags(-1, 0, 0, 1);
    1
}

pub fn daa(cpu: &mut LR35902CPU) -> u8 {
    let c = cpu.get_flag(CPUFlags::C);
    let h = cpu.get_flag(CPUFlags::H);
    let n = cpu.get_flag(CPUFlags::N);
    let mut a = cpu.get_register(&CPURegisterId::A);

    let mut correction: u16 = if c != 0 { 0x60 } else { 0x0 };

    if h != 0 || (n == 0 && (a & 0xf) > 9) {
        correction |= 0x6;
    }

    if c != 0 || (n == 0 && (a > 0x99)) {
        correction |= 0x60;
    }

    if n != 0 {
        a = (a as i16 - correction as i16) as u8;
    } else {
        a = (a as u16 + correction) as u8;
    }

    cpu.set_flags(
        (a == 0) as i8,
        -1,
        0,
        ((correction << 2) & 0x100 != 0) as i8,
    );
    cpu.set_register(&CPURegisterId::A, a as u16);
    1
}

pub fn halt(cpu: &mut LR35902CPU) -> u8 {
    cpu.halt = true;
    1
}

pub fn stop(cpu: &mut LR35902CPU) -> u8 {
    cpu.halt = true;
    1
}

pub fn ei(cpu: &mut LR35902CPU) -> u8 {
    cpu.enabling_ints = true;
    cpu.halt = false;
    1
}

pub fn di(cpu: &mut LR35902CPU) -> u8 {
    cpu.int_master = false;
    1
}
