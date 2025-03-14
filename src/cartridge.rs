use std::{fs, path::PathBuf};

#[derive(Debug)]
pub struct CartridgeError;

#[derive(Debug)]
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
    pub fn parse(header: &[u8]) -> CartridgeHeader {
        CartridgeHeader {
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

pub struct Cartridge {
    pub header: CartridgeHeader,
    content: Vec<u8>,
}

impl Cartridge {
    pub fn read(&self, addr: u16) -> u8 {
        self.content[addr as usize]
    }
}

pub fn parse_cartridge(cartridge: PathBuf) -> Result<Cartridge, CartridgeError> {
    let contents: Vec<u8> = fs::read(cartridge).expect("Unable to read the cartridge");

    let header = CartridgeHeader::parse(&contents[0x134..=0x14f]);

    // println!("ROM loaded!");
    // println!("\tTitle: {}", header.title);
    // println!("\tLicensee code: 0x{}", header.new_licensee_code);
    // println!("\tType: 0x{:2x}", header.cartridge_type);
    // println!("\tROM size: {} kb ({} banks)", 32 * (1 << header.rom_size), 32 * (1 << header.rom_size) / 16);
    // println!("\tRAM type: {}", header.ram_size);

    Ok(Cartridge {
        header,
        content: contents,
    })
}
