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
}

impl PPU {
    pub fn new() -> Self {
        Self {
            oam_ram: std::iter::repeat_with(|| OA::new())
                .take(40)
                .collect::<Vec<OA>>(),
            vram: [0; 0x2000],
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
            _ => panic!("Unreachable"),
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
            _ => panic!("Unreachable"),
        }
    }

    pub fn vram_write(&mut self, addr: u16, value: u8) {
        self.vram[(addr - 0x8000) as usize] = value;
    }

    pub fn vram_read(&self, addr: u16) -> u8 {
        self.vram[(addr - 0x8000) as usize]
    }
}
