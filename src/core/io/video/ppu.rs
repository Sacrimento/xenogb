use super::lcd::{PPUMode, LCD, LCDC_FLAGS, LCDS_FLAGS};
use crate::core::cpu::interrupts::{request_interrupt, InterruptFlags};
use crate::debugger::{PpuMetricFields, PPU_METRICS};
use crate::flag_set;

use crossbeam_channel::Sender;

const LINES_PER_FRAME: u8 = 154;
const TICKS_PER_LINE: u16 = 456;
pub const TICKS_PER_FRAME: u32 = LINES_PER_FRAME as u32 * TICKS_PER_LINE as u32;

pub const RESX: usize = 160;
pub const RESY: usize = 144;

pub type Vbuf = [u8; RESX * RESY];

#[allow(nonstandard_style)]
#[derive(Debug)]
pub enum PPU_LAYER {
    BACKGROUND,
    WINDOW,
    SPRITE,
}

macro_rules! between {
    ( $x:expr, $l:expr, $h:expr ) => {
        ($l..=$h).contains(&$x)
    };
}

#[allow(nonstandard_style)]
mod SpriteFlags {
    #[allow(dead_code)]
    pub const CGB_PALETTE: u8 = 0x7;
    #[allow(dead_code)]
    pub const BANK: u8 = 0x8;
    pub const DMG_PALETTE: u8 = 0x10;
    pub const X_FLIP: u8 = 0x20;
    pub const Y_FLIP: u8 = 0x40;
    pub const PRIORITY: u8 = 0x80;
}

#[derive(Clone, Copy, Debug)]
struct Sprite {
    pub y: u8,
    pub x: u8,
    pub tile_idx: u8,
    pub flags: u8,
}

impl Sprite {
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
    oam: Vec<Sprite>,
    pub vram: [[u8; 0x2000]; 2],
    vram_bank: u8,

    pub lcd: LCD,
    line_ticks: u16,
    pub line_x: u8,
    line_sprites: Option<Vec<Sprite>>,
    window_line: u8,
    window_drawn: bool,

    last_frame: std::time::Instant,

    vbuf: Vbuf,
    video_channel_sd: Sender<Vbuf>,

    pub frames: u64,

    draw_background: bool,
    draw_window: bool,
    draw_sprites: bool,
}

impl PPU {
    pub fn new(video_channel_sd: Sender<Vbuf>) -> Self {
        let mut lcd = LCD::new();
        lcd.set_ppu_mode(PPUMode::OAMScan);

        Self {
            oam: std::iter::repeat_with(Sprite::new)
                .take(40)
                .collect::<Vec<Sprite>>(),
            vram: [[0; 0x2000]; 2],
            vram_bank: 0,
            lcd,
            line_ticks: 0,
            line_x: 0,
            line_sprites: None,
            window_line: 0,
            window_drawn: false,
            last_frame: std::time::Instant::now(),
            vbuf: [0xff; RESX * RESY],
            video_channel_sd,
            frames: 0,
            draw_background: true,
            draw_window: true,
            draw_sprites: true,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9fff => self.vram_write(addr, value),
            0xfe00..=0xfe9f => self.oam_write(addr, value),
            0xff4f => self.vram_bank = value & 0x1,
            _ => unreachable!(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9fff => self.vram_read(addr),
            0xfe00..=0xfe9f => self.oam_read(addr),
            0xff4f => 0xfe | self.vram_bank,
            _ => unreachable!(),
        }
    }

    fn oam_write(&mut self, mut addr: u16, value: u8) {
        if addr >= 0xfe00 {
            addr -= 0xfe00;
        }

        let offset = addr / std::mem::size_of::<Sprite>() as u16;
        match addr % std::mem::size_of::<Sprite>() as u16 {
            0 => self.oam[offset as usize].y = value,
            1 => self.oam[offset as usize].x = value,
            2 => self.oam[offset as usize].tile_idx = value,
            3 => self.oam[offset as usize].flags = value,
            _ => unreachable!(),
        }
    }

    fn oam_read(&self, mut addr: u16) -> u8 {
        if addr >= 0xfe00 {
            addr -= 0xfe00;
        }

        let offset = addr / std::mem::size_of::<Sprite>() as u16;
        match addr % std::mem::size_of::<Sprite>() as u16 {
            0 => self.oam[offset as usize].y,
            1 => self.oam[offset as usize].x,
            2 => self.oam[offset as usize].tile_idx,
            3 => self.oam[offset as usize].flags,
            _ => unreachable!(),
        }
    }

    fn vram_write(&mut self, addr: u16, value: u8) {
        self.vram[self.vram_bank as usize][(addr - 0x8000) as usize] = value;
    }

    fn vram_read(&self, addr: u16) -> u8 {
        self.vram[self.vram_bank as usize][(addr - 0x8000) as usize]
    }

    pub fn tick(&mut self) {
        if !flag_set!(self.lcd.lcdc, LCDC_FLAGS::LCD_PPU_ENABLE) {
            return;
        }

        for _ in 0..4 {
            self.line_ticks += 1;
            match self.lcd.get_ppu_mode() {
                PPUMode::HBlank => self.hblank(),
                PPUMode::VBlank => self.vblank(),
                PPUMode::OAMScan => self.oam_scan(),
                PPUMode::Draw => self.draw(),
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
            ((tile_id as u8).wrapping_add(128) as u16 * 16 + 0x800) as usize
        };
        &self.vram[self.vram_bank as usize][offset..offset + 16]
    }

    fn render_tile(&self, scx: usize, scy: usize, tile_map_flag: u8) -> Option<(bool, u8)> {
        let tile = self.get_tile(scx, scy, flag_set!(self.lcd.lcdc, tile_map_flag));

        let tile_x = scx & 7;
        let tile_y = scy & 7;

        let hi = ((tile[tile_y * 2 + 1] >> (7 - tile_x)) & 1) << 1;
        let lo = (tile[tile_y * 2] >> (7 - tile_x)) & 1;
        let color = hi | lo;

        Some((
            between!(color, 1, 3),
            LCD::get_pixel((self.lcd.bg_palette >> (color * 2)) & 0b11),
        ))
    }

    fn render_bg(&self, x: usize, y: usize) -> Option<(bool, u8)> {
        if !flag_set!(self.lcd.lcdc, LCDC_FLAGS::WINDOW_BG_ENABLE) {
            return None;
        }

        let scx = (x + self.lcd.scx as usize) % 256;
        let scy = (y + self.lcd.scy as usize) % 256;

        self.render_tile(scx, scy, LCDC_FLAGS::BG_TILE_MAP)
    }

    fn render_window(&mut self, x: usize, y: usize) -> Option<(bool, u8)> {
        if !flag_set!(self.lcd.lcdc, LCDC_FLAGS::WINDOW_BG_ENABLE)
            || !flag_set!(self.lcd.lcdc, LCDC_FLAGS::WINDOW_ENABLE)
            || self.lcd.wx > 166
            || self.lcd.wy > 143
        {
            return None;
        }

        if x + 7 < self.lcd.wx as usize || y < self.lcd.wy as usize {
            return None;
        }

        let scx = (x - self.lcd.wx.saturating_sub(7) as usize) % 256;

        self.window_drawn = true;
        self.render_tile(scx, self.window_line as usize, LCDC_FLAGS::WINDOW_TILE_MAP)
    }

    fn render_sprite_from(
        &self,
        sprite: &Sprite,
        tile_id: u8,
        x: usize,
        y: usize,
    ) -> Option<(bool, u8)> {
        let tile =
            &self.vram[self.vram_bank as usize][tile_id as usize * 16..tile_id as usize * 16 + 16];

        let tile_x = x & 7;
        let tile_y = y & 7;

        let x_offset = if flag_set!(sprite.flags, SpriteFlags::X_FLIP) {
            tile_x
        } else {
            7 - tile_x
        };

        let y_offset = if flag_set!(sprite.flags, SpriteFlags::Y_FLIP) {
            14 - (tile_y * 2)
        } else {
            tile_y * 2
        };

        let hi = ((tile[y_offset + 1] >> x_offset) & 1) << 1;
        let lo = (tile[y_offset] >> x_offset) & 1;
        let color = hi | lo;

        if color & 0b11 == 0 {
            return None;
        }

        let palette =
            self.lcd.obj_palettes[flag_set!(sprite.flags, SpriteFlags::DMG_PALETTE) as usize];

        Some((
            !flag_set!(sprite.flags, SpriteFlags::PRIORITY),
            LCD::get_pixel((palette >> (color * 2)) & 0b11),
        ))
    }

    fn render_sprite(&self, x: usize, y: usize) -> Option<(bool, u8)> {
        if !flag_set!(self.lcd.lcdc, LCDC_FLAGS::OBJ_ENABLE) {
            return None;
        }

        let cur_sprites = self.line_sprites.as_ref().unwrap();

        for sprite in cur_sprites
            .iter()
            .filter(|sprite| between!(x + 8, sprite.x as usize, sprite.x as usize + 7))
        {
            let sprite_x = x as i16 - (sprite.x as i16 - 8);
            let sprite_y = y as i16 - (sprite.y as i16 - 16);

            if !between!(sprite_x, 0, RESX as i16 - 1) || !between!(sprite_y, 0, RESY as i16 - 1) {
                return None;
            }

            if flag_set!(self.lcd.lcdc, LCDC_FLAGS::OBJ_SIZE) {
                let is_flipped = flag_set!(sprite.flags, SpriteFlags::Y_FLIP);
                if (sprite_y < 8 && !is_flipped) || (sprite_y >= 8 && is_flipped) {
                    match self.render_sprite_from(
                        sprite,
                        sprite.tile_idx & 0xfe,
                        sprite_x as usize,
                        sprite_y as usize,
                    ) {
                        Some(pix) => return Some(pix),
                        None => continue,
                    }
                } else {
                    match self.render_sprite_from(
                        sprite,
                        sprite.tile_idx | 1,
                        sprite_x as usize,
                        sprite_y as usize,
                    ) {
                        Some(pix) => return Some(pix),
                        None => continue,
                    }
                }
            } else {
                match self.render_sprite_from(
                    sprite,
                    sprite.tile_idx,
                    sprite_x as usize,
                    sprite_y as usize,
                ) {
                    Some(pix) => return Some(pix),
                    None => continue,
                }
            }
        }
        None
    }

    fn oam_scan(&mut self) {
        if self.line_ticks >= 80 {
            self.lcd.set_ppu_mode(PPUMode::Draw);
            return;
        }

        if self.line_sprites.is_some() {
            return;
        }

        let mut oam = self.oam.clone();
        oam.sort_by_key(|sprite| sprite.x);
        self.line_sprites = Some(
            oam.iter()
                .filter(|sprite| {
                    between!(
                        self.lcd.ly + 16,
                        sprite.y,
                        sprite
                            .y
                            .wrapping_add(if !flag_set!(self.lcd.lcdc, LCDC_FLAGS::OBJ_SIZE) {
                                7
                            } else {
                                15
                            })
                    )
                })
                .take(10)
                .cloned()
                .collect(),
        );
    }

    fn get_pixel(&mut self) -> u8 {
        let bg_pixel = if self.draw_background {
            self.render_bg(self.line_x as usize, self.lcd.ly as usize)
        } else {
            None
        };
        let win_pixel = if self.draw_window {
            self.render_window(self.line_x as usize, self.lcd.ly as usize)
        } else {
            None
        };
        let s_pixel = if self.draw_sprites {
            self.render_sprite(self.line_x as usize, self.lcd.ly as usize)
        } else {
            None
        };

        let (bg_prio, background_pixel) = match (bg_pixel, win_pixel) {
            (None, None) => (false, 0xff),
            (Some((prio, bg_pxl)), None) => (prio, bg_pxl),
            (None, Some((prio, win_pxl))) => (prio, win_pxl),
            (Some(_), Some((prio, win_pxl))) => (prio, win_pxl),
            // (Some((prio, bg_pxl)), Some(_)) => (prio, bg_pxl),
        };

        let pixel;

        if s_pixel.is_some() {
            let (has_prio, s_pixel) = s_pixel.unwrap();
            if !has_prio && bg_prio {
                pixel = background_pixel;
            } else {
                pixel = s_pixel;
            }
        } else {
            pixel = background_pixel;
        }

        pixel
    }

    fn draw(&mut self) {
        if self.line_x as usize >= RESX {
            self.lcd.set_ppu_mode(PPUMode::HBlank);

            if flag_set!(self.lcd.lcds, LCDS_FLAGS::MODE_HBLANK_STAT) {
                request_interrupt(InterruptFlags::STAT);
            }

            return;
        }

        self.vbuf[self.lcd.ly as usize * RESX + self.line_x as usize] = self.get_pixel();

        self.line_x += 1;
    }

    fn hblank(&mut self) {
        if self.line_ticks >= TICKS_PER_LINE {
            self.lcd.inc_ly();
            self.line_ticks = 0;
            self.line_sprites = None;
            self.line_x = 0;

            if self.window_drawn {
                self.window_line += 1;
            }
            self.window_drawn = false;

            if self.lcd.ly as usize >= RESY {
                self.lcd.set_ppu_mode(PPUMode::VBlank);

                request_interrupt(InterruptFlags::VBLANK);

                if flag_set!(self.lcd.lcds, LCDS_FLAGS::MODE_VBLANK_STAT) {
                    request_interrupt(InterruptFlags::STAT);
                }
            } else {
                self.lcd.set_ppu_mode(PPUMode::OAMScan);

                if flag_set!(self.lcd.lcds, LCDS_FLAGS::MODE_OAM_STAT) {
                    request_interrupt(InterruptFlags::STAT);
                }
            }
        }
    }

    fn vblank(&mut self) {
        if self.line_ticks >= TICKS_PER_LINE {
            self.lcd.inc_ly();

            if self.lcd.ly >= LINES_PER_FRAME {
                self.lcd.ly = 0;
                self.lcd.set_ppu_mode(PPUMode::OAMScan);

                if !self.video_channel_sd.is_full() {
                    _ = self.video_channel_sd.send(self.vbuf);
                }

                self.frames += 1;
                PPU_METRICS.with_borrow_mut(|mh| mh.count(PpuMetricFields::FRAME_RATE, 1));

                if flag_set!(self.lcd.lcds, LCDS_FLAGS::MODE_OAM_STAT) {
                    request_interrupt(InterruptFlags::STAT);
                }

                let now = std::time::Instant::now();
                self.last_frame = now;
            }

            self.line_ticks = 0;
            self.line_sprites = None;
            self.line_x = 0;
            self.window_line = 0;
            self.window_drawn = false;
        }
    }

    pub fn hide_layer(&mut self, layer: PPU_LAYER) {
        match layer {
            PPU_LAYER::BACKGROUND => self.draw_background = !self.draw_background,
            PPU_LAYER::WINDOW => self.draw_window = !self.draw_window,
            PPU_LAYER::SPRITE => self.draw_sprites = !self.draw_sprites,
        }
    }
}
