#[macro_export]
macro_rules! set_u16_hi {
    ( $dest:expr, $val:expr ) => {
        ($dest & 0xff) | (($val as u16) << 8)
    };
}

#[macro_export]
macro_rules! set_u16_lo {
    ( $dest:expr, $val:expr ) => {
        ($dest & 0xff00) | $val as u16
    };
}
