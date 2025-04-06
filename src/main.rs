mod cpu;
mod dbg;
mod debugger;
mod io;
mod mem;
mod ui;
mod utils;

use clap::Parser;
use cpu::cpu::LR35902CPU;
use crossbeam_channel::{bounded, unbounded, Receiver};
use debugger::Debugger;
use eframe::egui::ViewportBuilder;
use io::video::ppu::{Vbuf, RESX, RESY};
use mem::boot::BootRom;
use mem::bus::Bus;
use mem::cartridge::parse_cartridge;
use signal_hook::consts::SIGUSR1;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use ui::XenoGBUI;
use utils::vbuf_snapshot;

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

fn run_headless(mut cpu: LR35902CPU, video_channel_rc: Receiver<Vbuf>) {
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

fn run(mut cpu: LR35902CPU, mut debugger: Debugger) {
    loop {
        debugger.handle_events();

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

    let (io_events_sd, io_events_rc) = unbounded();
    let cpu = LR35902CPU::new(bus, args.serial, io_events_rc);

    if args.headless {
        return Ok(run_headless(cpu, video_channel_rc));
    }

    eframe::run_native(
        "xenogb",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_inner_size(ui::WINDOW_SIZE)
                .with_resizable(false),
            ..Default::default()
        },
        Box::new(move |ctx| {
            let (ui_debugger_commands_sd, ui_debugger_commands_rc) = unbounded();
            let (dbg_data_sd, dbg_data_rc) = bounded(1);

            std::thread::spawn(move || {
                run(
                    cpu,
                    Debugger::new(args.debug, ui_debugger_commands_rc, dbg_data_sd),
                )
            });

            Ok(Box::new(XenoGBUI::new(
                ctx,
                io_events_sd,
                video_channel_rc,
                ui_debugger_commands_sd,
                dbg_data_rc,
                args.debug,
            )))
        }),
    )
    .unwrap();

    Ok(())
}
