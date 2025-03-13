use super::instructions::{Instruction, CPURegister, INSTRUCTIONS};
use std::time::Instant;
use std::{thread, time};


use crate::dbg::print_state;
use crate::Bus;
use crate::utils::*;

#[repr(u8)]
pub enum CPUFlags {
    Z = 7u8,
    N = 6u8,
    H = 5u8,
    C = 4u8,
}

struct CPURegisters {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    pc: u16,
    sp: u16,
}

impl CPURegisters {
    pub fn new() -> Self {
        Self {
            a: 0xb0,
            f: 0x01,
            b: 0x13,
            c: 0x00,
            d: 0xd8,
            e: 0x00,
            h: 0x4d,
            l: 0x01,

            pc: 0x0100,
            sp: 0xfffe,
        }
    }
}

pub struct LR35902CPU {
    registers: CPURegisters,

    pub bus: Bus,

    elapsed_cycles: u8,
    pub current_instruction: &'static Instruction,
    pub halt: bool,
    pub interrupt_master_enabled: bool,
}

impl LR35902CPU {
    pub fn new(bus: Bus) -> Self {
        Self {
            bus: bus,
            elapsed_cycles: 0,
            current_instruction: &INSTRUCTIONS[&0],
            halt: false,
            registers: CPURegisters::new(),
            interrupt_master_enabled: false,
        }
    }

    pub fn run(&mut self) {
        loop {
            // let dl = Instant::now() + time::Duration::from_secs_f32(0.0002384185791015625);
            
            // self.elapsed_cycles = 0;
            self.set_instruction();
            print_state(self);
            
            (self.current_instruction.func)(self);

            // println!("0x{:04X} 0x{:02X}", self.pc(), self.bus.read(self.pc()));

            // thread::sleep(dl - Instant::now());
        }
    }

    pub fn set_register(&mut self, register: &CPURegister, value: u16) {
        match register {
            CPURegister::A => {self.registers.a = (value & 0xff) as u8;},
            CPURegister::F => {panic!("use cpu.set_flags dummy");},
            CPURegister::B => {self.registers.b = (value & 0xff) as u8;},
            CPURegister::C => {self.registers.c = (value & 0xff) as u8;},
            CPURegister::D => {self.registers.d = (value & 0xff) as u8;},
            CPURegister::E => {self.registers.e = (value & 0xff) as u8;},
            CPURegister::H => {self.registers.h = (value & 0xff) as u8;},
            CPURegister::L => {self.registers.l = (value & 0xff) as u8;},
            CPURegister::AF => {self.registers.a = (value >> 8) as u8;},
            CPURegister::BC => {
                self.registers.b = (value >> 8) as u8;
                self.registers.c = (value & 0xff) as u8;
            },
            CPURegister::DE => {
                self.registers.d = (value >> 8) as u8;
                self.registers.e = (value & 0xff) as u8;
            },
            CPURegister::HL => {
                self.registers.h = (value >> 8) as u8;
                self.registers.l = (value & 0xff) as u8;
            },
            CPURegister::PC => {self.registers.pc = value;},
            CPURegister::SP => {self.registers.sp = value;},
        }
    }

    pub fn get_register(&self, register: &CPURegister) -> u8 {
        match register {
            CPURegister::A => self.registers.a,
            CPURegister::F => self.registers.f,
            CPURegister::B => self.registers.b,
            CPURegister::C => self.registers.c,
            CPURegister::D => self.registers.d,
            CPURegister::E => self.registers.e,
            CPURegister::H => self.registers.h,
            CPURegister::L => self.registers.l,
            _ => panic!("use cpu.get_register16 dummy"),
        }
    }

    pub fn get_register16(&self, register: &CPURegister) -> u16 {
        match register {
            CPURegister::AF => ((self.registers.a as u16) << 8) | self.registers.f as u16,
            CPURegister::BC => ((self.registers.b as u16) << 8) | self.registers.c as u16,
            CPURegister::DE => ((self.registers.d as u16) << 8) | self.registers.e as u16,
            CPURegister::HL => ((self.registers.h as u16) << 8) | self.registers.l as u16,
            CPURegister::PC => self.registers.pc,
            CPURegister::SP => self.registers.sp,
            _ => panic!("use cpu.get_register dummy"),
        }
    }

    pub fn inc_pc(&mut self, inc: u16) {
        self.registers.pc += inc;
    }

    pub fn pc(&self) -> u16 {
        self.registers.pc
    }

    pub fn inc_sp(&mut self) {
        self.registers.sp += 1;
    }

    pub fn dec_sp(&mut self) {
        self.registers.sp -= 1;
    }

    pub fn sp(&self) -> u16 {
        self.registers.sp
    }

    pub fn get_flag(&self, flag: u8) -> u8 {
        get_bit(self.registers.f, flag)
    }

    pub fn set_flags(&mut self, z: i8, n: i8, h: i8, c: i8) {
        if z != -1 {
            self.registers.f = set_bit(self.registers.f, CPUFlags::Z as u8, z as u8);
        }

        if n != -1 {
            self.registers.f = set_bit(self.registers.f, CPUFlags::N as u8, n as u8);
        }

        if h != -1 {
            self.registers.f = set_bit(self.registers.f, CPUFlags::H as u8, h as u8);
        }

        if c != -1 {
            self.registers.f = set_bit(self.registers.f, CPUFlags::C as u8, c as u8);
        } 
    }

    pub fn set_instruction(&mut self) {
        let mut opcode: u16 = self.bus.read(self.registers.pc) as u16;
        self.registers.pc += 1;

        if opcode == 0xcb {
            opcode = (1 << 8) | self.bus.read(self.registers.pc) as u16;
            self.registers.pc += 1;
        }

        self.current_instruction = &INSTRUCTIONS[&opcode];
    }
}