use std::fs::File;
use std::io::Write;

const ROM_SIZE: usize = 0x400;
const CODE_SIZE: usize = 0x100;
const START: usize = 0x150;

const JP_START: [u8; 3] = [0xc3, 0x50, 0x01]; // JP 0x150
const RESET_A: [u8; 2] = [0x3e, 0x0]; // LD A, 0

fn generate_loop(instr: u8) -> [u8; 256] {
    let mut l = [0; CODE_SIZE];

    l[0..2].copy_from_slice(&RESET_A);

    for i in 2..CODE_SIZE - 3 {
        l[i] = instr;
    }

    l[CODE_SIZE - 3..CODE_SIZE].copy_from_slice(&JP_START);

    l
}

fn generate_bench_rom(output: &str, instr: u8, rom_name: &str) {
    let mut rom = vec![0x00; ROM_SIZE];

    rom[0x100..0x103].copy_from_slice(&JP_START);

    let title = rom_name.as_bytes();
    let max_len = 16.min(title.len());
    rom[0x134..0x134 + max_len].copy_from_slice(&title[..max_len]);

    rom[START..START + CODE_SIZE].copy_from_slice(&generate_loop(instr));

    let mut file = File::create(output).unwrap();
    file.write_all(&rom).unwrap();
}

fn main() {
    std::fs::create_dir_all("benches/roms").unwrap();

    generate_bench_rom("benches/roms/nop.gb", 0x00, "NOP");
    generate_bench_rom("benches/roms/inc_a.gb", 0x3C, "INC_A");
    generate_bench_rom("benches/roms/add_a_b.gb", 0x80, "ADD_AB");
}
