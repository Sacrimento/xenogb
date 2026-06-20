use crate::core::cpu::LR35902CPU;
use crate::core::io::video::ppu::Vbuf;
use crate::core::io::video::ppu::{RESX, RESY};
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::path::PathBuf;

use std::{fs, io::Write};

use log::info;

#[macro_export]
macro_rules! flag_set {
    ( $x: expr, $flag: expr ) => {
        $x & $flag == $flag
    };
}

pub fn dump_regs(cpu: &LR35902CPU, test_out_dir: &Option<PathBuf>) {
    let fpath = match test_out_dir {
        None => format!("{}/cpu_registers.txt", temp_dir().display()),
        Some(test_out_dir) => {
            create_dir_all(test_out_dir).expect("Could not create test dir");
            format!("{}/cpu_registers.txt", test_out_dir.display())
        }
    };

    info!("CPU registers saved to {}", fpath);
    let mut file = fs::File::create(fpath).unwrap();

    file.write_all(
        format!(
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X}",
            cpu.registers.a,
            cpu.registers.f,
            cpu.registers.b,
            cpu.registers.c,
            cpu.registers.d,
            cpu.registers.e,
            cpu.registers.h,
            cpu.registers.l
        )
        .as_bytes(),
    )
    .unwrap();
}

pub fn vbuf_snapshot(frame: Vbuf, test_out_dir: &Option<PathBuf>) {
    // Output the current video buffer to a PPM formatted file

    let fpath = match test_out_dir {
        None => format!("{}/vbuf_snapshot.ppm", temp_dir().display()),
        Some(test_out_dir) => {
            create_dir_all(test_out_dir).expect("Could not create test dir");
            format!("{}/vbuf_snapshot.ppm", test_out_dir.display())
        }
    };

    info!("Video buffer snapshot saved to {}", fpath);
    let mut file = fs::File::create(fpath).unwrap();

    file.write_all(format!("P6\n{RESX} {RESY}\n255\n").as_bytes())
        .unwrap();

    for y in 0..RESY {
        for x in 0..RESX {
            file.write_all(&[
                frame[(y * RESX) + x].r,
                frame[(y * RESX) + x].g,
                frame[(y * RESX) + x].b,
            ])
            .unwrap();
        }
    }
}
