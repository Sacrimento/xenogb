mod cpu;
mod dbg;
mod debugger;
mod io;
mod io_event;
mod mem;
mod playback;
mod run_emu;
mod ui;
mod utils;

use crate::cpu::cpu::LR35902CPU;

use crate::mem::boot::BootRom;
use crate::mem::bus::Bus;
use crate::mem::cartridge::parse_cartridge;
use crate::playback::Playback;
use crate::run_emu::run_headless;
use crate::ui::run_ui;
use clap::Parser;
use crossbeam_channel::bounded;

use std::path::PathBuf;

#[derive(Debug)]
struct XenoGBError;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    cartridge: PathBuf,

    #[arg(long, default_value_t = false)]
    headless: bool,

    #[arg(short, long, default_value_t = false)]
    serial: bool,

    #[arg(short, long, value_enum, default_value_t = BootRom::NONE)]
    boot_rom: BootRom,

    #[arg(short, long, default_value_t = false)]
    debug: bool,

    #[arg(long, default_value_t = false)]
    record: bool,

    #[arg(long, default_value = None)]
    record_path: Option<PathBuf>,

    #[arg(long, default_value = None)]
    replay_path: Option<PathBuf>,
}

fn main() -> Result<(), XenoGBError> {
    let args = Args::parse();

    let cartridge = parse_cartridge(args.cartridge).expect("Could not load the cartrdige");

    let (video_channel_sd, video_channel_rc) = bounded(1);
    let bus = Bus::new(cartridge, args.boot_rom, video_channel_sd);

    if args.headless {
        return Ok(run_headless(
            LR35902CPU::new(bus, args.serial, u32::MAX),
            video_channel_rc,
        ));
    }

    Ok(run_ui(
        bus,
        video_channel_rc,
        args.debug,
        args.serial,
        args.record,
        args.record_path,
        args.replay_path,
    ))
}
