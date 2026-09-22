/// a (- eps) <= b
#[macro_export]
macro_rules! loose_le {
    ($a:expr, $b:expr) => {
        loose_le!($a, $b, 1e-4)
    };
    ($a:expr, $b:expr, $eps:expr) => {
        $a - $eps <= $b
    };
}

/// a (+ eps) < b
#[macro_export]
macro_rules! strict_lt {
    ($a:expr, $b:expr) => {
        strict_lt!($a, $b, 1e-4)
    };
    ($a:expr, $b:expr, $eps:expr) => {
        $a + $eps < $b
    };
}

/// a (+ eps) >= b
#[macro_export]
macro_rules! loose_ge {
    ($a:expr, $b:expr) => {
        loose_ge!($a, $b, 1e-4)
    };
    ($a:expr, $b:expr, $eps:expr) => {
        $a + $eps >= $b
    };
}

/// a (- eps) > b
#[macro_export]
macro_rules! strict_gt {
    ($a:expr, $b:expr) => {
        strict_gt!($a, $b, 1e-4)
    };
    ($a:expr, $b:expr, $eps:expr) => {
        $a - $eps > $b
    };
}

#[inline(always)]
pub const fn square(x: f32) -> f32 {
    x * x
}

#[inline(always)]
pub const fn cube(x: f32) -> f32 {
    x * x * x
}
