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
