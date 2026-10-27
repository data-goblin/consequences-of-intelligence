mod color;
mod flower;
mod shape;
mod skull;

use color::*;
use flower::*;
use shape::*;
use skull::*;
use std::f64::consts::PI;
use std::fmt::Write;

const INK: &str = "#2a2118";
const BONE: &str = "#f3ead3";
const PETAL_INK: [&str; 7] = [
    "#9a7fd1", "#6fa3d8", "#e3c55a", "#cf8fc6", "#76bfb2", "#a9b8e8", "#b9d27a",
];

pub struct Glyph<'a> {
    pub valence: f64,
    pub theme: &'a str,
    pub evidence: &'a str,
    pub seed: u32,
    pub radius: f64,
    pub tint: bool,
}

pub fn seed_of(id: &str) -> u32 {
    let mut h: u32 = 7;
    for ch in id.chars() {
        let mut buf = [0u16; 2];
        let unit = ch.encode_utf16(&mut buf)[0];
        h = h.wrapping_mul(31).wrapping_add(unit as u32);
    }
    h % 2147483646 + 1
}

pub fn chart_radius(probability: f64, stage: &str) -> f64 {
    let certainty = if stage == "active" { 1.0 } else { probability };
    4.5 + certainty * 5.0
}

struct Rng(u64);

impl Rng {
    fn new(seed: u32) -> Self {
        let s = seed as u64 % 2147483647;
        Rng(if s == 0 { 1 } else { s })
    }

    fn next(&mut self) -> f64 {
        self.0 = self.0 * 16807 % 2147483647;
        self.0 as f64 / 2147483647.0
    }
}

fn theme_ink(theme: &str) -> &'static str {
    match theme {
        "ip" => "#c28e45",
        "power" => "#b06a83",
        "mind" => "#8f74b0",
        "security" => "#6782a8",
        "information" => "#4e9c9c",
        "world" => "#74a05a",
        "work" => "#a9a23f",
        _ => "#999",
    }
}

fn haze(evidence: &str) -> [f64; 3] {
    match evidence {
        "reported" => [1.5, 0.24, 0.2],
        "projected" => [1.95, 0.19, 0.28],
        "speculative" => [2.5, 0.15, 0.36],
        _ => [1.1, 0.3, 0.12],
    }
}

fn intensity(v: f64) -> f64 {
    (v.abs() / 5.0).min(1.0).powf(1.7)
}

fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

fn r3(x: f64) -> f64 {
    let v = (x * 1000.0).round() / 1000.0;
    if v == 0.0 { 0.0 } else { v }
}

fn tint(s: &mut Svg, r: f64, g: &Glyph, rnd: &mut Rng) {
    let [k, op, j] = haze(g.evidence);
    let cx = (rnd.next() - 0.5) * r * 0.3;
    let cy = (rnd.next() - 0.5) * r * 0.3;
    let pts = blob_pts(cx, cy, r * k, r * k * 0.91, rnd, 9, j);
    let fill = if g.valence <= -0.5 {
        "#8c8c8c"
    } else {
        theme_ink(g.theme)
    };
    for f in [1.0, 0.72, 0.46] {
        let scaled: Vec<Pt> = pts
            .iter()
            .map(|&[x, y]| [cx + (x - cx) * f, cy + (y - cy) * f])
            .collect();
        s.path(
            &closed(&scaled),
            &[at("fill", fill), at("opacity", op * 0.55)],
        );
    }
}

fn extent(g: &Glyph) -> f64 {
    let r = g.radius;
    let e = intensity(g.valence);
    let art = if g.valence >= 0.5 {
        r * 1.8 + 4.2
    } else if g.valence <= -0.5 {
        let rr = r * 0.82;
        let mut x = rr * 1.4;
        if e > 0.52 {
            x = x.max(rr * ((1.45 + 0.5 * e) * 1.28 + 0.25));
        }
        x
    } else {
        r * 1.0
    };
    let haze = if g.tint {
        let [k, _, j] = haze(g.evidence);
        r * (k * (1.0 + j) + 0.15)
    } else {
        0.0
    };
    art.max(haze) + 1.0
}

pub fn svg(g: &Glyph, size_px: f64) -> String {
    let h = extent(g);
    let mut s = Svg { out: String::new() };
    let _ = write!(
        s.out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size_px}\" height=\"{size_px}\" viewBox=\"{} {} {} {}\">",
        -h,
        -h,
        2.0 * h,
        2.0 * h
    );
    if g.valence >= 0.5 {
        s.out.push_str("<defs><filter id=\"wc\" x=\"-30%\" y=\"-30%\" width=\"160%\" height=\"160%\"><feGaussianBlur stdDeviation=\"1.4\"/></filter></defs>");
    }
    s.open_g(&[]);
    let mut rnd = Rng::new(g.seed);
    if g.tint {
        tint(&mut s, g.radius, g, &mut rnd);
    }
    if g.valence >= 0.5 {
        flower(&mut s, g.radius, g.valence, &mut rnd);
    } else if g.valence <= -0.5 {
        skull(&mut s, g.radius, g.valence, &mut rnd);
    } else {
        sprout(&mut s, g.radius, g.valence, &mut rnd);
    }
    s.close_g();
    s.out.push_str("</svg>");
    s.out
}
