mod core;
mod dbg;
mod debugger;
mod ui;

use core::cpu::cpu::LR35902CPU;
use core::mem::boot::BootRom;
use core::mem::bus::Bus;
use core::mem::cartridge::parse_cartridge;
use core::run_emu::run_headless;
use ui::run_ui;

use chrono::Local;
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

fn setup_logger() -> String {
    let file_path = format!(
        "logs/{}.log",
        Local::now().format("%Y-%m-%d_%H-%M-%S").to_string()
    );

    let config = include_str!("../logging.yml")
        .to_string()
        .replace("__LOG_FILE__", &file_path);

    log4rs::init_raw_config(serde_yaml::from_str(&config).expect("Could not parse logging file"))
        .unwrap();

    file_path
}

fn main() -> Result<(), XenoGBError> {
    setup_logger();

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
