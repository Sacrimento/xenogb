use crate::io::video::ppu::{RESX, RESY, VIDEO_BUFFER};
use std::{fs, io::Write};

#[macro_export]
macro_rules! between {
    ( $x:expr, $l:expr, $h:expr ) => {
        $x >= $l && $x <= $h
    };
}

#[macro_export]
macro_rules! flag_set {
    ( $x:expr, $flag: expr ) => {
        $x & $flag == $flag
    };
}

pub fn vbuf_snapshot() {
    // Output the current video buffer to a PGM formatted file
    let vbuf = VIDEO_BUFFER.lock().unwrap();

    let mut file = fs::File::create("vbuf_snapshot.pgm").unwrap();

    file.write(format!("P5\n{RESX} {RESY}\n255\n").as_bytes())
        .unwrap();

    for y in 0..RESY as usize {
        for x in 0..RESX as usize {
            file.write(&[vbuf[(y * RESX as usize) + x]]).unwrap();
        }
    }
}
