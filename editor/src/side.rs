use crate::app::{Editor, node_value};
use crate::graph::{self, EdgeKind};
use crate::style::*;
use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::{h_flex, v_flex};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

const GW: f32 = 340.;
const GH: f32 = 400.;
const PAD: f32 = 34.;

fn at(p: (f32, f32)) -> (f32, f32) {
    (PAD + p.0 * (GW - 2. * PAD), PAD + p.1 * (GH - 2. * PAD))
}

fn edge_color(k: EdgeKind) -> Rgba {
    match k {
        EdgeKind::Leads => lead(),
        EdgeKind::Counters => counter(),
        EdgeKind::Twin => twin(),
        EdgeKind::Shared(_) => rule(),
    }
}

impl Editor {
    fn graph_for(&self, i: usize) -> Rc<graph::Graph> {
        let mut h = DefaultHasher::new();
        i.hash(&mut h);
        for l in self.store.links() {
            for k in ["source", "target", "kind"] {
                l[k].as_str().hash(&mut h);
            }
        }
        for n in self.store.nodes() {
            for r in n["refs"].as_array().into_iter().flatten() {
                r["url"].as_str().hash(&mut h);
            }
        }
        let fp = h.finish();
        let mut cache = self.graph_cache.borrow_mut();
        match &*cache {
            Some((k, g)) if *k == fp => g.clone(),
            _ => {
                let g = Rc::new(graph::build(&self.store, &self.store.shared_refs(), i));
                *cache = Some((fp, g.clone()));
                g
            }
        }
    }

    pub fn render_side(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(i) = self.selected.filter(|&i| i < self.store.nodes().len()) else {
            return div().w(px(380.)).into_any_element();
        };
        let n = node_value(&self.store, i);
        let v = n["valence"].as_f64().unwrap_or(0.0);
        let p = n["probability"].as_f64().unwrap_or(0.0);
        let word = if n.get("kind").is_some() {
            "hidden from chart"
        } else if v >= 0.5 {
            "bloom"
        } else if v <= -0.5 {
            "doom"
        } else {
            "sprout"
        };
        let preview =
            v_flex()
                .items_center()
                .gap_1()
                .pt_4()
                .child(img(self.glyph(n, 230., true)).size(px(230.)))
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(img(self.glyph(n, 44., true)).size(px(44.)))
                        .child(
                            v_flex()
                                .child(div().text_sm().text_color(tone(v)).child(format!(
                                    "{word} {}{v:.2}",
                                    if v > 0. { "+" } else { "" }
                                )))
                                .child(div().text_xs().text_color(ink_soft()).child(format!(
                                    "{:.0}%  |  {} {}  |  {}",
                                    p * 100.,
                                    n["year"].as_i64().unwrap_or_default(),
                                    n["stage"].as_str().unwrap_or_default(),
                                    n["evidence"].as_str().unwrap_or_default()
                                ))),
                        ),
                );

        let g = self.graph_for(i);
        let pts: Vec<(f32, f32)> = g.pos.iter().map(|&q| at(q)).collect();
        let edges: Vec<(usize, usize, EdgeKind)> =
            g.edges.iter().map(|e| (e.from, e.to, e.kind)).collect();
        let radius: Vec<f32> = (0..g.nodes.len())
            .map(|s| if s == 0 { 22. } else { 13. })
            .collect();
        let (paint_pts, paint_r) = (pts.clone(), radius.clone());
        let lines = canvas(
            |_, _, _| {},
            move |b, _, window, _| {
                let o = b.origin;
                let pt = |q: (f32, f32)| point(o.x + px(q.0), o.y + px(q.1));
                for &(a, c, kind) in &edges {
                    if a == c {
                        continue;
                    }
                    let (pa, pc) = (paint_pts[a], paint_pts[c]);
                    let (dx, dy) = (pc.0 - pa.0, pc.1 - pa.1);
                    let len = (dx * dx + dy * dy).sqrt().max(1.);
                    let (ux, uy) = (dx / len, dy / len);
                    let start = (pa.0 + ux * paint_r[a], pa.1 + uy * paint_r[a]);
                    let end = (pc.0 - ux * (paint_r[c] + 2.), pc.1 - uy * (paint_r[c] + 2.));
                    let width = match kind {
                        EdgeKind::Shared(k) => 0.8 + 0.5 * (k.min(4) as f32),
                        _ => 1.4,
                    };
                    let mut pb = PathBuilder::stroke(px(width));
                    if matches!(kind, EdgeKind::Shared(_) | EdgeKind::Twin) {
                        pb = pb.dash_array(&[px(4.), px(3.)]);
                    }
                    pb.move_to(pt(start));
                    pb.line_to(pt(end));
                    if let Ok(path) = pb.build() {
                        window.paint_path(path, edge_color(kind));
                    }
                    if matches!(kind, EdgeKind::Leads | EdgeKind::Counters) {
                        let (bx, by) = (end.0 - ux * 7., end.1 - uy * 7.);
                        let mut head = PathBuilder::fill();
                        head.move_to(pt(end));
                        head.line_to(pt((bx - uy * 3.5, by + ux * 3.5)));
                        head.line_to(pt((bx + uy * 3.5, by - ux * 3.5)));
                        head.close();
                        if let Ok(path) = head.build() {
                            window.paint_path(path, edge_color(kind));
                        }
                    }
                }
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();

        let mut net = div()
            .relative()
            .w(px(GW))
            .h(px(GH))
            .rounded(px(6.))
            .border_1()
            .border_color(rule())
            .bg(paper_card())
            .child(lines);
        for (s, &node) in g.nodes.iter().enumerate() {
            let nv = node_value(&self.store, node);
            let r = radius[s];
            let (x, y) = pts[s];
            let text: SharedString = nv["label"].as_str().unwrap_or_default().to_string().into();
            let hovered = self.graph_hover == Some(node);
            net = net.child(
                v_flex()
                    .id(("gnode", node))
                    .absolute()
                    .left(px(x - 60.))
                    .top(px(y - r))
                    .w(px(120.))
                    .items_center()
                    .cursor_pointer()
                    .on_hover(cx.listener(move |this, on: &bool, _, cx| {
                        this.graph_hover = if *on {
                            Some(node)
                        } else {
                            this.graph_hover.filter(|&h| h != node)
                        };
                        cx.notify();
                    }))
                    .on_click(cx.listener(move |this, _, window, cx| this.select(node, window, cx)))
                    .child(img(self.glyph(nv, r * 2., false)).size(px(r * 2.)))
                    .when(s == 0 || hovered, |d| {
                        d.child(
                            div()
                                .px_1()
                                .rounded(px(3.))
                                .bg(paper_card())
                                .text_size(px(11.))
                                .line_height(px(13.))
                                .text_center()
                                .text_color(ink())
                                .child(text),
                        )
                    }),
            );
        }

        let key_row = h_flex().gap_3().flex_wrap().children(
            [
                ("leads to", lead()),
                ("counters", counter()),
                ("twin", twin()),
                ("shared source", rule()),
            ]
            .map(|(t, c)| {
                h_flex()
                    .gap_1()
                    .items_center()
                    .child(div().w(px(14.)).h(px(2.)).bg(c))
                    .child(div().text_xs().text_color(ink_soft()).child(t))
            }),
        );

        v_flex()
            .w(px(380.))
            .flex_none()
            .h_full()
            .px_5()
            .gap_3()
            .border_l_1()
            .border_color(rule())
            .child(preview)
            .child(div().h(px(1.)).bg(rule()).my_2())
            .child(net)
            .child(key_row)
            .when(g.nodes.len() == 1, |d| {
                d.child(
                    div()
                        .text_xs()
                        .text_color(ink_soft())
                        .child("No connections or shared sources"),
                )
            })
            .into_any_element()
    }
}
