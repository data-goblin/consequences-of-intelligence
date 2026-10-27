use crate::config::Config;
use crate::form::Form;
use crate::glyph;
use crate::store::{SaveError, Store, key, node_path};
use crate::style::*;
use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{Disableable as _, h_flex, v_flex};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

actions!(coi_editor, [Save]);

pub struct Editor {
    pub store: Store,
    pub cfg: Config,
    pub selected: Option<usize>,
    pub form: Option<Form>,
    pub open: BTreeSet<&'static str>,
    pub touched: BTreeSet<String>,
    pub confirm_delete: bool,
    pub status: SharedString,
    pub graph_hover: Option<usize>,
    scoring: bool,
    filter: Entity<InputState>,
    list_scroll: UniformListScrollHandle,
    focus: FocusHandle,
    glyphs: RefCell<HashMap<String, Arc<Image>>>,
    pub graph_cache: RefCell<Option<(u64, std::rc::Rc<crate::graph::Graph>)>>,
    _subs: Vec<Subscription>,
}

impl Editor {
    pub fn new(cfg: Config, store: Store, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let filter = cx.new(|cx| InputState::new(window, cx).placeholder("Filter"));
        let subs = vec![
            cx.subscribe_in(&filter, window, |_, _, ev: &InputEvent, _, cx| {
                if matches!(ev, InputEvent::Change) {
                    cx.notify();
                }
            }),
        ];
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let mut this = Self {
            store,
            cfg,
            selected: None,
            form: None,
            open: BTreeSet::from(["refs", "links"]),
            touched: BTreeSet::new(),
            confirm_delete: false,
            status: SharedString::default(),
            graph_hover: None,
            scoring: false,
            filter,
            list_scroll: UniformListScrollHandle::new(),
            focus,
            glyphs: RefCell::new(HashMap::new()),
            graph_cache: RefCell::new(None),
            _subs: subs,
        };
        if !this.store.nodes().is_empty() {
            this.select(0, window, cx);
        }
        this
    }

    pub fn select(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.selected = Some(index);
        self.confirm_delete = false;
        self.graph_hover = None;
        self.rebuild(window, cx);
    }

    pub fn rebuild(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.form = self
            .selected
            .filter(|&i| i < self.store.nodes().len())
            .map(|i| Form::build(self, i, window, cx));
        window.focus(&self.focus, cx);
        cx.notify();
    }

    pub fn selected_id(&self) -> Option<String> {
        self.selected
            .and_then(|i| self.store.nodes().get(i))
            .and_then(|n| n["id"].as_str())
            .map(str::to_string)
    }

    pub fn touch(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_id() {
            self.touched.insert(id);
        }
        cx.notify();
    }

    pub fn glyph(&self, node: &Value, size: f32, tint: bool) -> Arc<Image> {
        let id = node["id"].as_str().unwrap_or_default();
        let valence = node["valence"].as_f64().unwrap_or(0.0);
        let theme = node["theme"].as_str().unwrap_or_default();
        let evidence = node["evidence"].as_str().unwrap_or("measured");
        let radius = glyph::chart_radius(
            node["probability"].as_f64().unwrap_or(0.5),
            node["stage"].as_str().unwrap_or_default(),
        );
        let kind = node.get("kind").is_some();
        let valence = if kind { 0.0 } else { valence };
        let k = format!("{id}|{valence}|{theme}|{evidence}|{radius}|{size}|{tint}");
        self.glyphs
            .borrow_mut()
            .entry(k)
            .or_insert_with(|| {
                let g = glyph::Glyph {
                    valence,
                    theme,
                    evidence,
                    seed: glyph::seed_of(id),
                    radius,
                    tint,
                };
                Arc::new(Image::from_bytes(
                    ImageFormat::Svg,
                    glyph::svg(&g, size as f64).into_bytes(),
                ))
            })
            .clone()
    }

    fn save(&mut self, cx: &mut Context<Self>) -> bool {
        match self.store.save() {
            Ok(()) => {
                self.status = "Saved".into();
                cx.notify();
                true
            }
            Err(SaveError::ChangedOnDisk) => {
                self.status = "data.json changed on disk; reload first".into();
                cx.notify();
                false
            }
            Err(SaveError::Io(e)) => {
                self.status = format!("Save failed: {e}").into();
                cx.notify();
                false
            }
        }
    }

    fn reload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let keep = self.selected_id();
        match Store::load(&self.cfg.data, &self.cfg.notes) {
            Ok(store) => {
                self.store = store;
                self.selected = keep.and_then(|id| self.store.index_of(&id)).or(Some(0));
                self.status = "Reloaded".into();
                self.rebuild(window, cx);
            }
            Err(e) => {
                self.status = e.into();
                cx.notify();
            }
        }
    }

    fn score_ids(&self) -> Vec<String> {
        if self.touched.is_empty() {
            self.selected_id().into_iter().collect()
        } else {
            self.touched.iter().cloned().collect()
        }
    }

    fn score(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.cfg.score.is_none() || self.scoring || !self.save(cx) {
            return;
        }
        let Some(score) = &self.cfg.score else { return };
        let ids = self.score_ids();
        let mut args: Vec<String> = Vec::new();
        for a in &score.command {
            if a == "{ids}" {
                args.extend(ids.iter().cloned());
            } else {
                args.push(a.clone());
            }
        }
        let Some((program, rest)) = args.split_first() else {
            return;
        };
        let mut cmd = std::process::Command::new(program);
        cmd.args(rest).env("COI_DATA", self.store.path());
        if let Some(dir) = &score.cwd {
            cmd.current_dir(dir);
        }
        self.scoring = true;
        self.status = format!("{}: {} node(s)", score.label, ids.len()).into();
        cx.notify();
        let run = cx.background_executor().spawn(async move { cmd.output() });
        cx.spawn_in(window, async move |this, cx| {
            let out = run.await;
            this.update_in(cx, |this, window, cx| {
                this.scoring = false;
                match out {
                    Ok(o) if o.status.success() => {
                        for id in &ids {
                            this.touched.remove(id);
                        }
                        this.reload(window, cx);
                        this.status = format!("Scored {}", ids.join(", ")).into();
                    }
                    Ok(o) => {
                        let err = String::from_utf8_lossy(&o.stderr);
                        this.status = format!(
                            "Scoring failed: {}",
                            err.lines().last().unwrap_or("no output")
                        )
                        .into();
                    }
                    Err(e) => this.status = format!("Scoring failed: {e}").into(),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn add_node(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let i = self.store.add_node();
        self.filter.update(cx, |f, cx| f.set_value("", window, cx));
        self.list_scroll.scroll_to_item(i, ScrollStrategy::Center);
        self.select(i, window, cx);
        self.touch(cx);
    }

    pub fn delete_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(i) = self.selected else { return };
        if !self.confirm_delete {
            self.confirm_delete = true;
            cx.notify();
            return;
        }
        if let Some(id) = self.selected_id() {
            self.touched.remove(&id);
        }
        self.store.delete_node(i);
        let n = self.store.nodes().len();
        self.selected = (n > 0).then(|| i.min(n - 1));
        self.confirm_delete = false;
        self.rebuild(window, cx);
    }

    fn visible(&self, cx: &App) -> Vec<usize> {
        let q = self.filter.read(cx).value().to_lowercase();
        self.store
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, n)| {
                q.is_empty()
                    || n["label"]
                        .as_str()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&q)
                    || n["id"].as_str().unwrap_or_default().contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn render_row(&self, i: usize, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let n = &self.store.nodes()[i];
        let v = n["valence"].as_f64().unwrap_or(0.0);
        let label = n["label"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or("Untitled")
            .to_string();
        let meta = format!(
            "{}  |  {}  |  {}",
            n["theme"].as_str().unwrap_or_default(),
            n["year"].as_i64().unwrap_or_default(),
            n["stage"].as_str().unwrap_or_default()
        );
        let selected = self.selected == Some(i);
        let touched = n["id"].as_str().is_some_and(|id| self.touched.contains(id));
        h_flex()
            .id(("row", i))
            .w_full()
            .h(px(44.))
            .px_2()
            .gap_2()
            .rounded(px(4.))
            .when(selected, |d| d.bg(paper_deep()))
            .hover(|d| d.bg(paper_deep()))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, window, cx| this.select(i, window, cx)))
            .child(img(self.glyph(n, 32., false)).size(px(32.)).flex_none())
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(div().text_sm().text_color(ink()).truncate().child(label))
                    .child(
                        div()
                            .text_xs()
                            .text_color(ink_soft())
                            .truncate()
                            .child(meta),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .flex_none()
                    .text_color(tone(v))
                    .child(format!("{}{v:.1}", if v > 0.0 { "+" } else { "" })),
            )
            .when(touched, |d| {
                d.child(div().size(px(6.)).rounded_full().bg(sepia()).flex_none())
            })
    }

    fn render_list(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let rows = self.visible(cx);
        let count = rows.len();
        v_flex()
            .w(px(300.))
            .flex_none()
            .h_full()
            .border_r_1()
            .border_color(rule())
            .child(
                h_flex()
                    .p_2()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .child(Input::new(&self.filter).cleanable(true)),
                    )
                    .child(
                        Button::new("add")
                            .label("New")
                            .on_click(cx.listener(|this, _, window, cx| this.add_node(window, cx))),
                    ),
            )
            .child(
                div()
                    .px_3()
                    .pb_1()
                    .text_xs()
                    .text_color(ink_soft())
                    .child(format!("{count} consequences")),
            )
            .child(
                uniform_list(
                    "nodes",
                    count,
                    cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                        range.map(|r| this.render_row(rows[r], cx)).collect()
                    }),
                )
                .track_scroll(&self.list_scroll)
                .flex_1()
                .px_1(),
            )
            .into_any_element()
    }

    fn render_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        let changed = self.store.changed_on_disk();
        let score = self.cfg.score.as_ref().map(|s| s.label.clone());
        let pending = self.score_ids().len();
        h_flex()
            .h(px(52.))
            .px_4()
            .gap_3()
            .border_b_1()
            .border_color(rule())
            .child(
                div()
                    .text_lg()
                    .text_color(ink())
                    .child("Consequences of Intelligence"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(ink_soft())
                    .child(self.store.path().display().to_string()),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_sm()
                    .text_color(sepia())
                    .child(self.status.clone()),
            )
            .when(changed, |d| {
                d.child(
                    Button::new("reload")
                        .label("Reload")
                        .outline()
                        .on_click(cx.listener(|this, _, window, cx| this.reload(window, cx))),
                )
            })
            .child(
                Button::new("save")
                    .label(if self.store.dirty { "Save" } else { "Saved" })
                    .disabled(!self.store.dirty)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.save(cx);
                    })),
            )
            .when_some(score, |d, label| {
                d.child(
                    Button::new("score")
                        .primary()
                        .label(format!("{label} ({pending})"))
                        .loading(self.scoring)
                        .disabled(pending == 0)
                        .on_click(cx.listener(|this, _, window, cx| this.score(window, cx))),
                )
            })
            .into_any_element()
    }
}

impl Render for Editor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let list = self.render_list(cx);
        let bar = self.render_bar(cx);
        let form = self.render_form(window, cx);
        let side = self.render_side(cx);
        v_flex()
            .size_full()
            .bg(paper())
            .text_color(ink())
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &Save, _, cx| {
                this.save(cx);
            }))
            .child(bar)
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .items_start()
                    .child(list)
                    .child(form)
                    .child(side),
            )
    }
}

pub fn node_value(store: &Store, i: usize) -> &Value {
    store.get(&node_path(i)).unwrap_or(&Value::Null)
}

pub fn list_path(i: usize, list: &str) -> Vec<crate::store::Seg> {
    let mut p = node_path(i);
    p.push(key(list));
    p
}
