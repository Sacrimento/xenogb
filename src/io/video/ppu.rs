use super::lcd::{PPUMode, LCD, LCDS_FLAGS};
use crate::interrupts::{request_interrupt, InterruptFlags};

const LINES_PER_FRAME: u8 = 154;
const TICKS_PER_LINE: u16 = 456;
const RESX: u16 = 160;
const RESY: u8 = 144;

#[allow(nonstandard_style)]
mod OAFlags {
    const CGB_PALETTE: u8 = 0x7;
    const BANK: u8 = 0x8;
    const DMG_PALETTE: u8 = 0x10;
    const X_FLIP: u8 = 0x20;
    const Y_FLIP: u8 = 0x40;
    const PRIORITY: u8 = 0x80;
}

struct OA {
    pub y: u8,
    pub x: u8,
    pub tile_idx: u8,
    pub flags: u8,
}

impl OA {
    fn new() -> Self {
        Self {
            y: 0,
            x: 0,
            tile_idx: 0,
            flags: 0,
        }
    }
}

pub struct PPU {
    oam_ram: Vec<OA>,
    vram: [u8; 0x2000],

    pub lcd: LCD,
    line_ticks: u16,
}

impl PPU {
    pub fn new() -> Self {
        let mut lcd = LCD::new();
        lcd.set_ppu_mode(PPUMode::OAMScan);

        Self {
            oam_ram: std::iter::repeat_with(|| OA::new())
                .take(40)
                .collect::<Vec<OA>>(),
            vram: [0; 0x2000],
            lcd,
            line_ticks: 0,
        }
    }

    pub fn oam_write(&mut self, mut addr: u16, value: u8) {
        if addr >= 0xfe00 {
            addr -= 0xfe00;
        }

        let offset = addr / std::mem::size_of::<OA>() as u16;
        match addr % std::mem::size_of::<OA>() as u16 {
            0 => self.oam_ram[offset as usize].y = value,
            1 => self.oam_ram[offset as usize].x = value,
            2 => self.oam_ram[offset as usize].tile_idx = value,
            3 => self.oam_ram[offset as usize].flags = value,
            _ => unreachable!(),
        }
    }

    pub fn oam_read(&self, mut addr: u16) -> u8 {
        if addr >= 0xfe00 {
            addr -= 0xfe00;
        }

        let offset = addr / std::mem::size_of::<OA>() as u16;
        match addr % std::mem::size_of::<OA>() as u16 {
            0 => self.oam_ram[offset as usize].y,
            1 => self.oam_ram[offset as usize].x,
            2 => self.oam_ram[offset as usize].tile_idx,
            3 => self.oam_ram[offset as usize].flags,
            _ => unreachable!(),
        }
    }

    pub fn vram_write(&mut self, addr: u16, value: u8) {
        self.vram[(addr - 0x8000) as usize] = value;
    }

    pub fn vram_read(&self, addr: u16) -> u8 {
        self.vram[(addr - 0x8000) as usize]
    }

    pub fn tick(&mut self, cycles: u8) {
        self.line_ticks += cycles as u16;

        match self.lcd.get_ppu_mode() {
            PPUMode::HBlank => self.hblank(),
            PPUMode::VBlank => self.vblank(),
            PPUMode::OAMScan => self.oam_scan(),
            PPUMode::Draw => self.draw(),
        }
    }

    fn oam_scan(&mut self) {
        if self.line_ticks >= 80 {
            self.lcd.set_ppu_mode(PPUMode::Draw);
        }
    }

    fn draw(&mut self) {
        if self.line_ticks >= 80 + 172 {
            self.lcd.set_ppu_mode(PPUMode::HBlank);
        }
    }

    fn hblank(&mut self) {
        if self.line_ticks >= TICKS_PER_LINE {
            self.lcd.inc_ly();
            self.line_ticks = 0;

            if self.lcd.ly >= RESY {
                self.lcd.set_ppu_mode(PPUMode::VBlank);

                request_interrupt(InterruptFlags::VBLANK);

                if self.lcd.lcds & LCDS_FLAGS::MODE_VBLANK_STAT == LCDS_FLAGS::MODE_VBLANK_STAT {
                    request_interrupt(InterruptFlags::STAT);
                }
            } else {
                self.lcd.set_ppu_mode(PPUMode::OAMScan);
            }
        }
    }

    fn vblank(&mut self) {
        if self.line_ticks >= TICKS_PER_LINE {
            self.lcd.inc_ly();

            if self.lcd.ly >= LINES_PER_FRAME {
                self.lcd.ly = 0;
                self.lcd.set_ppu_mode(PPUMode::OAMScan);
            }

            self.line_ticks = 0;
        }
    }
}
