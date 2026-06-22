#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum BootRom {
    CGB,
    CGB0,
    DMG,
    DMG0,
    MGB,
    NONE,
}

pub fn get_boot_rom(rom: BootRom) -> (&'static [u8; 0x100], Option<&'static [u8; 0x700]>) {
    match rom {
        BootRom::NONE => (&[0; 0x100], None),
        BootRom::DMG => (include_bytes!("../../../boot/dmg_boot.gb"), None),
        BootRom::DMG0 => (include_bytes!("../../../boot/dmg0_boot.gb"), None),
        BootRom::CGB0 => {
            let rom = include_bytes!("../../../boot/cgb0_boot.gbc");
            (
                rom[0..0x100].try_into().expect("ROM"),
                Some(rom[0x200..0x900].try_into().expect("ROM")),
            )
        }
        BootRom::CGB => {
            let rom = include_bytes!("../../../boot/cgb_boot.gbc");
            (
                rom[0..0x100].try_into().expect("ROM"),
                Some(rom[0x200..0x900].try_into().expect("ROM")),
            )
        }
        BootRom::MGB => (include_bytes!("../../../boot/mgb_boot.gb"), None),
    }
}
