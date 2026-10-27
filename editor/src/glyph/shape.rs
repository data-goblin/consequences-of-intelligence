use super::*;
use std::fmt::Write;

pub(super) type Pt = [f64; 2];

pub(super) fn blob_pts(
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
    rnd: &mut Rng,
    n: usize,
    j: f64,
) -> Vec<Pt> {
    (0..n)
        .map(|i| {
            let a = i as f64 / n as f64 * PI * 2.0;
            let k = 1.0 + (rnd.next() - 0.5) * 2.0 * j;
            [cx + a.cos() * rx * k, cy + a.sin() * ry * k]
        })
        .collect()
}

#[derive(Default)]
pub(super) struct CatmullRom {
    out: String,
    closed: bool,
    point: u8,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    x3: f64,
    y3: f64,
    x4: f64,
    y4: f64,
    x5: f64,
    y5: f64,
    l01_a: f64,
    l12_a: f64,
    l23_a: f64,
    l01_2a: f64,
    l12_2a: f64,
    l23_2a: f64,
}

impl CatmullRom {
    pub(super) fn new(closed: bool) -> Self {
        Self {
            closed,
            x0: f64::NAN,
            y0: f64::NAN,
            x1: f64::NAN,
            y1: f64::NAN,
            x2: f64::NAN,
            y2: f64::NAN,
            x3: f64::NAN,
            y3: f64::NAN,
            x4: f64::NAN,
            y4: f64::NAN,
            x5: f64::NAN,
            y5: f64::NAN,
            ..Default::default()
        }
    }

    pub(super) fn move_to(&mut self, x: f64, y: f64) {
        let _ = write!(self.out, "M{},{}", r3(x), r3(y));
    }

    pub(super) fn line_to(&mut self, x: f64, y: f64) {
        let _ = write!(self.out, "L{},{}", r3(x), r3(y));
    }

    pub(super) fn bezier(&mut self, x: f64, y: f64) {
        let (mut x1, mut y1, mut x2, mut y2) = (self.x1, self.y1, self.x2, self.y2);
        if self.l01_a > 1e-12 {
            let a = 2.0 * self.l01_2a + 3.0 * self.l01_a * self.l12_a + self.l12_2a;
            let n = 3.0 * self.l01_a * (self.l01_a + self.l12_a);
            x1 = (x1 * a - self.x0 * self.l12_2a + self.x2 * self.l01_2a) / n;
            y1 = (y1 * a - self.y0 * self.l12_2a + self.y2 * self.l01_2a) / n;
        }
        if self.l23_a > 1e-12 {
            let b = 2.0 * self.l23_2a + 3.0 * self.l23_a * self.l12_a + self.l12_2a;
            let m = 3.0 * self.l23_a * (self.l23_a + self.l12_a);
            x2 = (x2 * b + self.x1 * self.l23_2a - x * self.l12_2a) / m;
            y2 = (y2 * b + self.y1 * self.l23_2a - y * self.l12_2a) / m;
        }
        let _ = write!(
            self.out,
            "C{},{},{},{},{},{}",
            r3(x1),
            r3(y1),
            r3(x2),
            r3(y2),
            r3(self.x2),
            r3(self.y2)
        );
    }

    pub(super) fn push(&mut self, x: f64, y: f64) {
        if self.point > 0 {
            let (x23, y23) = (self.x2 - x, self.y2 - y);
            self.l23_2a = (x23 * x23 + y23 * y23).powf(0.5);
            self.l23_a = self.l23_2a.sqrt();
        }
        if self.closed {
            match self.point {
                0 => {
                    self.point = 1;
                    self.x3 = x;
                    self.y3 = y;
                }
                1 => {
                    self.point = 2;
                    self.x4 = x;
                    self.y4 = y;
                    self.move_to(x, y);
                }
                2 => {
                    self.point = 3;
                    self.x5 = x;
                    self.y5 = y;
                }
                _ => self.bezier(x, y),
            }
        } else {
            match self.point {
                0 => {
                    self.point = 1;
                    self.move_to(x, y);
                }
                1 => self.point = 2,
                2 => {
                    self.point = 3;
                    self.bezier(x, y);
                }
                _ => self.bezier(x, y),
            }
        }
        self.l01_a = self.l12_a;
        self.l12_a = self.l23_a;
        self.l01_2a = self.l12_2a;
        self.l12_2a = self.l23_2a;
        self.x0 = self.x1;
        self.x1 = self.x2;
        self.x2 = x;
        self.y0 = self.y1;
        self.y1 = self.y2;
        self.y2 = y;
    }

    pub(super) fn end(mut self) -> String {
        if self.closed {
            match self.point {
                1 => {
                    self.move_to(self.x3, self.y3);
                    self.out.push('Z');
                }
                2 => {
                    self.line_to(self.x3, self.y3);
                    self.out.push('Z');
                }
                3 => {
                    let (x3, y3, x4, y4, x5, y5) =
                        (self.x3, self.y3, self.x4, self.y4, self.x5, self.y5);
                    self.push(x3, y3);
                    self.push(x4, y4);
                    self.push(x5, y5);
                }
                _ => {}
            }
        } else {
            match self.point {
                1 => self.out.push('Z'),
                2 => self.line_to(self.x2, self.y2),
                3 => {
                    let (x2, y2) = (self.x2, self.y2);
                    self.push(x2, y2);
                }
                _ => {}
            }
        }
        self.out
    }
}

pub(super) fn curve(pts: &[Pt], closed: bool) -> String {
    let mut c = CatmullRom::new(closed);
    for p in pts {
        c.push(p[0], p[1]);
    }
    c.end()
}

pub(super) fn closed(pts: &[Pt]) -> String {
    curve(pts, true)
}

pub(super) fn open(pts: &[Pt]) -> String {
    curve(pts, false)
}

pub(super) type Attrs<'a> = &'a [(&'a str, String)];

pub(super) fn at(k: &str, v: impl ToString) -> (&str, String) {
    (k, v.to_string())
}

pub(super) struct Svg {
    pub(super) out: String,
}

impl Svg {
    pub(super) fn attrs(&mut self, attrs: Attrs) {
        for (k, v) in attrs {
            let _ = write!(self.out, " {k}=\"{v}\"");
        }
    }

    pub(super) fn el(&mut self, name: &str, attrs: Attrs) {
        let _ = write!(self.out, "<{name}");
        self.attrs(attrs);
        self.out.push_str("/>");
    }

    pub(super) fn path(&mut self, d: &str, attrs: Attrs) {
        let _ = write!(self.out, "<path d=\"{d}\"");
        self.attrs(attrs);
        self.out.push_str("/>");
    }

    pub(super) fn open_g(&mut self, attrs: Attrs) {
        self.out.push_str("<g");
        self.attrs(attrs);
        self.out.push('>');
    }

    pub(super) fn close_g(&mut self) {
        self.out.push_str("</g>");
    }
}
