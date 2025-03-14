mod bus;
mod cartridge;
mod cpu;
mod dbg;
mod instructions;
mod io;
mod ram;
mod utils;

use bus::Bus;
use cartridge::parse_cartridge;
use clap::Parser;
use cpu::LR35902CPU;
use std::path::PathBuf;

#[derive(Debug)]
struct XenoGBError;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    cartridge: PathBuf,

    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn main() -> Result<(), XenoGBError> {
    let args = Args::parse();
    let cartridge = parse_cartridge(args.cartridge).expect("Could not load the cartrdige");

    let bus = Bus::new(cartridge);

    let mut cpu = LR35902CPU::new(bus);

    cpu.run();
    Ok(())
}
