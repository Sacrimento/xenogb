mod bus;
mod cartridge;
mod cpu;
mod instructions;
mod utils;
mod ram;
mod dbg;

use cpu::LR35902CPU;
use bus::Bus;
use cartridge::parse_cartridge;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug)]
struct XenoGBError;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    cartridge: PathBuf,

    #[arg(short, long, default_value_t = false)]
    debug: bool
}

fn main() -> Result<(), XenoGBError> {
    let args = Args::parse();
    let cartridge = parse_cartridge(args.cartridge).expect("Could not load the cartrdige");
    
    let mut bus = Bus::new(cartridge);

    let mut cpu = LR35902CPU::new(bus);

    cpu.run();
    Ok(())
}
