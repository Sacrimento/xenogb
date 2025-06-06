use super::mbc::{mbc, MemoryBankController};
use std::{fs, path::PathBuf};

#[allow(unused)]
pub struct CartridgeHeader {
    //0104-0133 — Nintendo logo
    //0134-0143 — Title
    //013F-0142 — Manufacturer code
    //0143 — CGB flag
    //0144–0145 — New licensee code
    //0146 — SGB flag
    //0147 — Cartridge type
    //0148 — ROM size
    //0149 — RAM size
    //014A — Destination code
    //014B — Old licensee code
    //014C — Mask ROM version number
    //014D — Header checksum
    //014E-014F — Global checksum
    title: String,             // 11-16 bytes
    manufacturer_code: String, // 4 bytes
    cgb_flag: u8,
    new_licensee_code: String, // 2 bytes
    sgb_flag: u8,
    cartridge_type: u8,
    rom_size: u8,
    ram_size: u8,
    dest_code: u8,
    old_licensee_code: u8,
    rom_version: u8,
    header_checksum: u8,
    global_checksum: u16,
}

impl CartridgeHeader {
    pub fn new(header: &[u8]) -> Self {
        Self {
            title: std::str::from_utf8(&header[0..11])
                .expect("Invalid cartridge title")
                .into(),
            manufacturer_code: std::str::from_utf8(&header[11..15])
                .expect("Invalid cartridge title")
                .into(),
            cgb_flag: header[15],
            new_licensee_code: std::str::from_utf8(&header[16..18])
                .expect("Invalid licensee code")
                .into(),
            sgb_flag: header[18],
            cartridge_type: header[19],
            rom_size: header[20],
            ram_size: header[21],
            dest_code: header[22],
            old_licensee_code: header[23],
            rom_version: header[24],
            header_checksum: header[25],
            global_checksum: u16::from_be_bytes(header[26..28].try_into().unwrap()),
        }
    }
}

#[allow(unused)]
pub struct Cartridge {
    header: CartridgeHeader,
    pub mbc: Box<dyn MemoryBankController + Send + Sync>,
}

impl Cartridge {
    pub fn new(rom_path: PathBuf) -> Self {
        let contents: Vec<u8> = fs::read(&rom_path).expect("Unable to read the rom_path");

        let header = CartridgeHeader::new(&contents[0x134..=0x14f]);

        let mbc = mbc(
            header.cartridge_type,
            header.ram_size,
            header.rom_size,
            contents,
            rom_path,
        );

        Self { header, mbc }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.mbc.read(addr)
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.mbc.write(addr, value);
    }
}
