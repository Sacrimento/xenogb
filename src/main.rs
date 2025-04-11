mod cpu;
mod dbg;
mod debugger;
mod io;
mod mem;
mod ui;
mod utils;

use crate::cpu::cpu::LR35902CPU;
use crate::debugger::Debugger;
use crate::io::video::ppu::Vbuf;
use crate::mem::boot::BootRom;
use crate::mem::bus::Bus;
use crate::mem::cartridge::parse_cartridge;
use clap::Parser;
use crossbeam_channel::{bounded, unbounded, Receiver};
use std::path::PathBuf;

use ui::run_ui;

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
}

#[cfg(unix)]
fn run_headless(mut cpu: LR35902CPU, video_channel_rc: Receiver<Vbuf>) {
    use crate::io::video::ppu::{RESX, RESY};
    use crate::utils::vbuf_snapshot;
    use signal_hook::consts::SIGUSR1;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    let usr1 = Arc::new(AtomicBool::new(false));
    let mut last_frame: Vbuf = [0; RESX * RESY];
    signal_hook::flag::register(SIGUSR1, Arc::clone(&usr1)).unwrap();

    loop {
        cpu.step();
        if usr1.load(Ordering::Relaxed) {
            vbuf_snapshot(last_frame);
            return;
        }
        if let Ok(frame) = video_channel_rc.try_recv() {
            last_frame = frame;
        }
    }
}

#[cfg(windows)]
fn run_headless(mut _cpu: LR35902CPU, _video_channel_rc: Receiver<Vbuf>) {
    panic!("Running headless mode is not yet supported on windows")
}

fn run(mut cpu: LR35902CPU, mut debugger: Debugger) {
    loop {
        debugger.handle_events(&mut cpu);

        if debugger.cpu_should_step(&cpu) {
            cpu.step();
            cpu.handle_io_events();
        }

        debugger.collect(&cpu);
    }
}

fn main() -> Result<(), XenoGBError> {
    let args = Args::parse();

    let cartridge = parse_cartridge(args.cartridge).expect("Could not load the cartrdige");

    let (video_channel_sd, video_channel_rc) = bounded(1);
    let bus = Bus::new(cartridge, args.boot_rom, video_channel_sd);

    if args.headless {
        let (_, io_events_rc) = unbounded();
        return Ok(run_headless(
            LR35902CPU::new(bus, args.serial, u32::MAX, io_events_rc),
            video_channel_rc,
        ));
    }

    Ok(run_ui(bus, video_channel_rc, args.debug, args.serial))
}
