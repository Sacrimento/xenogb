mod cpu;
mod dbg;
mod io;
mod mem;
mod ui;
mod utils;

use clap::Parser;
use cpu::cpu::LR35902CPU;
use eframe::egui::ViewportBuilder;
use eframe::Renderer;
use mem::boot::BootRom;
use mem::bus::Bus;
use mem::cartridge::parse_cartridge;
use signal_hook::consts::SIGUSR1;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
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
    // #[arg(short, long, default_value_t = false)]
    // debug: bool,
}

fn main() -> Result<(), XenoGBError> {
    let args = Args::parse();

    let cartridge = parse_cartridge(args.cartridge).expect("Could not load the cartrdige");

    let bus = Bus::new(cartridge, args.boot_rom);

    let mut cpu = LR35902CPU::new(bus, args.serial);

    if args.headless {
        let usr1 = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(SIGUSR1, Arc::clone(&usr1)).unwrap();
        loop {
            if usr1.load(Ordering::Relaxed) {
                vbuf_snapshot();
                return Ok(());
            }
            cpu.step();
        }
    }

    eframe::run_native(
        "xenogb",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size(ui::WINDOW_SIZE),
            ..Default::default()
        },
        Box::new(move |ctx| {
            let cpu: Arc<Mutex<LR35902CPU>> = Arc::new(Mutex::new(cpu));
            let mut _cpu = cpu.clone();

            std::thread::spawn(move || loop {
                _cpu.lock().unwrap().step();
            });

            Ok(Box::new(XenoGBUI::new(ctx, cpu)))
        }),
    )
    .unwrap();

    Ok(())
}
