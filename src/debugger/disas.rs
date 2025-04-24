use std::fmt;

use crate::core::cpu::instructions::{
    AddrMode::*, CPURegisterId, CondType, Instruction, INSTRUCTIONS,
};
use crate::core::cpu::LR35902CPU;

pub struct GbAsm {
    pub addr: u16,
    pub asm: String,
}

impl fmt::Display for CPURegisterId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CPURegisterId::A => write!(f, "A"),
            CPURegisterId::F => write!(f, "F"),
            CPURegisterId::B => write!(f, "B"),
            CPURegisterId::C => write!(f, "C"),
            CPURegisterId::D => write!(f, "D"),
            CPURegisterId::E => write!(f, "E"),
            CPURegisterId::H => write!(f, "H"),
            CPURegisterId::L => write!(f, "L"),
            CPURegisterId::PC => write!(f, "PC"),
            CPURegisterId::SP => write!(f, "SP"),
            CPURegisterId::AF => write!(f, "AF"),
            CPURegisterId::BC => write!(f, "BC"),
            CPURegisterId::DE => write!(f, "DE"),
            CPURegisterId::HL => write!(f, "HL"),
        }
    }
}

impl fmt::Display for CondType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CondType::Z => write!(f, "Z, "),
            CondType::C => write!(f, "C, "),
            CondType::Nz => write!(f, "NZ, "),
            CondType::Nc => write!(f, "NC, "),
        }
    }
}

impl Instruction {
    fn to_string(&self, cpu: &LR35902CPU, addr: u16) -> String {
        let name = self.name;

        let cond = self
            .condition
            .as_ref()
            .map_or(String::new(), |c| c.to_string());

        let args = match self.addr_mode {
            R => {
                if self.reg1.is_some() {
                    self.reg1.as_ref().unwrap().to_string()
                } else {
                    String::new()
                }
            }
            R_R => {
                if self.reg1.is_some() {
                    format!(
                        "{}, {}",
                        self.reg1.as_ref().unwrap(),
                        self.reg2.as_ref().unwrap()
                    )
                } else {
                    String::new()
                }
            }
            R_IMM => format!(
                "{}, $0x{:02X}",
                self.reg1.as_ref().unwrap(),
                cpu.bus.read(addr)
            ),
            R_RADDR => format!(
                "{}, [{}]",
                self.reg1.as_ref().unwrap(),
                self.reg2.as_ref().unwrap()
            ),
            R_IMMADDR => format!(
                "{}, [0x{:04X}]",
                self.reg1.as_ref().unwrap(),
                cpu.bus.read16(addr)
            ),
            RADDR => format!("[{}]", self.reg1.as_ref().unwrap()),
            RADDR_R => format!(
                "[{}], {}",
                self.reg1.as_ref().unwrap(),
                self.reg2.as_ref().unwrap()
            ),
            RADDR_IMM => format!(
                "[{}], $0x{:02X}",
                self.reg1.as_ref().unwrap(),
                cpu.bus.read(addr)
            ),
            R16 => format!("{}", self.reg1.as_ref().unwrap()),
            R16_R16 => format!(
                "{}, {}",
                self.reg1.as_ref().unwrap(),
                self.reg2.as_ref().unwrap()
            ),
            R16_SIMM => format!(
                "{}, $0x{:02X}",
                self.reg1.as_ref().unwrap(),
                cpu.bus.read(addr) as i8
            ),
            R16_IMM16 => format!(
                "{}, $0x{:04X}",
                self.reg1.as_ref().unwrap(),
                cpu.bus.read16(addr)
            ),
            R16_R16_IMM => format!(
                "{}, {}, $0x{:02X}",
                self.reg1.as_ref().unwrap(),
                self.reg2.as_ref().unwrap(),
                cpu.bus.read(addr) as i8
            ),
            IMM_R => format!(
                "$0x{:02X}, {}",
                cpu.bus.read(addr),
                self.reg2.as_ref().unwrap()
            ),
            IMMADDR => format!("$0x{:04X}", cpu.bus.read16(addr)),
            IMM_RADDR => format!(
                "$0x{:02X}, [{}]",
                cpu.bus.read(addr),
                self.reg2.as_ref().unwrap()
            ),
            IMMADDR_R => format!(
                "[0x{:04X}], {}",
                cpu.bus.read16(addr),
                self.reg2.as_ref().unwrap()
            ),
            IMMADDR_R16 => format!(
                "[0x{:04X}], {}",
                cpu.bus.read16(addr),
                self.reg2.as_ref().unwrap()
            ),
            SIMM => format!("$0x{:02X}", cpu.bus.read(cpu.pc()) as i8),
        };

        format!("{name} {cond}{args}")
    }
}

fn get_instr_size(instr: &Instruction) -> u16 {
    match instr.addr_mode {
        R | R16 | R_R | R_RADDR | RADDR | RADDR_R | R16_R16 => 1,
        R_IMM | R16_SIMM | RADDR_IMM | R16_R16_IMM | IMM_R | IMM_RADDR | SIMM => 2,
        R_IMMADDR | R16_IMM16 | IMMADDR | IMMADDR_R | IMMADDR_R16 => 3,
    }
}

fn disas_at(cpu: &LR35902CPU, addr: u16) -> (u16, GbAsm) {
    let mut opcode: u16 = cpu.bus.read(addr) as u16;
    let mut prefixed = 0;

    if opcode == 0xcb {
        prefixed = 1;
        opcode = (1 << 8) | cpu.bus.read(addr + 1) as u16;
    }

    if !INSTRUCTIONS.contains_key(&opcode) {
        return (
            1,
            GbAsm {
                addr,
                asm: String::from("??"),
            },
        );
    }

    let instr = &INSTRUCTIONS[&opcode];

    let size = get_instr_size(instr) + prefixed;

    (
        size,
        GbAsm {
            addr,
            asm: instr.to_string(cpu, addr + prefixed + 1),
        },
    )
}

pub fn disas(cpu: &LR35902CPU, size: usize) -> Vec<GbAsm> {
    let mut ret = Vec::with_capacity(size);
    let mut addr = cpu.pc();

    for _ in 0..size {
        let (instr_size, asm) = disas_at(cpu, addr);
        addr += instr_size;

        ret.push(asm);
    }

    ret
}
