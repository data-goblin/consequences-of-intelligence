use super::*;

pub(super) fn petal_path(l: f64, w: f64, rnd: &mut Rng, pointy: bool) -> String {
    let mut j = || (rnd.next() - 0.5) * w * 0.3;
    let tip = if pointy {
        format!("{} {} 0 {}", w * 0.35 + j(), -l * 0.8, -l)
    } else {
        format!("{} {} 0 {}", w * 0.95 + j(), -l * 1.05, -l)
    };
    let tip2 = if pointy {
        format!("{} {}", -w * 0.35 + j(), -l * 0.8)
    } else {
        format!("{} {}", -w * 0.95 + j(), -l * 1.05)
    };
    let c1 = w + j();
    let c2 = -w + j();
    format!(
        "M0 0 C{} {} {tip} C{tip2} {} {} 0 0Z",
        c1,
        -l * 0.22,
        c2,
        -l * 0.22
    )
}

pub(super) fn flower(s: &mut Svg, r: f64, v: f64, rnd: &mut Rng) {
    let e = intensity(v);
    let sw = (r * 0.07).clamp(0.45, 0.8);
    let so = 0.35 + 0.3 * (e * 2.5).min(1.0);
    let hue = PETAL_INK[(rnd.next() * 7.0).floor() as usize];
    let hue2 = PETAL_INK[(rnd.next() * 7.0).floor() as usize];
    if e < 0.07 {
        let tilt = (rnd.next() - 0.5) * 30.0;
        s.open_g(&[
            at("transform", format!("rotate({tilt})")),
            at("opacity", 0.5 + e * 5.0),
        ]);
        s.path(
            &open(&[[0.0, r * 0.85], [r * 0.06, r * 0.35], [0.0, r * 0.05]]),
            &[
                at("fill", "none"),
                at("stroke", INK),
                at("stroke-width", sw * 0.8),
            ],
        );
        let w = r * (0.26 + rnd.next() * 0.12);
        let pointy = rnd.next() > 0.5;
        let d = petal_path(r * 0.75, w, rnd, pointy);
        s.path(
            &d,
            &[
                at("transform", format!("translate(0 {})", r * 0.25)),
                at("fill", BONE),
                at("stroke", INK),
                at("stroke-width", sw),
            ],
        );
        if e > 0.03 {
            let d = petal_path(r * 0.6, r * 0.2, rnd, true);
            let rot = (rnd.next() - 0.5) * 40.0;
            s.path(
                &d,
                &[
                    at(
                        "transform",
                        format!("translate(0 {}) rotate({rot})", r * 0.25),
                    ),
                    at("fill", hue),
                    at("fill-opacity", 0.25),
                    at("stroke", INK),
                    at("stroke-width", sw * 0.8),
                ],
            );
        }
        s.close_g();
        return;
    }
    let wash = |s: &mut Svg, d: &str, extra: Option<(&str, String)>, col: &str, op: f64| {
        let mut a: Vec<(&str, String)> = extra.into_iter().collect();
        a.extend([
            at("fill", col),
            at("fill-opacity", op),
            at("stroke", INK),
            at("stroke-opacity", so),
            at("stroke-width", sw),
        ]);
        s.path(d, &a);
    };
    if e > 0.3 {
        for sd in [-1.0, 1.0] {
            if rnd.next() < 0.25 {
                continue;
            }
            let a = 30.0 + rnd.next() * 40.0;
            let d = format!(
                "M0 0 C{} {} {} {} {} {} C{} {} {} {} 0 0Z",
                sd * r * 0.3,
                r * 0.1,
                sd * r * 0.8,
                r * 0.25,
                sd * r * 1.15,
                r * 0.05,
                sd * r * 0.8,
                r * 0.55,
                sd * r * 0.35,
                r * 0.5
            );
            wash(
                s,
                &d,
                Some(at("transform", format!("rotate({})", sd * (a - 45.0)))),
                "#6f9a52",
                0.18 + 0.45 * e,
            );
        }
    }
    if e > 0.12 {
        s.open_g(&[at("filter", "url(#wc)")]);
        let count = 2 + js_round(e * 4.0) as usize;
        for k in 0..count {
            let cx = (rnd.next() - 0.5) * r * 0.6;
            let cy = (rnd.next() - 0.5) * r * 0.6;
            let rx = r * (0.6 + 0.5 * rnd.next());
            let ry = r * (0.6 + 0.5 * rnd.next());
            let pts = blob_pts(cx, cy, rx, ry, rnd, 8, 0.32);
            s.path(
                &closed(&pts),
                &[
                    at("fill", if k % 2 == 1 { hue2 } else { hue }),
                    at("opacity", 0.1 + 0.3 * e),
                ],
            );
        }
        s.close_g();
    }
    let n = 4 + (rnd.next() * 3.0).floor() as usize + js_round(e * 4.0) as usize;
    let pointy = rnd.next() > 0.55;
    let l = r * (0.55 + 0.45 * e.sqrt());
    let w = l * (0.3 + 0.2 * rnd.next() + 0.15 * e);
    let off = rnd.next() * 360.0;
    let layer = |s: &mut Svg,
                 rnd: &mut Rng,
                 len: f64,
                 wid: f64,
                 rot: f64,
                 count: usize,
                 col: &str,
                 op: f64| {
        for i in 0..count {
            let a = off + rot + i as f64 * 360.0 / count as f64 + (rnd.next() - 0.5) * 14.0;
            let plen = len * (0.9 + rnd.next() * 0.2);
            let d = petal_path(plen, wid, rnd, pointy);
            wash(
                s,
                &d,
                Some(at("transform", format!("rotate({a})"))),
                col,
                op,
            );
            if e > 0.4 && r > 8.0 {
                s.path(
                    &format!("M0 {} L0 {}", -len * 0.2, -len * 0.65),
                    &[
                        at("transform", format!("rotate({a})")),
                        at("stroke", INK),
                        at("stroke-width", 0.45),
                        at("opacity", 0.4),
                    ],
                );
            }
        }
    };
    layer(s, rnd, l, w, 0.0, n, hue, 0.1 + 0.45 * e);
    if e > 0.4 && r > 7.0 {
        layer(
            s,
            rnd,
            l * 0.68,
            w * 0.85,
            180.0 / n as f64,
            n,
            if e > 0.7 { hue2 } else { hue },
            0.15 + 0.45 * e,
        );
    }
    if e > 0.78 && r > 9.0 {
        layer(
            s,
            rnd,
            l * 0.4,
            w * 0.6,
            90.0 / n as f64,
            n.saturating_sub(2).max(4),
            hue,
            0.35 + 0.35 * e,
        );
    }
    let c = r * (0.13 + 0.1 * e);
    let pts = blob_pts(0.0, 0.0, c, c, rnd, 7, 0.15);
    wash(s, &closed(&pts), None, "#e8cf6a", 0.45 + 0.4 * e);
    if e > 0.35 {
        let count = 3 + js_round(e * 5.0) as usize;
        for _ in 0..count {
            let a = rnd.next() * PI * 2.0;
            let d = rnd.next() * c * 0.6;
            s.el(
                "circle",
                &[
                    at("cx", a.cos() * d),
                    at("cy", a.sin() * d),
                    at("r", 0.5),
                    at("fill", INK),
                ],
            );
        }
    }
}
