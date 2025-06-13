use crate::{
    core::cpu::interrupts::{request_interrupt, InterruptFlags},
    flag_set,
};

const DMG_COLORS: [u8; 4] = [0xff, 0xaa, 0x55, 0x00];

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

#[derive(Clone, Copy, Default)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<(u8, u8)> for Pixel {
    fn from(value: (u8, u8)) -> Self {
        let mut value: u16 = (value.0 as u16) << 8 | value.1 as u16;
        value >>= 1;
        // let adjust = |c: u8| ((c as u32 * 527 + 23) >> 6) as u8; // ?????
        // let adjust = |c: u8| c / 0x1f * 0xff;
        // let adjust = |c: u8| c & 0x7 | (c << 3);
        let adjust = |c: u8| (c << 3) | (c >> 2);

        let b = (value as u8) & 0x1f;
        let g = ((value >> 5) & 0x1f) as u8;
        let r = ((value >> 10) & 0x1f) as u8;

        Self {
            r: adjust(r),
            g: adjust(g),
            b: adjust(b),
        }

        // Self { r, g, b }
    }
}

#[derive(Debug, PartialEq)]
pub enum PPUMode {
    VBlank,
    OAMScan,
    Draw,
    HBlank,
}

#[derive(Default, Clone, Copy)]
struct PaletteIndex {
    auto_increment: bool,
    pub addr: usize,
}

impl PaletteIndex {
    pub fn get(&mut self) -> usize {
        let addr = self.addr;

        if self.auto_increment && self.addr < 0x3f {
            self.addr += 1;
        }

        addr
    }
}

impl From<u8> for PaletteIndex {
    fn from(value: u8) -> Self {
        Self {
            auto_increment: flag_set!(value, 0x80),
            addr: (value as usize) & 0x3f,
        }
    }
}

impl From<PaletteIndex> for u8 {
    fn from(value: PaletteIndex) -> Self {
        (value.auto_increment as u8) << 7 | value.addr as u8
    }
}

pub struct LCD {
    pub lcdc: u8,
    pub lcds: u8,

    pub scy: u8,
    pub scx: u8,
    pub ly: u8,
    lyc: u8,

    pub dmg_bg_palette: u8,
    pub dmg_obj_palettes: [u8; 2],

    bg_palette_index: PaletteIndex,
    bg_cram: [u8; 0x40],

    obj_palette_index: PaletteIndex,
    obj_cram: [u8; 0x40],

    pub wy: u8,
    pub wx: u8,
}

impl Default for LCD {
    fn default() -> Self {
        Self {
            lcdc: 0x91,
            lcds: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            dmg_bg_palette: 0xfc,
            dmg_obj_palettes: [0xff, 0xff],
            bg_palette_index: PaletteIndex::default(),
            bg_cram: [0xff; 0x40],
            obj_palette_index: PaletteIndex::default(),
            obj_cram: [0; 0x40],
            wy: 0,
            wx: 0,
        }
    }
}

impl LCD {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff40 => self.lcdc,
            0xff41 => self.lcds,
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff45 => self.lyc,
            0xff47 => self.dmg_bg_palette,
            0xff48 => self.dmg_obj_palettes[0],
            0xff49 => self.dmg_obj_palettes[1],
            0xff4a => self.wy,
            0xff4b => self.wx,
            0xff68 => self.bg_palette_index.into(),
            0xff69 => self.bg_cram[self.bg_palette_index.addr],
            0xff6a => self.obj_palette_index.into(),
            0xff6b => self.obj_cram[self.obj_palette_index.addr],
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
            0xff47 => self.dmg_bg_palette = value,
            0xff48 => self.dmg_obj_palettes[0] = value,
            0xff49 => self.dmg_obj_palettes[1] = value,
            0xff4a => self.wy = value,
            0xff4b => self.wx = value,
            0xff68 => self.bg_palette_index = value.into(),
            0xff69 => self.bg_cram[self.bg_palette_index.get()] = value,
            0xff6a => self.obj_palette_index = value.into(),
            0xff6b => self.obj_cram[self.obj_palette_index.get()] = value,
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

    #[inline]
    pub fn get_dmg_pixel(idx: u8) -> u8 {
        DMG_COLORS[idx as usize]
    }

    #[inline]
    pub fn get_cgb_bg_pixel(&self, palette_idx: usize, color_idx: usize) -> Pixel {
        (
            self.bg_cram[palette_idx * 8 + color_idx * 2],
            self.bg_cram[palette_idx * 8 + color_idx * 2 + 1],
        )
            .into()
    }

    #[inline]
    pub fn get_cgb_obj_pixel(&self, palette_idx: usize, color_idx: usize) -> Pixel {
        (
            self.obj_cram[palette_idx * 8 + color_idx * 2],
            self.obj_cram[palette_idx * 8 + color_idx * 2 + 1],
        )
            .into()
    }
}
