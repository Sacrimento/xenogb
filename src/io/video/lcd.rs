use crate::cpu::interrupts::{request_interrupt, InterruptFlags};

const COLORS: [u8; 4] = [0xff, 0xaa, 0x55, 0x00];

#[allow(nonstandard_style)]
pub mod LCDC_FLAGS {
    pub const LCD_PPU_ENABLE: u8 = 0x80;
    pub const WINDOW_TILE_MAP: u8 = 0x40;
    pub const WINDOW_ENABLE: u8 = 0x20;
    pub const WINDOW_BG_ADDRESSING_MODE: u8 = 0x10;
    pub const BG_TILE_MAP: u8 = 0x8;
    pub const OBJ_SIZE: u8 = 0x4;
    pub const OBJ_ENABLE: u8 = 0x2;
    pub const WINDOW_BG_ENABLE: u8 = 0x1;
}

#[allow(nonstandard_style)]
pub mod LCDS_FLAGS {
    pub const MODE_LYC_EQ_LY_STAT: u8 = 0x40;
    pub const MODE_OAM_STAT: u8 = 0x20;
    pub const MODE_VBLANK_STAT: u8 = 0x10;
    pub const MODE_HBLANK_STAT: u8 = 0x8;
    pub const LYC_EQ_LY: u8 = 0x4;
    pub const PPU_MODE: u8 = 0x3;
}

#[derive(Debug)]
pub enum PPUMode {
    VBlank,
    OAMScan,
    Draw,
    HBlank,
}

#[derive(Default)]
pub struct LCD {
    pub lcdc: u8,
    pub lcds: u8,

    pub scy: u8,
    pub scx: u8,
    pub ly: u8,
    lyc: u8,

    pub bg_palette: u8,
    pub obj_palettes: [u8; 2],

    pub wy: u8,
    pub wx: u8,
}

impl LCD {
    pub fn new() -> Self {
        Self {
            lcdc: 0x91,
            bg_palette: 0xfc,
            obj_palettes: [0xff, 0xff],
            ..Default::default()
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff40 => self.lcdc,
            0xff41 => self.lcds,
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff45 => self.lyc,
            0xff47 => self.bg_palette,
            0xff48 => self.obj_palettes[0],
            0xff49 => self.obj_palettes[1],
            0xff4a => self.wy,
            0xff4b => self.wx,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xff40 => self.lcdc = value,
            0xff41 => self.lcds = value,
            0xff42 => self.scy = value,
            0xff43 => self.scx = value,
            0xff44 => self.ly = value,
            0xff45 => self.lyc = value,
            0xff47 => self.bg_palette = value,
            0xff48 => self.obj_palettes[0] = value,
            0xff49 => self.obj_palettes[1] = value,
            0xff4a => self.wy = value,
            0xff4b => self.wx = value,
            _ => unreachable!(),
        }
    }

    pub fn get_ppu_mode(&self) -> PPUMode {
        match self.lcds & LCDS_FLAGS::PPU_MODE {
            0 => PPUMode::HBlank,
            1 => PPUMode::VBlank,
            2 => PPUMode::OAMScan,
            3 => PPUMode::Draw,
            _ => unreachable!(),
        }
    }

    pub fn set_ppu_mode(&mut self, mode: PPUMode) {
        let val = match mode {
            PPUMode::HBlank => 0,
            PPUMode::VBlank => 1,
            PPUMode::OAMScan => 2,
            PPUMode::Draw => 3,
        };
        self.lcds = (self.lcds & 0xfc) | val;
    }

    pub fn inc_ly(&mut self) {
        self.ly += 1;

        if self.ly == self.lyc {
            self.lcds |= LCDS_FLAGS::LYC_EQ_LY;

            if self.lcds & LCDS_FLAGS::MODE_LYC_EQ_LY_STAT == LCDS_FLAGS::MODE_LYC_EQ_LY_STAT {
                request_interrupt(InterruptFlags::STAT);
            }
        } else {
            self.lcds &= !LCDS_FLAGS::LYC_EQ_LY;
        }
    }

    pub fn get_pixel(idx: u8) -> u8 {
        COLORS[idx as usize]
    }
}
