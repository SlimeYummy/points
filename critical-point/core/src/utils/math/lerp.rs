use std::f32::consts::PI;

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
pub fn lerp_with(a: f32, b: f32, t: f32, ease: fn(f32) -> f32) -> f32 {
    a + (b - a) * ease(t)
}

#[inline]
pub fn ease_in_sine(x: f32) -> f32 {
    1.0 - (x * PI / 2.0).cos()
}

#[inline]
pub fn ease_out_sine(x: f32) -> f32 {
    (x * PI / 2.0).sin()
}

#[inline]
pub fn ease_in_out_sine(x: f32) -> f32 {
    -((PI * x).cos() - 1.0) / 2.0
}

#[inline]
pub fn ease_in_quad(x: f32) -> f32 {
    x * x
}

#[inline]
pub fn ease_out_quad(x: f32) -> f32 {
    let y = 1.0 - x;
    1.0 - y * y
}

#[inline]
pub fn ease_in_out_quad(x: f32) -> f32 {
    if x < 0.5 {
        2.0 * x * x
    }
    else {
        let y = -2.0 * x + 2.0;
        1.0 - y * y / 2.0
    }
}

#[inline]
pub fn ease_in_cubic(x: f32) -> f32 {
    x * x * x
}

#[inline]
pub fn ease_out_cubic(x: f32) -> f32 {
    let y = 1.0 - x;
    1.0 - y * y * y
}

#[inline]
pub fn ease_in_out_cubic(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x * x * x
    }
    else {
        let y = -2.0 * x + 2.0;
        1.0 - y * y * y / 2.0
    }
}

#[inline]
pub fn ease_in_quart(x: f32) -> f32 {
    x * x * x * x
}

#[inline]
pub fn ease_out_quart(x: f32) -> f32 {
    let y = 1.0 - x;
    1.0 - y * y * y * y
}

#[inline]
pub fn ease_in_out_quart(x: f32) -> f32 {
    if x < 0.5 {
        8.0 * x * x * x * x
    }
    else {
        let y = -2.0 * x + 2.0;
        1.0 - y * y * y * y / 2.0
    }
}

#[inline]
pub fn ease_in_quint(x: f32) -> f32 {
    x * x * x * x * x
}

#[inline]
pub fn ease_out_quint(x: f32) -> f32 {
    let x = 1.0 - x;
    1.0 - x * x * x * x * x
}

#[inline]
pub fn ease_in_out_quint(x: f32) -> f32 {
    if x < 0.5 {
        16.0 * x * x * x * x * x
    }
    else {
        let x = -2.0 * x + 2.0;
        1.0 - x * x * x * x * x / 2.0
    }
}

#[inline]
pub fn ease_in_expo(x: f32) -> f32 {
    if x == 0.0 {
        0.0
    }
    else {
        2.0f32.powf(10.0 * x - 10.0)
    }
}

#[inline]
pub fn ease_out_expo(x: f32) -> f32 {
    if x == 1.0 {
        1.0
    }
    else {
        1.0 - 2.0f32.powf(-10.0 * x)
    }
}

#[inline]
pub fn ease_in_out_expo(x: f32) -> f32 {
    if x == 0.0 {
        0.0
    }
    else if x == 1.0 {
        1.0
    }
    else if x < 0.5 {
        2.0f32.powf(20.0 * x - 10.0) / 2.0
    }
    else {
        (2.0 - 2.0f32.powf(-20.0 * x + 10.0)) / 2.0
    }
}

#[inline]
pub fn ease_in_circ(x: f32) -> f32 {
    1.0 - (1.0 - x.powi(2)).sqrt()
}

#[inline]
pub fn ease_out_circ(x: f32) -> f32 {
    (1.0 - (x - 1.0).powi(2)).sqrt()
}

#[inline]
pub fn ease_in_out_circ(x: f32) -> f32 {
    if x < 0.5 {
        (1.0 - (1.0 - (2.0 * x).powi(2)).sqrt()) / 2.0
    }
    else {
        ((1.0 - (-2.0 * x + 2.0).powi(2)).sqrt() + 1.0) / 2.0
    }
}

#[inline]
pub fn ease_in_back(x: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    C3 * x * x * x - C1 * x * x
}

#[inline]
pub fn ease_out_back(x: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    let y = x - 1.0;
    1.0 + C3 * y * y * y + C1 * y * y
}

#[inline]
pub fn ease_in_out_back(x: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C2: f32 = C1 * 1.525;
    if x < 0.5 {
        let y = 2.0 * x;
        (y * y * ((C2 + 1.0) * y - C2)) / 2.0
    }
    else {
        let y = 2.0 * x - 2.0;
        (y * y * ((C2 + 1.0) * y + C2) + 2.0) / 2.0
    }
}

#[inline]
pub fn ease_in_elastic(x: f32) -> f32 {
    const C4: f32 = (2.0 * PI) / 3.0;
    if x == 0.0 {
        0.0
    }
    else if x == 1.0 {
        1.0
    }
    else {
        -2.0f32.powf(10.0 * x - 10.0) * ((x * 10.0 - 10.75) * C4).sin()
    }
}

#[inline]
pub fn ease_out_elastic(x: f32) -> f32 {
    const C4: f32 = (2.0 * PI) / 3.0;
    if x == 0.0 {
        0.0
    }
    else if x == 1.0 {
        1.0
    }
    else {
        2.0f32.powf(-10.0 * x) * ((x * 10.0 - 0.75) * C4).sin() + 1.0
    }
}

#[inline]
pub fn ease_in_out_elastic(x: f32) -> f32 {
    const C5: f32 = (2.0 * PI) / 4.5;
    if x == 0.0 {
        0.0
    }
    else if x == 1.0 {
        1.0
    }
    else if x < 0.5 {
        -(2.0f32.powf(20.0 * x - 10.0) * ((20.0 * x - 11.125) * C5).sin()) / 2.0
    }
    else {
        (2.0f32.powf(-20.0 * x + 10.0) * ((20.0 * x - 11.125) * C5).sin()) / 2.0 + 1.0
    }
}

#[inline]
fn ease_out_bounce(x: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;

    if x < 1.0 / D1 {
        N1 * x * x
    }
    else if x < 2.0 / D1 {
        let x_new = x - 1.5 / D1;
        N1 * x_new * x_new + 0.75
    }
    else if x < 2.5 / D1 {
        let x_new = x - 2.25 / D1;
        N1 * x_new * x_new + 0.9375
    }
    else {
        let x_new = x - 2.625 / D1;
        N1 * x_new * x_new + 0.984375
    }
}

#[inline]
pub fn ease_in_bounce(x: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - x)
}

#[inline]
pub fn ease_in_out_bounce(x: f32) -> f32 {
    if x < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * x)) / 2.0
    }
    else {
        (1.0 + ease_out_bounce(2.0 * x - 1.0)) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_ease_sine() {
        assert_eq!(ease_in_sine(0.0), 0.0);
        assert_eq!(ease_in_sine(1.0), 1.0);
        assert_abs_diff_eq!(ease_in_sine(0.5), 0.29289323, epsilon = 1e-5);

        assert_eq!(ease_out_sine(0.0), 0.0);
        assert_eq!(ease_out_sine(1.0), 1.0);
        assert_abs_diff_eq!(ease_out_sine(0.5), 0.70710677, epsilon = 1e-5);

        assert_eq!(ease_in_out_sine(0.0), 0.0);
        assert_eq!(ease_in_out_sine(1.0), 1.0);
        assert_eq!(ease_in_out_sine(0.5), 0.5);
    }

    #[test]
    fn test_ease_quad() {
        assert_eq!(ease_in_quad(0.0), 0.0);
        assert_eq!(ease_in_quad(1.0), 1.0);
        assert_eq!(ease_in_quad(0.5), 0.25);

        assert_eq!(ease_out_quad(0.0), 0.0);
        assert_eq!(ease_out_quad(1.0), 1.0);
        assert_eq!(ease_out_quad(0.5), 0.75);

        assert_eq!(ease_in_out_quad(0.0), 0.0);
        assert_eq!(ease_in_out_quad(1.0), 1.0);
        assert_eq!(ease_in_out_quad(0.5), 0.5);
    }

    #[test]
    fn test_ease_cubic() {
        assert_eq!(ease_in_cubic(0.0), 0.0);
        assert_eq!(ease_in_cubic(1.0), 1.0);
        assert_eq!(ease_in_cubic(0.5), 0.125);

        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);
        assert_eq!(ease_out_cubic(0.5), 0.875);

        assert_eq!(ease_in_out_cubic(0.0), 0.0);
        assert_eq!(ease_in_out_cubic(1.0), 1.0);
        assert_eq!(ease_in_out_cubic(0.5), 0.5);
    }

    #[test]
    fn test_ease_quart() {
        assert_eq!(ease_in_quart(0.0), 0.0);
        assert_eq!(ease_in_quart(1.0), 1.0);
        assert_eq!(ease_in_quart(0.5), 0.0625);

        assert_eq!(ease_out_quart(0.0), 0.0);
        assert_eq!(ease_out_quart(1.0), 1.0);
        assert_eq!(ease_out_quart(0.5), 0.9375);

        assert_eq!(ease_in_out_quart(0.0), 0.0);
        assert_eq!(ease_in_out_quart(1.0), 1.0);
        assert_eq!(ease_in_out_quart(0.5), 0.5);
    }

    #[test]
    fn test_ease_quint() {
        assert_eq!(ease_in_quint(0.0), 0.0);
        assert_eq!(ease_in_quint(1.0), 1.0);
        assert_eq!(ease_in_quint(0.5), 0.03125);

        assert_eq!(ease_out_quint(0.0), 0.0);
        assert_eq!(ease_out_quint(1.0), 1.0);
        assert_eq!(ease_out_quint(0.5), 0.96875);

        assert_eq!(ease_in_out_quint(0.0), 0.0);
        assert_eq!(ease_in_out_quint(1.0), 1.0);
        assert_eq!(ease_in_out_quint(0.5), 0.5);
    }

    #[test]
    fn test_ease_expo() {
        assert_eq!(ease_in_expo(0.0), 0.0);
        assert_eq!(ease_in_expo(1.0), 1.0);
        assert_abs_diff_eq!(ease_in_expo(0.5), 0.03125, epsilon = 1e-5);

        assert_eq!(ease_out_expo(0.0), 0.0);
        assert_eq!(ease_out_expo(1.0), 1.0);
        assert_abs_diff_eq!(ease_out_expo(0.5), 0.96875, epsilon = 1e-5);

        assert_eq!(ease_in_out_expo(0.0), 0.0);
        assert_eq!(ease_in_out_expo(1.0), 1.0);
        assert_eq!(ease_in_out_expo(0.5), 0.5);
    }

    #[test]
    fn test_ease_circ() {
        assert_eq!(ease_in_circ(0.0), 0.0);
        assert_eq!(ease_in_circ(1.0), 1.0);
        assert_abs_diff_eq!(ease_in_circ(0.5), 0.1339746, epsilon = 1e-5);

        assert_eq!(ease_out_circ(0.0), 0.0);
        assert_eq!(ease_out_circ(1.0), 1.0);
        assert_abs_diff_eq!(ease_out_circ(0.5), 0.8660254, epsilon = 1e-5);

        assert_eq!(ease_in_out_circ(0.0), 0.0);
        assert_eq!(ease_in_out_circ(1.0), 1.0);
        assert_eq!(ease_in_out_circ(0.5), 0.5);
    }

    #[test]
    fn test_ease_back() {
        assert_eq!(ease_in_back(0.0), 0.0);
        assert_eq!(ease_in_back(1.0), 1.0);
        assert_abs_diff_eq!(ease_in_back(0.5), -0.08769499, epsilon = 1e-5);

        assert_eq!(ease_out_back(0.0), 0.0);
        assert_eq!(ease_out_back(1.0), 1.0);
        assert_abs_diff_eq!(ease_out_back(0.5), 1.087695, epsilon = 1e-5);

        assert_eq!(ease_in_out_back(0.0), 0.0);
        assert_eq!(ease_in_out_back(1.0), 1.0);
        assert_eq!(ease_in_out_back(0.5), 0.5);
    }

    #[test]
    fn test_ease_elastic() {
        assert_eq!(ease_in_elastic(0.0), 0.0);
        assert_eq!(ease_in_elastic(1.0), 1.0);
        assert_abs_diff_eq!(ease_in_elastic(0.5), -0.015625, epsilon = 1e-5);

        assert_eq!(ease_out_elastic(0.0), 0.0);
        assert_eq!(ease_out_elastic(1.0), 1.0);
        assert_abs_diff_eq!(ease_out_elastic(0.5), 1.015625, epsilon = 1e-5);

        assert_eq!(ease_in_out_elastic(0.0), 0.0);
        assert_eq!(ease_in_out_elastic(1.0), 1.0);
        assert_eq!(ease_in_out_elastic(0.5), 0.5);
    }

    #[test]
    fn test_ease_bounce() {
        assert_eq!(ease_in_bounce(0.0), 0.0);
        assert_eq!(ease_in_bounce(1.0), 1.0);
        assert_abs_diff_eq!(ease_in_bounce(0.5), 0.234375, epsilon = 1e-5);

        assert_eq!(ease_out_bounce(0.0), 0.0);
        assert_eq!(ease_out_bounce(1.0), 1.0);
        assert_abs_diff_eq!(ease_out_bounce(0.5), 0.765625, epsilon = 1e-5);

        assert_eq!(ease_in_out_bounce(0.0), 0.0);
        assert_eq!(ease_in_out_bounce(1.0), 1.0);
        assert_eq!(ease_in_out_bounce(0.5), 0.5);
    }
}
