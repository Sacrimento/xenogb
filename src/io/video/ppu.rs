use super::lcd::{PPUMode, LCD, LCDC_FLAGS, LCDS_FLAGS};
use crate::interrupts::{request_interrupt, InterruptFlags};
use crate::{between, flag_set};

const LINES_PER_FRAME: u8 = 154;
const TICKS_PER_LINE: u16 = 456;
const RESX: u16 = 160;
const RESY: u16 = 144;

#[allow(nonstandard_style)]
mod OAFlags {
    pub const CGB_PALETTE: u8 = 0x7;
    pub const BANK: u8 = 0x8;
    pub const DMG_PALETTE: u8 = 0x10;
    pub const X_FLIP: u8 = 0x20;
    pub const Y_FLIP: u8 = 0x40;
    pub const PRIORITY: u8 = 0x80;
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
    pub video_buffer: [u8; (RESX * RESY) as usize],
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
            video_buffer: [0xff; (RESX * RESY) as usize],
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

    fn process_frame(&mut self) -> () {
        self.process_bg();
        self.process_win();
        self.process_sprites();
    }

    fn process_bg(&mut self) {
        if !flag_set!(self.lcd.lcdc, LCDC_FLAGS::WINDOW_BG_ENABLE) {
            return;
        }

        for ry in 0..RESY as usize {
            for rx in 0..RESX as usize {
                let scx = (rx + self.lcd.scx as usize) % 256;
                let scy = (ry + self.lcd.scy as usize) % 256;

                let tile =
                    self.get_tile(scx, scy, flag_set!(self.lcd.lcdc, LCDC_FLAGS::BG_TILE_MAP));

                let tile_x = scx & 7;
                let tile_y = scy & 7;

                let hi = ((tile[tile_y * 2 + 1] >> (7 - tile_x)) & 1) << 1;
                let lo = (tile[tile_y * 2] >> (7 - tile_x)) & 1;
                let color = hi | lo;

                self.video_buffer[ry * RESX as usize + rx] =
                    (self.lcd.bg_palette >> (color * 2)) & 0xff;
            }
        }
    }

    fn get_tile(&self, x: usize, y: usize, area_flag: bool) -> &[u8] {
        let area = if area_flag { 0x9c00 } else { 0x9800 };
        let id_offset = ((y >> 3) << 5) + (x >> 3);

        let tile_id = self.vram_read((area + id_offset) as u16) as usize;

        let offset = if flag_set!(self.lcd.lcdc, LCDC_FLAGS::WINDOW_BG_ADDRESSING_MODE) {
            tile_id * 16
        } else {
            0x800 + tile_id.wrapping_add(128) * 16
        };
        &self.vram[offset..offset + 16]
    }

    fn process_win(&mut self) {
        if !flag_set!(self.lcd.lcdc, LCDC_FLAGS::WINDOW_ENABLE) {
            return;
        }

        for ry in 0..RESY as usize {
            for rx in 0..RESX as usize {
                let scx = (rx + self.lcd.wx as usize) % 256;
                let scy = (ry + self.lcd.wy as usize) % 256;

                let tile = self.get_tile(
                    scx,
                    scy,
                    flag_set!(self.lcd.lcdc, LCDC_FLAGS::WINDOW_TILE_MAP),
                );

                let tile_x = scx & 7;
                let tile_y = scy & 7;

                let hi = ((tile[tile_y * 2 + 1] >> (7 - tile_x)) & 1) << 1;
                let lo = (tile[tile_y * 2] >> (7 - tile_x)) & 1;
                let color = hi | lo;

                self.video_buffer[ry * RESX as usize + rx] =
                    (self.lcd.bg_palette >> (color * 2)) & 0xff;
            }
        }
    }

    // fn render_tile(
    //     &mut self,
    //     tile_data: &[u8],
    //     (scx, scy): (usize, usize),
    //     (rx, ry): (usize, usize),
    // ) {
    //     let tile_x = scx & 7;
    //     let tile_y = scy & 7;

    //     let hi = ((tile_data[tile_y * 2 + 1] >> (7 - tile_x)) & 1) << 1;
    //     let lo = (tile_data[tile_y * 2] >> (7 - tile_x)) & 1;

    //     self.video_buffer[ry * RESX as usize + rx] = COLORS[(hi | lo) as usize];
    // }

    fn process_sprites(&mut self) {
        for oa in self.oam_ram.iter() {
            if !between!(oa.x, 1, 167) || !between!(oa.y, 2, 152) {
                continue;
            }

            let tile = &self.vram[oa.tile_idx as usize * 16..oa.tile_idx as usize * 16 + 16];

            for y in oa.y..oa.y + 8 {
                for x in oa.x..oa.x + 8 {
                    let tile_x = x as usize & 7;
                    let tile_y = y as usize & 7;

                    let hi = ((tile[tile_y * 2 + 1] >> (7 - tile_x)) & 1) << 1;
                    let lo = (tile[tile_y * 2] >> (7 - tile_x)) & 1;
                    let color = hi | lo;

                    if color & 0b11 == 0 {
                        continue;
                    }

                    let palette =
                        self.lcd.obj_palettes[flag_set!(oa.flags, OAFlags::DMG_PALETTE) as usize];

                    self.video_buffer[(y as usize - 16) * RESX as usize + x as usize - 8] =
                        (palette >> (color * 2)) & 0xff;
                }
            }
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.line_ticks += 1;
            match self.lcd.get_ppu_mode() {
                PPUMode::HBlank => self.hblank(),
                PPUMode::VBlank => self.vblank(),
                PPUMode::OAMScan => self.oam_scan(),
                PPUMode::Draw => self.draw(),
            }
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

            if flag_set!(self.lcd.lcds, LCDS_FLAGS::MODE_VBLANK_STAT) {
                request_interrupt(InterruptFlags::STAT);
            }
        }
    }

    fn hblank(&mut self) {
        if self.line_ticks >= TICKS_PER_LINE {
            self.lcd.inc_ly();
            self.line_ticks = 0;

            if self.lcd.ly as u16 >= RESY {
                self.lcd.set_ppu_mode(PPUMode::VBlank);

                request_interrupt(InterruptFlags::VBLANK);

                if flag_set!(self.lcd.lcds, LCDS_FLAGS::MODE_VBLANK_STAT) {
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
                self.process_frame();
            }

            self.line_ticks = 0;
        }
    }
}
