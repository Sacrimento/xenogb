#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum BootRom {
    NONE,
    DMG0,
    DMG,
    MGB,
}

pub fn boot_rom(rom_name: BootRom) -> &'static [u8; 0x100] {
    match rom_name {
        BootRom::NONE => &[0; 0x100],
        BootRom::DMG0 => include_bytes!("../../../boot/dmg0_boot.gb"),
        BootRom::DMG => include_bytes!("../../../boot/dmg_boot.gb"),
        BootRom::MGB => include_bytes!("../../../boot/mgb_boot.gb"),
    }
}
