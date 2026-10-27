use super::*;

pub(super) fn skull(s: &mut Svg, r: f64, v: f64, rnd: &mut Rng) {
    let e = intensity(v);
    let sw = (r * 0.09).clamp(0.5, 1.0);
    let rr = r * 0.82;
    let ink = interpolate_lab("#777777", "#161616", (e * 1.6).min(1.0));
    let bone = interpolate_lab("#ecebe8", "#5c5c5c", e.powf(1.1));
    let tilt = (rnd.next() - 0.5) * (14.0 - 10.0 * e);
    let jw = 0.4 + rnd.next() * 0.14;
    let sq = 0.88 + rnd.next() * 0.16;
    if e < 0.06 {
        s.open_g(&[
            at("transform", format!("rotate({tilt})")),
            at("opacity", 0.55 + e * 6.0),
        ]);
        let pts = blob_pts(0.0, 0.0, rr * 0.85, rr * 0.75 * sq, rnd, 8, 0.12);
        s.path(
            &closed(&pts),
            &[
                at("fill", "#ecebe8"),
                at("stroke", &ink),
                at("stroke-width", sw * 0.8),
            ],
        );
        if e > 0.02 {
            for k in [-1.0, 1.0] {
                s.el(
                    "circle",
                    &[
                        at("cx", k * rr * 0.28),
                        at("cy", -rr * 0.02),
                        at("r", rr * 0.1),
                        at("fill", &ink),
                        at("opacity", 0.35),
                    ],
                );
            }
        }
        s.close_g();
        return;
    }
    if e > 0.52 {
        for _ in 0..3 {
            let cx = (rnd.next() - 0.5) * rr * 0.4;
            let cy = rr * 0.1 + (rnd.next() - 0.5) * rr * 0.3;
            let rx = rr * (1.25 + 0.5 * e + rnd.next() * 0.2);
            let pts = blob_pts(cx, cy, rx, rr * (1.15 + 0.4 * e), rnd, 10, 0.28);
            s.path(
                &closed(&pts),
                &[
                    at("fill", "#1a1a1a"),
                    at("opacity", 0.07 + 0.3 * (e - 0.52)),
                ],
            );
        }
    }
    if e > 0.82 {
        for k in [-1.0, 1.0] {
            let (x1, y1, x2, y2) = (-k * rr * 1.1, -rr * 0.5, k * rr * 1.1, rr * 1.25);
            let len = (x2 - x1).hypot(y2 - y1);
            let px = -(y2 - y1) / len * rr * 0.13;
            let py = (x2 - x1) / len * rr * 0.13;
            let line = |s: &mut Svg, col: &str, w: f64| {
                s.el(
                    "line",
                    &[
                        at("x1", x1),
                        at("y1", y1),
                        at("x2", x2),
                        at("y2", y2),
                        at("stroke", col),
                        at("stroke-width", w),
                        at("stroke-linecap", "round"),
                    ],
                );
            };
            line(s, &ink, rr * 0.3);
            line(s, &bone, rr * 0.3 - 1.8);
            for (x, y) in [(x1, y1), (x2, y2)] {
                for o in [-1.0, 1.0] {
                    s.el(
                        "circle",
                        &[
                            at("cx", x + o * px),
                            at("cy", y + o * py),
                            at("r", rr * 0.16),
                            at("fill", &bone),
                            at("stroke", &ink),
                            at("stroke-width", sw * 0.8),
                        ],
                    );
                }
            }
        }
    }
    s.open_g(&[at("transform", format!("rotate({tilt})"))]);
    let mut outline: Vec<Pt> = (0..9)
        .map(|i| {
            let a = PI * (0.92 + i as f64 / 8.0 * 1.16);
            let k = 1.0 + (rnd.next() - 0.5) * (0.06 + 0.1 * e);
            [a.cos() * rr * k, -rr * 0.12 + a.sin() * rr * 0.95 * sq * k]
        })
        .collect();
    outline.extend([
        [rr * 0.66, rr * 0.42],
        [rr * (jw + 0.08), rr * 0.62],
        [rr * jw, rr * 0.92],
        [0.0, rr],
        [-rr * jw, rr * 0.92],
        [-rr * (jw + 0.08), rr * 0.62],
        [-rr * 0.66, rr * 0.42],
    ]);
    s.path(
        &closed(&outline),
        &[
            at("fill", &bone),
            at("stroke", &ink),
            at("stroke-width", sw),
        ],
    );
    let eye = rr * (0.16 + 0.16 * e);
    for k in [-1.0, 1.0] {
        let (ex, ey) = (k * rr * 0.34, rr * 0.12);
        let ry = eye * (0.75 + 0.3 * rnd.next());
        let pts = blob_pts(ex, ey, eye, ry, rnd, 7, 0.1 + 0.2 * e);
        s.path(
            &closed(&pts),
            &[
                at("fill", "#0d0d0d"),
                at("opacity", 0.25 + 0.75 * (e * 1.5).min(1.0)),
            ],
        );
        if e > 0.55 {
            s.el(
                "circle",
                &[
                    at("cx", ex + (rnd.next() - 0.5) * eye * 0.4),
                    at("cy", ey),
                    at("r", (eye * 0.22).max(0.6)),
                    at("fill", if e > 0.8 { "#e2e2e2" } else { "#b4b4b4" }),
                    at("opacity", 0.9),
                ],
            );
        }
        if e > 0.62 {
            let mid = ex + (rnd.next() - 0.5) * 2.0;
            s.path(
                &open(&[
                    [ex, ey + eye * 0.8],
                    [mid, ey + eye * 1.6],
                    [ex, ey + eye * 2.2],
                ]),
                &[
                    at("fill", "none"),
                    at("stroke", "#0d0d0d"),
                    at("stroke-width", sw * 1.2),
                    at("stroke-linecap", "round"),
                    at("opacity", 0.7),
                ],
            );
        }
    }
    if e > 0.74 {
        let pts = blob_pts(0.0, -rr * 0.42, eye * 0.6, eye * 0.75, rnd, 6, 0.15);
        s.path(&closed(&pts), &[at("fill", "#0d0d0d")]);
        s.el(
            "circle",
            &[
                at("cx", 0),
                at("cy", -rr * 0.42),
                at("r", (eye * 0.18).max(0.6)),
                at("fill", "#e2e2e2"),
            ],
        );
    }
    s.path(
        &format!(
            "M0 {} L{} {} L{} {}Z",
            rr * 0.34,
            rr * 0.09,
            rr * 0.52,
            -rr * 0.09,
            rr * 0.52
        ),
        &[
            at("fill", "#0d0d0d"),
            at("opacity", 0.2 + 0.7 * (e * 1.5).min(1.0)),
        ],
    );
    if e > 0.18 {
        s.path(
            &format!("M{} {} L{} {}", -rr * 0.34, rr * 0.72, rr * 0.34, rr * 0.72),
            &[
                at("stroke", &ink),
                at("stroke-width", 0.6),
                at("opacity", 0.4 + 0.6 * e),
            ],
        );
        let nt = 3 + js_round(e * 4.0) as usize;
        for i in 0..nt {
            let x = (-0.3 + 0.6 * i as f64 / (nt - 1) as f64) * rr;
            let h = if e > 0.6 {
                rr * (0.12 + rnd.next() * 0.14)
            } else {
                rr * 0.13
            };
            let d = if e > 0.6 {
                format!(
                    "M{} {} L{} {} L{} {}",
                    x - rr * 0.04,
                    rr * 0.72,
                    x,
                    rr * 0.72 + h,
                    x + rr * 0.04,
                    rr * 0.72
                )
            } else {
                format!("M{} {} L{} {}", x, rr * 0.64, x, rr * 0.88)
            };
            s.path(
                &d,
                &[
                    at("fill", if e > 0.6 { bone.as_str() } else { "none" }),
                    at("stroke", &ink),
                    at("stroke-width", 0.6),
                    at("opacity", 0.4 + 0.6 * e),
                ],
            );
        }
    }
    if e > 0.4 {
        let count = 1 + js_round(e * 2.0) as usize;
        for _ in 0..count {
            let x0 = (rnd.next() - 0.5) * rr * 0.9;
            let p1 = x0 + (rnd.next() - 0.5) * rr * 0.3;
            let p2 = x0 + (rnd.next() - 0.5) * rr * 0.3;
            let p3 = x0 + (rnd.next() - 0.5) * rr * 0.4;
            s.path(
                &open(&[
                    [x0, -rr * 1.05],
                    [p1, -rr * 0.82],
                    [p2, -rr * 0.6],
                    [p3, -rr * 0.38],
                ]),
                &[
                    at("fill", "none"),
                    at("stroke", &ink),
                    at("stroke-width", 0.75),
                ],
            );
        }
    }
    s.close_g();
}

pub(super) fn sprout(s: &mut Svg, r: f64, v: f64, rnd: &mut Rng) {
    let rot = (rnd.next() - 0.5) * 30.0;
    s.open_g(&[
        at("opacity", 0.6),
        at("transform", format!("rotate({rot})")),
    ]);
    let pts = blob_pts(0.0, 0.0, r * 0.55, r * 0.42, rnd, 7, 0.12);
    s.path(
        &closed(&pts),
        &[
            at("fill", BONE),
            at("stroke", "#8a7b66"),
            at("stroke-width", 0.8),
        ],
    );
    if v >= 0.0 {
        s.path(
            &open(&[[0.0, -r * 0.3], [r * 0.1, -r * 0.65], [r * 0.3, -r * 0.8]]),
            &[
                at("fill", "none"),
                at("stroke", "#6f9a52"),
                at("stroke-width", 0.9),
            ],
        );
    } else {
        s.path(
            &open(&[[-r * 0.2, -r * 0.1], [0.0, r * 0.1], [r * 0.15, -r * 0.05]]),
            &[
                at("fill", "none"),
                at("stroke", "#8a7b66"),
                at("stroke-width", 0.7),
            ],
        );
    }
    s.close_g();
}
