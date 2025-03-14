pub fn get_bit(x: u8, n: u8) -> u8 {
    (x >> n) & 1
}

pub fn set_bit(x: u8, n: u8, v: u8) -> u8 {
    (x & (!(1 << n))) | (v << n)
}

pub fn flip_bit(x: u8, n: u8) -> u8 {
    x ^ (1 << n)
}

#[macro_export]
macro_rules! between {
    ( $x:expr, $l:expr, $h:expr ) => {
        $x >= $l && $x <= $h
    };
}
