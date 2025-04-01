mod cpu;
mod dbg;
mod io;
mod mem;
mod ui;
mod utils;

use clap::Parser;
use cpu::cpu::LR35902CPU;
use mem::bus::Bus;
use mem::cartridge::parse_cartridge;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use ui::XenoGBUI;

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
    // #[arg(short, long, default_value_t = false)]
    // debug: bool,
}

fn main() -> Result<(), XenoGBError> {
    let args = Args::parse();
    let cartridge = parse_cartridge(args.cartridge).expect("Could not load the cartrdige");

    let bus = Bus::new(cartridge);

    let mut cpu = LR35902CPU::new(bus, args.serial);

    if args.headless {
        loop {
            cpu.step();
        }
    }

    eframe::run_native(
        "xenogb",
        eframe::NativeOptions::default(),
        Box::new(move |ctx| {
            let cpu: Arc<Mutex<LR35902CPU>> = Arc::new(Mutex::new(cpu));
            let mut _cpu = cpu.clone();

            std::thread::spawn(move || loop {
                _cpu.lock().unwrap().step();
            });

            Ok(Box::new(XenoGBUI::new(ctx, cpu)))
        }),
    );

    Ok(())
}
