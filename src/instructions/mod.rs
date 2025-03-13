mod ld;
mod arithmetics;
mod bit;
mod misc;
mod stack;
mod control;

use std::fmt;
use phf::phf_map;
use ld::*;
use arithmetics::*;
use bit::*;
use misc::*;
use stack::*;
use control::*;

use crate::cpu::LR35902CPU;

#[derive(Debug)]
pub enum AddrMode {
    R,
    R_R,
    R_IMM,
    R_RADDR,
    R_IMMADDR,
    RADDR,
    RADDR_R,
    RADDR_IMM,
    R16,
    R16_R16,
    R16_SIMM,
    R16_IMM16,
    R16_R16_IMM,
    IMM_R,
    IMMADDR,
    IMM_RADDR,
    IMMADDR_R,
    IMMADDR_R16,
}


enum CondType {
    Nz,
    Z,
    Nc,
    C,
}

#[derive(PartialEq, PartialOrd)]
pub enum CPURegister {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    PC,
    SP,
}

impl fmt::Display for CPURegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CPURegister::A => write!(f, "A"),
            CPURegister::F => write!(f, "F"),
            CPURegister::B => write!(f, "B"),
            CPURegister::C => write!(f, "C"),
            CPURegister::D => write!(f, "D"),
            CPURegister::E => write!(f, "E"),
            CPURegister::H => write!(f, "H"),
            CPURegister::L => write!(f, "L"),
            CPURegister::PC => write!(f, "PC"),
            CPURegister::SP => write!(f, "SP"),
            CPURegister::AF => write!(f, "AF"),
            CPURegister::BC => write!(f, "BC"),
            CPURegister::DE => write!(f, "DE"),
            CPURegister::HL => write!(f, "HL"),
        }
    }
}

type FnType = fn(&mut LR35902CPU) -> ();

pub struct Instruction {
    pub name: &'static str,
    pub addr_mode: AddrMode,
    pub func: FnType,
    condition: Option<CondType>,
    pub reg1: Option<CPURegister>,
    pub reg2: Option<CPURegister>,
}

impl Instruction {
    const fn default() -> Self {
        Self {
            name: "",
            addr_mode: AddrMode::R_R,
            func: |_| {},
            condition: None,
            reg1: None,
            reg2: None,
        }
    }
}


pub static INSTRUCTIONS: phf::Map<u16, Instruction> = phf_map! {
        0x00u16 => Instruction { 
            name: "NOP",
            addr_mode: AddrMode::R_R,
            func: |_| {},
            ..Instruction::default()
        },
        0x01u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R16_IMM16,
            func: ldr,
            reg1: Some(CPURegister::BC),
            reg2: None,
            ..Instruction::default()
        },
        0x02u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::RADDR_R,
            func: ldm,
            reg1: Some(CPURegister::BC),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x03u16 => Instruction { 
            name: "INC",
            addr_mode: AddrMode::R16,
            func: inc,
            reg1: Some(CPURegister::BC),
            ..Instruction::default()
        },
        0x04u16 => Instruction { 
            name: "INC",
            addr_mode: AddrMode::R,
            func: inc,
            reg1: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x05u16 => Instruction { 
            name: "DEC",
            addr_mode: AddrMode::R,
            func: dec,
            reg1: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x06u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_IMM,
            func: ldr,
            reg1: Some(CPURegister::B),
            reg2: None,
            ..Instruction::default()
        },
        0x07u16 => Instruction { 
            name: "RLCA",
            addr_mode: AddrMode::R,
            func: rlca,
            ..Instruction::default()
        },
        0x08u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::IMMADDR_R16,
            func: ldm,
            reg1: None,
            reg2: Some(CPURegister::SP),
            ..Instruction::default()
        },
        0x09u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R16_R16,
            func: add,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::BC),
            ..Instruction::default()
        },
        0x0Au16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_RADDR,
            func: ldr,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::BC),
            ..Instruction::default()
        },
        0x0Bu16 => Instruction { 
            name: "DEC",
            addr_mode: AddrMode::R16,
            func: dec,
            reg1: Some(CPURegister::BC),
            ..Instruction::default()
        },
        0x0Cu16 => Instruction { 
            name: "INC",
            addr_mode: AddrMode::R,
            func: inc,
            reg1: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x0Du16 => Instruction { 
            name: "DEC",
            addr_mode: AddrMode::R,
            func: dec,
            reg1: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x0Eu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_IMM,
            func: ldr,
            reg1: Some(CPURegister::C),
            reg2: None,
            ..Instruction::default()
        },
        0x0Fu16 => Instruction { 
            name: "RRCA",
            addr_mode: AddrMode::R,
            func: rrca,
            ..Instruction::default()
        },
        0x10u16 => Instruction { 
            name: "STOP",
            addr_mode: AddrMode::R,
            func: stop,
            reg1: None,
            ..Instruction::default()
        },
        0x11u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R16_IMM16,
            func: ldr,
            reg1: Some(CPURegister::DE),
            reg2: None,
            ..Instruction::default()
        },
        0x12u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::RADDR_R,
            func: ldm,
            reg1: Some(CPURegister::DE),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x13u16 => Instruction { 
            name: "INC",
            addr_mode: AddrMode::R16,
            func: inc,
            reg1: Some(CPURegister::DE),
            ..Instruction::default()
        },
        0x14u16 => Instruction { 
            name: "INC",
            addr_mode: AddrMode::R,
            func: inc,
            reg1: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x15u16 => Instruction { 
            name: "DEC",
            addr_mode: AddrMode::R,
            func: dec,
            reg1: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x16u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_IMM,
            func: ldr,
            reg1: Some(CPURegister::D),
            reg2: None,
            ..Instruction::default()
        },
        0x17u16 => Instruction { 
            name: "RLA",
            addr_mode: AddrMode::R,
            func: rla,
            ..Instruction::default()
        },
        0x18u16 => Instruction { 
            name: "JR",
            addr_mode: AddrMode::R,
            func: jr,
            reg1: None,
            ..Instruction::default()
        },
        0x19u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R16_R16,
            func: add,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::DE),
            ..Instruction::default()
        },
        0x1Au16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_RADDR,
            func: ldr,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::DE),
            ..Instruction::default()
        },
        0x1Bu16 => Instruction { 
            name: "DEC",
            addr_mode: AddrMode::R16,
            func: dec,
            reg1: Some(CPURegister::DE),
            ..Instruction::default()
        },
        0x1Cu16 => Instruction { 
            name: "INC",
            addr_mode: AddrMode::R,
            func: inc,
            reg1: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x1Du16 => Instruction { 
            name: "DEC",
            addr_mode: AddrMode::R,
            func: dec,
            reg1: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x1Eu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_IMM,
            func: ldr,
            reg1: Some(CPURegister::E),
            reg2: None,
            ..Instruction::default()
        },
        0x1Fu16 => Instruction { 
            name: "RRA",
            addr_mode: AddrMode::R,
            func: rra,
            ..Instruction::default()
        },
        0x20u16 => Instruction { 
            name: "JR",
            addr_mode: AddrMode::R,
            func: jr,
            condition: Some(CondType::Nz),
            ..Instruction::default()
        },
        0x21u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R16_IMM16,
            func: ldr,
            reg1: Some(CPURegister::HL),
            reg2: None,
            ..Instruction::default()
        },
        0x22u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::RADDR_R,
            func: ldi,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x23u16 => Instruction { 
            name: "INC",
            addr_mode: AddrMode::R16,
            func: inc,
            reg1: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x24u16 => Instruction { 
            name: "INC",
            addr_mode: AddrMode::R,
            func: inc,
            reg1: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x25u16 => Instruction { 
            name: "DEC",
            addr_mode: AddrMode::R,
            func: dec,
            reg1: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x26u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_IMM,
            func: ldr,
            reg1: Some(CPURegister::H),
            reg2: None,
            ..Instruction::default()
        },
        0x27u16 => Instruction { 
            name: "DAA",
            addr_mode: AddrMode::R,
            func: daa,
            ..Instruction::default()
        },
        0x28u16 => Instruction { 
            name: "JR",
            addr_mode: AddrMode::R,
            func: jr,
            condition: Some(CondType::Z),
            ..Instruction::default()
        },
        0x29u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R16_R16,
            func: add,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x2Au16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_RADDR,
            func: ldi,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x2Bu16 => Instruction { 
            name: "DEC",
            addr_mode: AddrMode::R16,
            func: dec,
            reg1: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x2Cu16 => Instruction { 
            name: "INC",
            addr_mode: AddrMode::R,
            func: inc,
            reg1: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x2Du16 => Instruction { 
            name: "DEC",
            addr_mode: AddrMode::R,
            func: dec,
            reg1: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x2Eu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_IMM,
            func: ldr,
            reg1: Some(CPURegister::L),
            reg2: None,
            ..Instruction::default()
        },
        0x2Fu16 => Instruction { 
            name: "CPL",
            addr_mode: AddrMode::R,
            func: cpl,
            ..Instruction::default()
        },
        0x30u16 => Instruction { 
            name: "JR",
            addr_mode: AddrMode::R,
            func: jr,
            condition: Some(CondType::Nc),
            ..Instruction::default()
        },
        0x31u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R16_IMM16,
            func: ldr,
            reg1: Some(CPURegister::SP),
            reg2: None,
            ..Instruction::default()
        },
        0x32u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::RADDR_R,
            func: ldd,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x33u16 => Instruction { 
            name: "INC",
            addr_mode: AddrMode::R16,
            func: inc,
            reg1: Some(CPURegister::SP),
            ..Instruction::default()
        },
        0x34u16 => Instruction { 
            name: "INC",
            addr_mode: AddrMode::RADDR,
            func: inc,
            reg1: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x35u16 => Instruction { 
            name: "DEC",
            addr_mode: AddrMode::RADDR,
            func: dec,
            reg1: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x36u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::RADDR_IMM,
            func: ldm,
            reg1: Some(CPURegister::HL),
            reg2: None,
            ..Instruction::default()
        },
        0x37u16 => Instruction { 
            name: "SCF",
            addr_mode: AddrMode::R,
            func: scf,
            ..Instruction::default()
        },
        0x38u16 => Instruction { 
            name: "JR",
            addr_mode: AddrMode::R,
            func: jr,
            condition: Some(CondType::C),
            ..Instruction::default()
        },
        0x39u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R16_R16,
            func: add,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::SP),
            ..Instruction::default()
        },
        0x3Au16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_RADDR,
            func: ldd,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x3Bu16 => Instruction { 
            name: "DEC",
            addr_mode: AddrMode::R16,
            func: dec,
            reg1: Some(CPURegister::SP),
            ..Instruction::default()
        },
        0x3Cu16 => Instruction { 
            name: "INC",
            addr_mode: AddrMode::R,
            func: inc,
            reg1: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x3Du16 => Instruction { 
            name: "DEC",
            addr_mode: AddrMode::R,
            func: dec,
            reg1: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x3Eu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_IMM,
            func: ldr,
            reg1: Some(CPURegister::A),
            reg2: None,
            ..Instruction::default()
        },
        0x3Fu16 => Instruction { 
            name: "CCF",
            addr_mode: AddrMode::R,
            func: ccf,
            ..Instruction::default()
        },
        0x40u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::B),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x41u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::B),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x42u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::B),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x43u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::B),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x44u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::B),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x45u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::B),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x46u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_RADDR,
            func: ldr,
            reg1: Some(CPURegister::B),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x47u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::B),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x48u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::C),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x49u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::C),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x4Au16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::C),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x4Bu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::C),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x4Cu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::C),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x4Du16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::C),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x4Eu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_RADDR,
            func: ldr,
            reg1: Some(CPURegister::C),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x4Fu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::C),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x50u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::D),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x51u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::D),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x52u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::D),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x53u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::D),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x54u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::D),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x55u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::D),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x56u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_RADDR,
            func: ldr,
            reg1: Some(CPURegister::D),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x57u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::D),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x58u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::E),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x59u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::E),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x5Au16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::E),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x5Bu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::E),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x5Cu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::E),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x5Du16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::E),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x5Eu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_RADDR,
            func: ldr,
            reg1: Some(CPURegister::E),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x5Fu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::E),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x60u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::H),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x61u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::H),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x62u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::H),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x63u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::H),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x64u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::H),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x65u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::H),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x66u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_RADDR,
            func: ldr,
            reg1: Some(CPURegister::H),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x67u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::H),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x68u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::L),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x69u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::L),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x6Au16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::L),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x6Bu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::L),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x6Cu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::L),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x6Du16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::L),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x6Eu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_RADDR,
            func: ldr,
            reg1: Some(CPURegister::L),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x6Fu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::L),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x70u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::RADDR_R,
            func: ldm,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x71u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::RADDR_R,
            func: ldm,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x72u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::RADDR_R,
            func: ldm,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x73u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::RADDR_R,
            func: ldm,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x74u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::RADDR_R,
            func: ldm,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x75u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::RADDR_R,
            func: ldm,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x76u16 => Instruction { 
            name: "HALT",
            addr_mode: AddrMode::R,
            func: halt,
            ..Instruction::default()
        },
        0x77u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::RADDR_R,
            func: ldm,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x78u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x79u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x7Au16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x7Bu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x7Cu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x7Du16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x7Eu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_RADDR,
            func: ldr,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x7Fu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_R,
            func: ldr,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x80u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R_R,
            func: add,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x81u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R_R,
            func: add,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x82u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R_R,
            func: add,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x83u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R_R,
            func: add,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x84u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R_R,
            func: add,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x85u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R_R,
            func: add,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x86u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R_RADDR,
            func: add,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x87u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R_R,
            func: add,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x88u16 => Instruction { 
            name: "ADC",
            addr_mode: AddrMode::R_R,
            func: adc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x89u16 => Instruction { 
            name: "ADC",
            addr_mode: AddrMode::R_R,
            func: adc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x8Au16 => Instruction { 
            name: "ADC",
            addr_mode: AddrMode::R_R,
            func: adc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x8Bu16 => Instruction { 
            name: "ADC",
            addr_mode: AddrMode::R_R,
            func: adc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x8Cu16 => Instruction { 
            name: "ADC",
            addr_mode: AddrMode::R_R,
            func: adc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x8Du16 => Instruction { 
            name: "ADC",
            addr_mode: AddrMode::R_R,
            func: adc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x8Eu16 => Instruction { 
            name: "ADC",
            addr_mode: AddrMode::R_RADDR,
            func: adc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x8Fu16 => Instruction { 
            name: "ADC",
            addr_mode: AddrMode::R_R,
            func: adc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x90u16 => Instruction { 
            name: "SUB",
            addr_mode: AddrMode::R_R,
            func: sub,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x91u16 => Instruction { 
            name: "SUB",
            addr_mode: AddrMode::R_R,
            func: sub,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x92u16 => Instruction { 
            name: "SUB",
            addr_mode: AddrMode::R_R,
            func: sub,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x93u16 => Instruction { 
            name: "SUB",
            addr_mode: AddrMode::R_R,
            func: sub,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x94u16 => Instruction { 
            name: "SUB",
            addr_mode: AddrMode::R_R,
            func: sub,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x95u16 => Instruction { 
            name: "SUB",
            addr_mode: AddrMode::R_R,
            func: sub,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x96u16 => Instruction { 
            name: "SUB",
            addr_mode: AddrMode::R_RADDR,
            func: sub,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x97u16 => Instruction { 
            name: "SUB",
            addr_mode: AddrMode::R_R,
            func: sub,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0x98u16 => Instruction { 
            name: "SBC",
            addr_mode: AddrMode::R_R,
            func: sbc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0x99u16 => Instruction { 
            name: "SBC",
            addr_mode: AddrMode::R_R,
            func: sbc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0x9Au16 => Instruction { 
            name: "SBC",
            addr_mode: AddrMode::R_R,
            func: sbc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0x9Bu16 => Instruction { 
            name: "SBC",
            addr_mode: AddrMode::R_R,
            func: sbc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0x9Cu16 => Instruction { 
            name: "SBC",
            addr_mode: AddrMode::R_R,
            func: sbc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0x9Du16 => Instruction { 
            name: "SBC",
            addr_mode: AddrMode::R_R,
            func: sbc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0x9Eu16 => Instruction { 
            name: "SBC",
            addr_mode: AddrMode::R_RADDR,
            func: sbc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0x9Fu16 => Instruction { 
            name: "SBC",
            addr_mode: AddrMode::R_R,
            func: sbc,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0xA0u16 => Instruction { 
            name: "AND",
            addr_mode: AddrMode::R_R,
            func: and,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0xA1u16 => Instruction { 
            name: "AND",
            addr_mode: AddrMode::R_R,
            func: and,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0xA2u16 => Instruction { 
            name: "AND",
            addr_mode: AddrMode::R_R,
            func: and,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0xA3u16 => Instruction { 
            name: "AND",
            addr_mode: AddrMode::R_R,
            func: and,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0xA4u16 => Instruction { 
            name: "AND",
            addr_mode: AddrMode::R_R,
            func: and,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0xA5u16 => Instruction { 
            name: "AND",
            addr_mode: AddrMode::R_R,
            func: and,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0xA6u16 => Instruction { 
            name: "AND",
            addr_mode: AddrMode::R_RADDR,
            func: and,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0xA7u16 => Instruction { 
            name: "AND",
            addr_mode: AddrMode::R_R,
            func: and,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0xA8u16 => Instruction { 
            name: "XOR",
            addr_mode: AddrMode::R_R,
            func: xor,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0xA9u16 => Instruction { 
            name: "XOR",
            addr_mode: AddrMode::R_R,
            func: xor,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0xAAu16 => Instruction { 
            name: "XOR",
            addr_mode: AddrMode::R_R,
            func: xor,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0xABu16 => Instruction { 
            name: "XOR",
            addr_mode: AddrMode::R_R,
            func: xor,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0xACu16 => Instruction { 
            name: "XOR",
            addr_mode: AddrMode::R_R,
            func: xor,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0xADu16 => Instruction { 
            name: "XOR",
            addr_mode: AddrMode::R_R,
            func: xor,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0xAEu16 => Instruction { 
            name: "XOR",
            addr_mode: AddrMode::R_RADDR,
            func: xor,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0xAFu16 => Instruction { 
            name: "XOR",
            addr_mode: AddrMode::R_R,
            func: xor,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0xB0u16 => Instruction { 
            name: "OR",
            addr_mode: AddrMode::R_R,
            func: or,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0xB1u16 => Instruction { 
            name: "OR",
            addr_mode: AddrMode::R_R,
            func: or,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0xB2u16 => Instruction { 
            name: "OR",
            addr_mode: AddrMode::R_R,
            func: or,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0xB3u16 => Instruction { 
            name: "OR",
            addr_mode: AddrMode::R_R,
            func: or,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0xB4u16 => Instruction { 
            name: "OR",
            addr_mode: AddrMode::R_R,
            func: or,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0xB5u16 => Instruction { 
            name: "OR",
            addr_mode: AddrMode::R_R,
            func: or,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0xB6u16 => Instruction { 
            name: "OR",
            addr_mode: AddrMode::R_RADDR,
            func: or,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0xB7u16 => Instruction { 
            name: "OR",
            addr_mode: AddrMode::R_R,
            func: or,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0xB8u16 => Instruction { 
            name: "CP",
            addr_mode: AddrMode::R_R,
            func: cp,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::B),
            ..Instruction::default()
        },
        0xB9u16 => Instruction { 
            name: "CP",
            addr_mode: AddrMode::R_R,
            func: cp,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0xBAu16 => Instruction { 
            name: "CP",
            addr_mode: AddrMode::R_R,
            func: cp,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::D),
            ..Instruction::default()
        },
        0xBBu16 => Instruction { 
            name: "CP",
            addr_mode: AddrMode::R_R,
            func: cp,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::E),
            ..Instruction::default()
        },
        0xBCu16 => Instruction { 
            name: "CP",
            addr_mode: AddrMode::R_R,
            func: cp,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::H),
            ..Instruction::default()
        },
        0xBDu16 => Instruction { 
            name: "CP",
            addr_mode: AddrMode::R_R,
            func: cp,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::L),
            ..Instruction::default()
        },
        0xBEu16 => Instruction { 
            name: "CP",
            addr_mode: AddrMode::R_RADDR,
            func: cp,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0xBFu16 => Instruction { 
            name: "CP",
            addr_mode: AddrMode::R_R,
            func: cp,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0xC0u16 => Instruction { 
            name: "RET",
            addr_mode: AddrMode::R,
            func: ret,
            condition: Some(CondType::Nz),
            ..Instruction::default()
        },
        0xC1u16 => Instruction { 
            name: "POP",
            addr_mode: AddrMode::R16,
            func: pop,
            reg1: Some(CPURegister::BC),
            ..Instruction::default()
        },
        0xC2u16 => Instruction { 
            name: "JP",
            addr_mode: AddrMode::IMMADDR,
            func: jp,
            condition: Some(CondType::Nz),
            ..Instruction::default()
        },
        0xC3u16 => Instruction { 
            name: "JP",
            addr_mode: AddrMode::IMMADDR,
            func: jp,
            ..Instruction::default()
        },
        0xC4u16 => Instruction { 
            name: "CALL",
            addr_mode: AddrMode::IMMADDR,
            func: call,
            condition: Some(CondType::Nz),
            ..Instruction::default()
        },
        0xC5u16 => Instruction { 
            name: "PUSH",
            addr_mode: AddrMode::R16,
            func: push,
            reg1: Some(CPURegister::BC),
            ..Instruction::default()
        },
        0xC6u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R_IMM,
            func: add,
            reg1: Some(CPURegister::A),
            ..Instruction::default()
        },
        0xC7u16 => Instruction { 
            name: "RST",
            addr_mode: AddrMode::R,
            func: |cpu: &mut LR35902CPU| { rst(cpu, 0x00); },
            ..Instruction::default()
        },
        0xC8u16 => Instruction { 
            name: "RET",
            addr_mode: AddrMode::R,
            func: ret,
            condition: Some(CondType::Z),
            ..Instruction::default()
        },
        0xC9u16 => Instruction { 
            name: "RET",
            addr_mode: AddrMode::R,
            func: ret,
            ..Instruction::default()
        },
        0xCAu16 => Instruction { 
            name: "JP",
            addr_mode: AddrMode::IMMADDR,
            func: jp,
            condition: Some(CondType::Z),
            ..Instruction::default()
        },
        0xCCu16 => Instruction { 
            name: "CALL",
            addr_mode: AddrMode::IMMADDR,
            func: call,
            condition: Some(CondType::Z),
            ..Instruction::default()
        },
        0xCDu16 => Instruction { 
            name: "CALL",
            addr_mode: AddrMode::IMMADDR,
            func: call,
            reg1: None,
            ..Instruction::default()
        },
        0xCEu16 => Instruction { 
            name: "ADC",
            addr_mode: AddrMode::R_IMM,
            func: adc,
            reg1: Some(CPURegister::A),
            ..Instruction::default()
        },
        0xCFu16 => Instruction { 
            name: "RST",
            addr_mode: AddrMode::R_R,
            func: |cpu: &mut LR35902CPU| { rst(cpu, 0x08); },
            reg1: None,
            ..Instruction::default()
        },
        0xD0u16 => Instruction { 
            name: "RET",
            addr_mode: AddrMode::R,
            func: ret,
            condition: Some(CondType::Nc),
            ..Instruction::default()
        },
        0xD1u16 => Instruction { 
            name: "POP",
            addr_mode: AddrMode::R16,
            func: pop,
            reg1: Some(CPURegister::DE),
            ..Instruction::default()
        },
        0xD2u16 => Instruction { 
            name: "JP",
            addr_mode: AddrMode::IMMADDR,
            func: jp,
            condition: Some(CondType::Nc),
            ..Instruction::default()
        },
        0xD4u16 => Instruction { 
            name: "CALL",
            addr_mode: AddrMode::IMMADDR,
            func: call,
            condition: Some(CondType::Nc),
            ..Instruction::default()
        },
        0xD5u16 => Instruction { 
            name: "PUSH",
            addr_mode: AddrMode::R16,
            func: push,
            reg1: Some(CPURegister::DE),
            ..Instruction::default()
        },
        0xD6u16 => Instruction { 
            name: "SUB",
            addr_mode: AddrMode::R_IMM,
            func: sub,
            reg1: Some(CPURegister::A),
            reg2: None,
            ..Instruction::default()
        },
        0xD7u16 => Instruction { 
            name: "RST",
            addr_mode: AddrMode::R,
            func: |cpu: &mut LR35902CPU| { rst(cpu, 0x10); },
            ..Instruction::default()
        },
        0xD8u16 => Instruction { 
            name: "RET",
            addr_mode: AddrMode::R,
            func: ret,
            reg1: Some(CPURegister::C),
            ..Instruction::default()
        },
        0xD9u16 => Instruction { 
            name: "RETI",
            addr_mode: AddrMode::R,
            func: reti,
            ..Instruction::default()
        },
        0xDAu16 => Instruction { 
            name: "JP",
            addr_mode: AddrMode::IMMADDR,
            func: jp,
            condition: Some(CondType::C),
            ..Instruction::default()
        },
        0xDCu16 => Instruction { 
            name: "CALL",
            addr_mode: AddrMode::IMMADDR,
            func: call,
            condition: Some(CondType::C),
            ..Instruction::default()
        },
        0xDEu16 => Instruction { 
            name: "SBC",
            addr_mode: AddrMode::R_IMM,
            func: sbc,
            reg1: Some(CPURegister::A),
            ..Instruction::default()
        },
        0xDFu16 => Instruction { 
            name: "RST",
            addr_mode: AddrMode::R_R,
            func: |cpu: &mut LR35902CPU| { rst(cpu, 0x18); },
            reg1: None,
            ..Instruction::default()
        },
        0xE0u16 => Instruction { 
            name: "LDH",
            addr_mode: AddrMode::IMMADDR_R,
            func: ldh,
            reg1: None,
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0xE1u16 => Instruction { 
            name: "POP",
            addr_mode: AddrMode::R16,
            func: pop,
            reg1: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0xE2u16 => Instruction { 
            name: "LDH",
            addr_mode: AddrMode::RADDR_R,
            func: ldh,
            reg1: Some(CPURegister::C),
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0xE5u16 => Instruction { 
            name: "PUSH",
            addr_mode: AddrMode::R16,
            func: push,
            reg1: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0xE6u16 => Instruction { 
            name: "AND",
            addr_mode: AddrMode::R_IMM,
            func: and,
            reg1: Some(CPURegister::A),
            reg2: None,
            ..Instruction::default()
        },
        0xE7u16 => Instruction { 
            name: "RST",
            addr_mode: AddrMode::R_R,
            func: |cpu: &mut LR35902CPU| { rst(cpu, 0x20); },
            reg1: None,
            ..Instruction::default()
        },
        0xE8u16 => Instruction { 
            name: "ADD",
            addr_mode: AddrMode::R16_SIMM,
            func: add,
            reg1: Some(CPURegister::SP),
            reg2: None,
            ..Instruction::default()
        },
        0xE9u16 => Instruction { 
            name: "JP",
            addr_mode: AddrMode::R16,
            func: jp,
            reg1: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0xEAu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::IMMADDR_R,
            func: ldm,
            reg1: None,
            reg2: Some(CPURegister::A),
            ..Instruction::default()
        },
        0xEEu16 => Instruction { 
            name: "XOR",
            addr_mode: AddrMode::R_IMM,
            func: xor,
            reg1: Some(CPURegister::A),
            reg2: None,
            ..Instruction::default()
        },
        0xEFu16 => Instruction { 
            name: "RST",
            addr_mode: AddrMode::R,
            func: |cpu: &mut LR35902CPU| { rst(cpu, 0x28); },
            reg1: None,
            ..Instruction::default()
        },
        0xF0u16 => Instruction { 
            name: "LDH",
            addr_mode: AddrMode::R_IMMADDR,
            func: ldh,
            reg1: Some(CPURegister::A),
            reg2: None,
            ..Instruction::default()
        },
        0xF1u16 => Instruction { 
            name: "POP",
            addr_mode: AddrMode::R16,
            func: pop,
            reg1: Some(CPURegister::AF),
            ..Instruction::default()
        },
        0xF2u16 => Instruction { 
            name: "LDH",
            addr_mode: AddrMode::R_RADDR,
            func: ldh,
            reg1: Some(CPURegister::A),
            reg2: Some(CPURegister::C),
            ..Instruction::default()
        },
        0xF3u16 => Instruction { 
            name: "DI",
            addr_mode: AddrMode::R_R,
            func: di,
            ..Instruction::default()
        },
        0xF5u16 => Instruction { 
            name: "PUSH",
            addr_mode: AddrMode::R16,
            func: push,
            reg1: Some(CPURegister::AF),
            ..Instruction::default()
        },
        0xF6u16 => Instruction { 
            name: "OR",
            addr_mode: AddrMode::R_IMM,
            func: or,
            reg1: Some(CPURegister::A),
            reg2: None,
            ..Instruction::default()
        },
        0xF7u16 => Instruction { 
            name: "RST",
            addr_mode: AddrMode::R_R,
            func: |cpu: &mut LR35902CPU| { rst(cpu, 0x30); },
            reg1: None,
            ..Instruction::default()
        },
        0xF8u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R16_R16_IMM,
            func: ldr,
            reg1: Some(CPURegister::HL),
            reg2: Some(CPURegister::SP),
            ..Instruction::default()
        },
        0xF9u16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R16_R16,
            func: ldr,
            reg1: Some(CPURegister::SP),
            reg2: Some(CPURegister::HL),
            ..Instruction::default()
        },
        0xFAu16 => Instruction { 
            name: "LD",
            addr_mode: AddrMode::R_IMMADDR,
            func: ldr,
            reg1: Some(CPURegister::A),
            reg2: None,
            ..Instruction::default()
        },
        0xFBu16 => Instruction { 
            name: "EI",
            addr_mode: AddrMode::R_R,
            func: ei,
            ..Instruction::default()
        },
        0xFEu16 => Instruction { 
            name: "CP",
            addr_mode: AddrMode::R_IMM,
            func: cp,
            reg1: Some(CPURegister::A),
            reg2: None,
            ..Instruction::default()
        },
        0xFFu16 => Instruction { 
            name: "RST",
            addr_mode: AddrMode::R_R,
            func: |cpu: &mut LR35902CPU| { rst(cpu, 0x30); },
            reg1: None,
            ..Instruction::default()
        },
    };
