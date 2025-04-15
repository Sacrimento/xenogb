#[cfg(unix)]
use crate::core::io::video::ppu::Vbuf;

#[macro_export]
macro_rules! flag_set {
    ( $x:expr, $flag: expr ) => {
        $x & $flag == $flag
    };
}

#[cfg(unix)]
pub fn vbuf_snapshot(frame: Vbuf) {
    use log::info;

    use crate::core::io::video::ppu::{RESX, RESY};
    use std::{fs, io::Write};
    // Output the current video buffer to a PGM formatted file

    let fpath = "vbuf_snapshot.pgm";

    info!("Video buffer snapshot saved to {}", fpath);
    let mut file = fs::File::create(fpath).unwrap();

    file.write(format!("P5\n{RESX} {RESY}\n255\n").as_bytes())
        .unwrap();

    for y in 0..RESY as usize {
        for x in 0..RESX as usize {
            file.write(&[frame[(y * RESX as usize) + x]]).unwrap();
        }
    }
}
