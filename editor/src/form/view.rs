use super::*;
use crate::app::list_path;
use crate::style::*;
use gpui::prelude::FluentBuilder as _;
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::input::{Input, Textarea};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::select::Select;
use gpui_component::slider::Slider;
use gpui_component::{h_flex, v_flex};

fn label(text: impl Into<SharedString>) -> Div {
    div().text_xs().text_color(ink_soft()).child(text.into())
}

fn chip(
    id: impl Into<ElementId>,
    text: impl Into<SharedString>,
    on: bool,
    color: Rgba,
) -> Stateful<Div> {
    div()
        .id(id.into())
        .px_2()
        .py(px(2.))
        .rounded(px(4.))
        .border_1()
        .text_sm()
        .cursor_pointer()
        .border_color(if on { color } else { rgba(0x00000000) })
        .text_color(if on { color } else { ink_soft() })
        .when(on, |d| d.bg(paper_card()))
        .hover(|d| d.text_color(ink()))
        .child(text.into())
}

impl Editor {
    fn field(&self, path: &[Seg]) -> AnyElement {
        match self
            .form
            .as_ref()
            .and_then(|f| f.fields.get(&path_id(path)))
        {
            Some(Field::Line(s)) => Input::new(s).into_any_element(),
            Some(Field::Para(s)) => Textarea::new(s).into_any_element(),
            None => div().into_any_element(),
        }
    }

    fn choice(
        &self,
        path: DocPath,
        options: &[(&str, &str)],
        none: Option<&'static str>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let current = self
            .store
            .get(&path)
            .and_then(Value::as_str)
            .map(str::to_string);
        let pid = path_id(&path);
        let mut row = h_flex().flex_wrap().gap_1();
        if let Some(none) = none {
            let p = path.clone();
            row = row.child(
                chip(
                    SharedString::from(format!("{pid}:none")),
                    none,
                    current.is_none(),
                    sepia(),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.store.remove(&p);
                    this.touch(cx);
                })),
            );
        }
        for &(value, text) in options {
            let p = path.clone();
            let v = value.to_string();
            let color = if pid.ends_with("theme") {
                theme_ink(value)
            } else {
                sepia()
            };
            row = row.child(
                chip(
                    SharedString::from(format!("{pid}:{value}")),
                    text.to_string(),
                    current.as_deref() == Some(value),
                    color,
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.store.set(&p, json!(v));
                    this.touch(cx);
                })),
            );
        }
        row
    }

    fn section(
        &self,
        name: &'static str,
        title: impl Into<SharedString>,
        count: usize,
        cx: &mut Context<Self>,
    ) -> (bool, Stateful<Div>) {
        let open = self.open.contains(name);
        let head = h_flex()
            .id(SharedString::from(format!("sec-{name}")))
            .w_full()
            .pt_4()
            .pb_1()
            .gap_2()
            .border_b_1()
            .border_color(rule())
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                if !this.open.remove(name) {
                    this.open.insert(name);
                }
                cx.notify();
            }))
            .child(
                div()
                    .text_color(sepia())
                    .child(if open { "▾" } else { "▸" }),
            )
            .child(div().text_color(ink()).child(title.into()))
            .child(
                div()
                    .text_xs()
                    .text_color(ink_soft())
                    .child(count.to_string()),
            );
        (open, head)
    }

    fn add_button(
        &self,
        id: impl Into<SharedString>,
        list: DocPath,
        item: Value,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        Button::new(ElementId::Name(id.into()))
            .label("Add")
            .outline()
            .compact()
            .on_click(cx.listener(move |this, _, window, cx| {
                this.store.push(&list, item.clone());
                this.touch(cx);
                this.rebuild(window, cx);
            }))
    }

    fn remove_button(
        &self,
        id: impl Into<SharedString>,
        list: DocPath,
        index: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(ElementId::Name(id.into()))
            .px_1()
            .text_color(ink_soft())
            .cursor_pointer()
            .hover(|d| d.text_color(bane()))
            .child("✕")
            .on_click(cx.listener(move |this, _, window, cx| {
                this.store.remove_item(&list, index);
                this.touch(cx);
                this.rebuild(window, cx);
            }))
    }

    fn fill_button(&self, base: DocPath, cx: &mut Context<Self>) -> AnyElement {
        let Some(fill) = &self.cfg.fill else {
            return div().into_any_element();
        };
        let mut url = base.clone();
        url.push(key("url"));
        let tag = path_id(&url);
        Button::new(ElementId::Name(format!("fill-{tag}").into()))
            .label(fill.label.clone())
            .outline()
            .compact()
            .loading(self.filling.contains(&tag))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.fill_ref(base.clone(), false, window, cx)
            }))
            .into_any_element()
    }

    fn ref_block(&self, base: &DocPath, cx: &mut Context<Self>) -> AnyElement {
        let mut rp = base.clone();
        rp.push(key("ref"));
        let at = |k: &str| {
            let mut p = rp.clone();
            p.push(key(k));
            p
        };
        let url = match self.store.get(&rp) {
            Some(Value::String(s)) => s.clone(),
            _ => self.store.text(&at("url")),
        };
        let body = match self.store.get(&rp) {
            Some(Value::String(_)) => v_flex().gap_1().child(self.field(&rp)),
            _ => v_flex()
                .gap_1()
                .child(
                    h_flex()
                        .gap_2()
                        .child(div().flex_1().child(self.field(&at("url"))))
                        .child(self.fill_button(rp.clone(), cx)),
                )
                .child(self.field(&at("title")))
                .child(self.field(&at("finding"))),
        };
        v_flex()
            .gap_1()
            .pl_3()
            .border_l_2()
            .border_color(paper_deep())
            .child(h_flex().gap_2().child(label("Source")).child(open_link(
                format!("open-{}", path_id(&rp)),
                url,
                cx,
            )))
            .child(body)
            .into_any_element()
    }

    fn render_refs(&self, i: usize, cx: &mut Context<Self>) -> AnyElement {
        let list = list_path(i, "refs");
        let refs = self
            .store
            .get(&list)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let (open, head) = self.section("refs", "References", refs.len(), cx);
        let mut col = v_flex().gap_3().child(head);
        if open {
            for (r, rf) in refs.iter().enumerate() {
                let mut p = list.clone();
                p.push(Seg::Idx(r));
                let at = |k: &str| {
                    let mut q = p.clone();
                    q.push(key(k));
                    q
                };
                let url = rf["url"].as_str().unwrap_or_default().to_string();
                col = col.child(
                    h_flex()
                        .items_start()
                        .gap_2()
                        .child(
                            div()
                                .w(px(28.))
                                .pt_1()
                                .text_color(sepia())
                                .child(format!("[{}]", r + 1)),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .gap_1()
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .child(div().flex_1().child(self.field(&at("url"))))
                                        .child(open_link(format!("open-ref-{r}"), url, cx))
                                        .child(self.fill_button(p.clone(), cx)),
                                )
                                .child(self.field(&at("title")))
                                .child(self.field(&at("finding"))),
                        )
                        .child(self.remove_button(format!("rm-ref-{r}"), list.clone(), r, cx)),
                );
            }
            col = col.child(h_flex().child(self.add_button(
                "add-ref",
                list,
                json!({"title": "", "url": "", "finding": ""}),
                cx,
            )));
        }
        col.into_any_element()
    }

    fn render_list_spec(&self, i: usize, spec: &ListSpec, cx: &mut Context<Self>) -> AnyElement {
        let list = list_path(i, spec.key);
        let items = self
            .store
            .get(&list)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let (open, head) = self.section(spec.key, spec.title, items.len(), cx);
        let mut col = v_flex().gap_3().child(head);
        if open {
            for r in 0..items.len() {
                let mut p = list.clone();
                p.push(Seg::Idx(r));
                let mut body = v_flex().flex_1().gap_1();
                for f in spec.fields {
                    let at = |k: &str| {
                        let mut q = p.clone();
                        q.push(key(k));
                        q
                    };
                    body = match *f {
                        F::Line(k, l) | F::Para(k, l) => {
                            body.child(label(l)).child(self.field(&at(k)))
                        }
                        F::Choice(k, opts) => {
                            let o: Vec<(&str, &str)> = opts.iter().map(|o| (*o, *o)).collect();
                            body.child(self.choice(at(k), &o, None, cx))
                        }
                        F::Ref => body.child(self.ref_block(&p, cx)),
                    };
                }
                col = col.child(
                    h_flex()
                        .items_start()
                        .gap_2()
                        .p_2()
                        .rounded(px(6.))
                        .bg(paper_card())
                        .child(body)
                        .child(self.remove_button(
                            format!("rm-{}-{r}", spec.key),
                            list.clone(),
                            r,
                            cx,
                        )),
                );
            }
            let mut item = serde_json::Map::new();
            for f in spec.fields {
                match *f {
                    F::Line(k, _) | F::Para(k, _) => {
                        item.insert(
                            k.into(),
                            if k == "counters" {
                                Value::Null
                            } else {
                                json!("")
                            },
                        );
                    }
                    F::Choice(k, opts) => {
                        item.insert(k.into(), json!(opts[0]));
                    }
                    F::Ref => {
                        item.insert("ref".into(), json!({"title": "", "url": "", "finding": ""}));
                    }
                }
            }
            col = col.child(h_flex().child(self.add_button(
                format!("add-{}", spec.key),
                list,
                Value::Object(item),
                cx,
            )));
        }
        col.into_any_element()
    }

    fn render_links(&self, i: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(form) = &self.form else {
            return div().into_any_element();
        };
        let id = self.selected_id().unwrap_or_default();
        let (open, head) = self.section("links", "Connections", form.links.len(), cx);
        let mut col = v_flex().gap_3().child(head);
        if open {
            for (li, sel) in &form.links {
                let li = *li;
                let l = &self.store.links()[li];
                let outgoing = l["source"] == id.as_str();
                let kind = vec![key("links"), Seg::Idx(li), key("kind")];
                let swap = div()
                    .id(("swap", li))
                    .px_2()
                    .rounded(px(4.))
                    .border_1()
                    .border_color(rule())
                    .text_sm()
                    .cursor_pointer()
                    .hover(|d| d.bg(paper_deep()))
                    .child(if outgoing { "this →" } else { "→ this" })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        let base = vec![key("links"), Seg::Idx(li)];
                        let mut s = base.clone();
                        s.push(key("source"));
                        let mut t = base;
                        t.push(key("target"));
                        let (a, b) = (this.store.text(&s), this.store.text(&t));
                        this.store.set(&s, json!(b));
                        this.store.set(&t, json!(a));
                        this.touch(cx);
                        this.rebuild(window, cx);
                    }));
                col = col.child(
                    h_flex()
                        .items_start()
                        .gap_2()
                        .p_2()
                        .rounded(px(6.))
                        .bg(paper_card())
                        .child(
                            v_flex()
                                .flex_1()
                                .gap_1()
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .child(swap)
                                        .child(self.choice(
                                            kind,
                                            &[
                                                ("leads", "leads to"),
                                                ("counters", "counters"),
                                                ("twin", "twin of"),
                                            ],
                                            None,
                                            cx,
                                        ))
                                        .child(
                                            div()
                                                .flex_1()
                                                .child(Select::new(sel).menu_width(px(420.))),
                                        ),
                                )
                                .child(self.field(&[key("links"), Seg::Idx(li), key("why")])),
                        )
                        .child(
                            div()
                                .id(("rm-link", li))
                                .px_1()
                                .text_color(ink_soft())
                                .cursor_pointer()
                                .hover(|d| d.text_color(bane()))
                                .child("✕")
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.store.remove(&[key("links"), Seg::Idx(li)]);
                                    this.touch(cx);
                                    this.rebuild(window, cx);
                                })),
                        ),
                );
            }
            let target = self
                .store
                .nodes()
                .iter()
                .filter_map(|n| n["id"].as_str())
                .find(|o| *o != id)
                .unwrap_or_default()
                .to_string();
            col = col.child(h_flex().child(self.add_button(
                "add-link",
                vec![key("links")],
                json!({"source": id, "target": target, "kind": "leads", "why": ""}),
                cx,
            )));
        }
        let _ = i;
        col.into_any_element()
    }

    fn render_meta(&self, i: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(form) = &self.form else {
            return div().into_any_element();
        };
        let list = list_path(i, "price_of");
        let ids: Vec<String> = self
            .store
            .get(&list)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        let (open, head) = self.section("meta", "Price of and merge", ids.len(), cx);
        let mut col = v_flex().gap_2().child(head);
        if open {
            let mut chips = h_flex().flex_wrap().gap_1();
            for (k, other) in ids.iter().enumerate() {
                let label = self
                    .store
                    .index_of(other)
                    .map(|j| {
                        self.store.nodes()[j]["label"]
                            .as_str()
                            .unwrap_or(other)
                            .to_string()
                    })
                    .unwrap_or(other.clone());
                chips = chips.child(
                    h_flex()
                        .gap_1()
                        .px_2()
                        .rounded(px(4.))
                        .bg(paper_card())
                        .text_sm()
                        .child(label)
                        .child(self.remove_button(format!("rm-price-{k}"), list.clone(), k, cx)),
                );
            }
            col = col
                .child(label("Price of: the benefits this harm is the cost of"))
                .child(chips)
                .child(
                    Select::new(&form.price_add)
                        .placeholder("Add")
                        .menu_width(px(420.)),
                )
                .child(label("Merged into"))
                .child(
                    Select::new(&form.merged)
                        .placeholder("Not merged")
                        .cleanable(true)
                        .menu_width(px(420.)),
                );
        }
        col.into_any_element()
    }

    pub fn render_form(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let (Some(i), Some(form)) = (self.selected, self.form.as_ref()) else {
            return div().flex_1().into_any_element();
        };
        let base = node_path(i);
        let at = |k: &str| {
            let mut p = base.clone();
            p.push(key(k));
            p
        };
        let n = node_value(&self.store, i).clone();
        let (stage, year) = (
            n["stage"].as_str().unwrap_or_default().to_string(),
            n["year"].as_i64().unwrap_or_default(),
        );
        let zone = ZONES
            .iter()
            .find(|z| z.0 == stage)
            .map(|z| z.1)
            .unwrap_or("");
        let themes: Vec<(String, String)> = self.store.themes();
        let theme_opts: Vec<(&str, &str)> = themes
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let v = n["valence"].as_f64().unwrap_or(0.0);
        let p = n["probability"].as_f64().unwrap_or(0.0);
        let has_control = n.get("control").is_some();
        let (val_s, val_i, prob_s, prob_i, when, id_input, notes) = (
            form.valence.0.clone(),
            form.valence.1.clone(),
            form.probability.0.clone(),
            form.probability.1.clone(),
            form.when.clone(),
            form.id_input.clone(),
            form.notes.clone(),
        );

        let slider_row = |title: &'static str,
                          left: &'static str,
                          right: &'static str,
                          s: Entity<SliderState>,
                          input: Option<Entity<InputState>>,
                          readout: String,
                          color: Rgba| {
            v_flex()
                .gap_1()
                .child(
                    h_flex()
                        .child(label(title))
                        .child(div().flex_1())
                        .child(div().text_sm().text_color(color).child(readout)),
                )
                .child(
                    h_flex()
                        .gap_3()
                        .child(
                            div()
                                .w(px(56.))
                                .text_xs()
                                .text_color(ink_soft())
                                .child(left),
                        )
                        .child(div().flex_1().child(Slider::new(&s)))
                        .child(
                            div()
                                .w(px(56.))
                                .text_xs()
                                .text_color(ink_soft())
                                .child(right),
                        )
                        .child(
                            div()
                                .w(px(72.))
                                .when_some(input, |d, inp| d.child(Input::new(&inp))),
                        ),
                )
        };

        let mut col = v_flex()
            .gap_3()
            .p_4()
            .pb(px(80.))
            .child(
                h_flex()
                    .gap_2()
                    .child(div().flex_1().text_xl().child(self.field(&at("label"))))
                    .child(div().w(px(160.)).child(Input::new(&id_input))),
            )
            .child(slider_row(
                "Doom to bloom",
                "doom",
                "bloom",
                val_s,
                Some(val_i),
                format!("{}{v:.2}", if v > 0.0 { "+" } else { "" }),
                tone(v),
            ))
            .child(slider_row(
                "Probability by 2035",
                "unlikely",
                "certain",
                prob_s,
                Some(prob_i),
                format!("{:.0}%", p * 100.0),
                ink(),
            ))
            .child(slider_row(
                "When",
                "",
                "",
                when,
                None,
                format!("{zone}, {year}"),
                ink(),
            ))
            .child(
                h_flex()
                    .gap_3()
                    .child(div().w(px(56.)))
                    .child(h_flex().flex_1().children(ZONES.iter().map(|z| {
                        div()
                            .flex_1()
                            .text_center()
                            .text_xs()
                            .text_color(if z.0 == stage { sepia() } else { rule() })
                            .child(z.1)
                    })))
                    .child(div().w(px(152.))),
            )
            .child(label("Theme"))
            .child(self.choice(at("theme"), &theme_opts, None, cx))
            .child(label("Evidence"))
            .child(self.choice(
                at("evidence"),
                &[
                    ("measured", "measured"),
                    ("reported", "reported"),
                    ("projected", "projected"),
                    ("speculative", "speculative"),
                ],
                None,
                cx,
            ))
            .child(label("Kind"))
            .child(self.choice(
                at("kind"),
                &[("capability", "capability"), ("event", "event")],
                Some("consequence"),
                cx,
            ))
            .child(label("Control"))
            .child(self.choice(
                at("control"),
                &[
                    ("inevitable", "inevitable"),
                    ("mitigable", "mitigable"),
                    ("preventable", "preventable"),
                    ("avoidable", "avoidable"),
                ],
                Some("unset"),
                cx,
            ))
            .when(has_control, |d| d.child(self.field(&at("control_note"))))
            .child(label("Summary"))
            .child(self.field(&at("summary")))
            .child(label("Notes (private)"))
            .child(Textarea::new(&notes))
            .child(self.render_refs(i, cx))
            .child(self.render_links(i, cx));
        for spec in LISTS {
            col = col.child(self.render_list_spec(i, spec, cx));
        }
        col = col.child(self.render_meta(i, cx)).child(
            h_flex().pt_6().child(
                Button::new("delete")
                    .danger()
                    .outline()
                    .label(if self.confirm_delete {
                        "Click again to delete"
                    } else {
                        "Delete consequence"
                    })
                    .on_click(cx.listener(|this, _, window, cx| this.delete_selected(window, cx))),
            ),
        );
        div()
            .flex_1()
            .min_w_0()
            .h_full()
            .child(
                div()
                    .id("form")
                    .size_full()
                    .overflow_y_scrollbar()
                    .child(col),
            )
            .into_any_element()
    }
}

fn open_link(id: String, url: String, cx: &mut Context<Editor>) -> impl IntoElement {
    let host = url
        .split("//")
        .nth(1)
        .and_then(|r| r.split('/').next())
        .map(|h| h.trim_start_matches("www.").to_string())
        .unwrap_or_default();
    div()
        .id(ElementId::Name(id.into()))
        .text_xs()
        .text_color(boon())
        .cursor_pointer()
        .hover(|d| d.text_color(ink()))
        .when(!host.is_empty(), |d| d.child(format!("{host} ↗")))
        .on_click(cx.listener(move |_, _, _, cx| {
            if url.starts_with("http") {
                cx.open_url(&url);
            }
        }))
}
