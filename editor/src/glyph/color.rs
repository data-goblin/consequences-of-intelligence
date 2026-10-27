use super::*;

pub(super) fn rgb2lrgb(x: f64) -> f64 {
    let x = x / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

pub(super) fn lrgb2rgb(x: f64) -> f64 {
    255.0
        * if x <= 0.0031308 {
            12.92 * x
        } else {
            1.055 * x.powf(1.0 / 2.4) - 0.055
        }
}

pub(super) const T0: f64 = 4.0 / 29.0;
pub(super) const T1: f64 = 6.0 / 29.0;
pub(super) const T2: f64 = 3.0 * T1 * T1;
pub(super) const T3: f64 = T1 * T1 * T1;
pub(super) const XN: f64 = 0.96422;
pub(super) const ZN: f64 = 0.82521;

pub(super) fn xyz2lab(t: f64) -> f64 {
    if t > T3 {
        t.powf(1.0 / 3.0)
    } else {
        t / T2 + T0
    }
}

pub(super) fn lab2xyz(t: f64) -> f64 {
    if t > T1 { t * t * t } else { T2 * (t - T0) }
}

pub(super) fn hex_rgb(hex: &str) -> [f64; 3] {
    let h = hex.trim_start_matches('#');
    let p = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).unwrap_or(0) as f64;
    [p(0), p(2), p(4)]
}

pub(super) fn to_lab(hex: &str) -> [f64; 3] {
    let [r8, g8, b8] = hex_rgb(hex);
    let (r, g, b) = (rgb2lrgb(r8), rgb2lrgb(g8), rgb2lrgb(b8));
    let y = xyz2lab(0.2225045 * r + 0.7168786 * g + 0.0606169 * b);
    let (x, z) = if r8 == g8 && g8 == b8 {
        (y, y)
    } else {
        (
            xyz2lab((0.4360747 * r + 0.3850649 * g + 0.1430804 * b) / XN),
            xyz2lab((0.0139322 * r + 0.0971045 * g + 0.7141733 * b) / ZN),
        )
    };
    [116.0 * y - 16.0, 500.0 * (x - y), 200.0 * (y - z)]
}

pub(super) fn lab_rgb([l, a, b]: [f64; 3]) -> String {
    let y = (l + 16.0) / 116.0;
    let x = XN * lab2xyz(y + a / 500.0);
    let z = ZN * lab2xyz(y - b / 200.0);
    let y = lab2xyz(y);
    let c = |v: f64| js_round(lrgb2rgb(v)).clamp(0.0, 255.0) as u8;
    format!(
        "rgb({}, {}, {})",
        c(3.1338561 * x - 1.6168667 * y - 0.4906146 * z),
        c(-0.9787684 * x + 1.9161415 * y + 0.0334540 * z),
        c(0.0719453 * x - 0.2289914 * y + 1.4052427 * z)
    )
}

pub(super) fn interpolate_lab(a: &str, b: &str, t: f64) -> String {
    let (la, lb) = (to_lab(a), to_lab(b));
    lab_rgb([
        la[0] + (lb[0] - la[0]) * t,
        la[1] + (lb[1] - la[1]) * t,
        la[2] + (lb[2] - la[2]) * t,
    ])
}
