use std::time::Instant;

use super::clock::Clock;
use super::instructions::{stack::_push, CPURegisterId, Instruction, INSTRUCTIONS};
use super::interrupts::{InterruptFlags, INTERRUPT_ENABLE, INTERRUPT_FLAGS};
use crate::core::mem::bus::Bus;
use crate::dbg::print_serial;
use crate::debugger::{CpuMetricFields, CPU_METRICS};
use crate::flag_set;

#[allow(nonstandard_style)]
pub mod CPUFlags {
    pub const Z: u8 = 0x80;
    pub const N: u8 = 0x40;
    pub const H: u8 = 0x20;
    pub const C: u8 = 0x10;
}

#[derive(Clone)]
pub struct CPURegisters {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    pub pc: u16,
    pub sp: u16,
}

impl CPURegisters {
    pub fn new(pc: u16) -> Self {
        Self {
            a: 0x01,
            f: 0xb0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xd8,
            h: 0x01,
            l: 0x4d,

            sp: 0xfffe,
            pc,
        }
    }
}

pub struct LR35902CPU {
    pub registers: CPURegisters,
    serial: bool,

    pub bus: Bus,

    pub current_instruction: &'static Instruction,
    pub halt: bool,
    pub int_master: bool,
    pub enabling_ints: bool,

    pub clock: Clock,
}

impl LR35902CPU {
    pub fn new(bus: Bus, serial: bool, clock_speed: u32) -> Self {
        let pc = if bus.booting { 0 } else { 0x100 };
        Self {
            bus,
            serial,
            current_instruction: &INSTRUCTIONS[&0],
            halt: false,
            registers: CPURegisters::new(pc),
            int_master: false,
            enabling_ints: false,
            clock: Clock::new(clock_speed),
        }
    }

    pub fn tick(&mut self) -> u8 {
        let mut cycles: u8 = 1;

        if !self.halt {
            self.set_instruction();

            if self.serial {
                print_serial(self);
            }

            cycles = (self.current_instruction.func)(self);
            CPU_METRICS.with_borrow_mut(|mh| mh.count(CpuMetricFields::INSTRUCTIONS, 1));
        }

        if INTERRUPT_FLAGS.get() > 0 {
            // Interrupt pending, wake up
            self.halt = false;
        }

        if self.int_master {
            self.handle_ints();
        }

        if self.enabling_ints {
            // Enable IME for next tick
            self.enabling_ints = false;
            self.int_master = true;
        }

        CPU_METRICS.with_borrow_mut(|mh| mh.count(CpuMetricFields::CYCLES, cycles as u32));
        cycles
    }

    pub fn step(&mut self) {
        let start = Instant::now();
        let cycles = self.tick();
        self.bus.io.timer.tick(cycles);
        self.bus.dma_tick(cycles);
        self.bus.io.ppu.tick(cycles);

        CPU_METRICS.with_borrow_mut(|mh| {
            mh.mean_time(
                CpuMetricFields::TICK_TIME,
                (Instant::now() - start) / (cycles as u32 * 4),
            )
        });

        self.clock.tick(cycles as u32 * 4);
    }

    pub fn set_register(&mut self, register: &CPURegisterId, value: u16) {
        match register {
            CPURegisterId::A => {
                self.registers.a = value as u8;
            }
            CPURegisterId::F => {
                self.registers.f = value as u8;
            }
            CPURegisterId::B => {
                self.registers.b = value as u8;
            }
            CPURegisterId::C => {
                self.registers.c = value as u8;
            }
            CPURegisterId::D => {
                self.registers.d = value as u8;
            }
            CPURegisterId::E => {
                self.registers.e = value as u8;
            }
            CPURegisterId::H => {
                self.registers.h = value as u8;
            }
            CPURegisterId::L => {
                self.registers.l = value as u8;
            }
            CPURegisterId::AF => {
                self.registers.a = (value >> 8) as u8;
                self.registers.f = (value & 0xff) as u8;
            }
            CPURegisterId::BC => {
                self.registers.b = (value >> 8) as u8;
                self.registers.c = (value & 0xff) as u8;
            }
            CPURegisterId::DE => {
                self.registers.d = (value >> 8) as u8;
                self.registers.e = (value & 0xff) as u8;
            }
            CPURegisterId::HL => {
                self.registers.h = (value >> 8) as u8;
                self.registers.l = (value & 0xff) as u8;
            }
            CPURegisterId::PC => {
                self.registers.pc = value;
            }
            CPURegisterId::SP => {
                self.registers.sp = value;
            }
        }
    }

    pub fn get_register(&self, register: &CPURegisterId) -> u8 {
        match register {
            CPURegisterId::A => self.registers.a,
            CPURegisterId::F => self.registers.f,
            CPURegisterId::B => self.registers.b,
            CPURegisterId::C => self.registers.c,
            CPURegisterId::D => self.registers.d,
            CPURegisterId::E => self.registers.e,
            CPURegisterId::H => self.registers.h,
            CPURegisterId::L => self.registers.l,
            _ => unreachable!(),
        }
    }

    pub fn get_register16(&self, register: &CPURegisterId) -> u16 {
        match register {
            CPURegisterId::AF => ((self.registers.a as u16) << 8) | self.registers.f as u16,
            CPURegisterId::BC => ((self.registers.b as u16) << 8) | self.registers.c as u16,
            CPURegisterId::DE => ((self.registers.d as u16) << 8) | self.registers.e as u16,
            CPURegisterId::HL => ((self.registers.h as u16) << 8) | self.registers.l as u16,
            CPURegisterId::PC => self.registers.pc,
            CPURegisterId::SP => self.registers.sp,
            _ => unreachable!(),
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
        flag_set!(self.registers.f, flag) as u8
    }

    pub fn set_flags(&mut self, z: i8, n: i8, h: i8, c: i8) {
        if z != -1 {
            self.registers.f = (self.registers.f & !CPUFlags::Z) | (CPUFlags::Z * z as u8);
        }

        if n != -1 {
            self.registers.f = (self.registers.f & !CPUFlags::N) | (CPUFlags::N * n as u8);
        }

        if h != -1 {
            self.registers.f = (self.registers.f & !CPUFlags::H) | (CPUFlags::H * h as u8);
        }

        if c != -1 {
            self.registers.f = (self.registers.f & !CPUFlags::C) | (CPUFlags::C * c as u8);
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

    fn handle_ints(&mut self) {
        let mut handle_int = |int: u8, addr: u16| -> bool {
            if (INTERRUPT_FLAGS.get() & int) == int && (INTERRUPT_ENABLE.get() & int) == int {
                _push(self, (self.registers.pc >> 8) as u8);
                _push(self, (self.registers.pc & 0xff) as u8);
                INTERRUPT_FLAGS.set(INTERRUPT_FLAGS.get() ^ int);
                self.halt = false;
                self.int_master = false;
                self.registers.pc = addr;
                return true;
            }
            false
        };

        for (int, addr) in [
            (InterruptFlags::VBLANK, 0x40),
            (InterruptFlags::STAT, 0x48),
            (InterruptFlags::TIMER, 0x50),
            (InterruptFlags::SERIAL, 0x58),
            (InterruptFlags::JOYPAD, 0x60),
        ] {
            if handle_int(int as u8, addr) {
                return;
            }
        }
    }
}
