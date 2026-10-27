mod view;

use crate::app::{Editor, node_value};
use crate::store::{DocPath, Seg, key, node_path, path_id};
use gpui::*;
use gpui_component::input::{InputEvent, InputState, TextareaState};
use gpui_component::searchable_list::{SearchableListItem, SearchableVec};
use gpui_component::select::{SelectEvent, SelectState};
use gpui_component::slider::{SliderEvent, SliderState};
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Clone)]
pub struct NodeItem {
    id: SharedString,
    title: SharedString,
}

impl SearchableListItem for NodeItem {
    type Value = SharedString;
    fn title(&self) -> SharedString {
        self.title.clone()
    }
    fn value(&self) -> &SharedString {
        &self.id
    }
}

pub(super) type NodeSelect = Entity<SelectState<SearchableVec<NodeItem>>>;

#[derive(Clone, Copy)]
pub(super) enum F {
    Line(&'static str, &'static str),
    Para(&'static str, &'static str),
    Choice(&'static str, &'static [&'static str]),
    Ref,
}

pub(super) const STATUS: &[&str] = &["in force", "tested", "proposed", "idea"];

pub(super) struct ListSpec {
    pub(super) key: &'static str,
    pub(super) title: &'static str,
    pub(super) fields: &'static [F],
}

pub(super) const LISTS: &[ListSpec] = &[
    ListSpec {
        key: "limiters",
        title: "Limiters",
        fields: &[
            F::Line("limit", "Limit"),
            F::Line("kind", "Kind"),
            F::Para("evidence", "Evidence"),
            F::Ref,
        ],
    },
    ListSpec {
        key: "enablers",
        title: "Enablers",
        fields: &[
            F::Line("action", "Action"),
            F::Line("actor", "Actor"),
            F::Choice("status", STATUS),
            F::Line("counters", "Counters limiter"),
            F::Para("evidence", "Evidence"),
            F::Ref,
        ],
    },
    ListSpec {
        key: "actions",
        title: "Actions",
        fields: &[
            F::Line("action", "Action"),
            F::Line("actor", "Actor"),
            F::Choice("status", STATUS),
            F::Choice("aim", &["probability", "impact"]),
            F::Para("evidence", "Evidence"),
            F::Ref,
        ],
    },
    ListSpec {
        key: "precedents",
        title: "Precedents",
        fields: &[
            F::Line("event", "Event"),
            F::Line("year", "Year"),
            F::Para("parallel", "Parallel"),
            F::Para("outcome", "Outcome"),
            F::Ref,
        ],
    },
];

pub(super) const REF_FIELDS: &[F] = &[
    F::Line("url", "Link"),
    F::Line("title", "Title"),
    F::Para("finding", "Finding"),
];

pub const ZONES: [(&str, &str, i64, i64); 3] = [
    ("active", "Already here", 2023, 2026),
    ("emerging", "Now / soon", 2025, 2035),
    ("expected", "Later", 2026, 2045),
];

pub fn when_to_t(stage: &str, year: i64) -> f32 {
    let z = ZONES.iter().position(|z| z.0 == stage).unwrap_or(2);
    let (_, _, lo, hi) = ZONES[z];
    let frac = ((year - lo) as f32 / (hi - lo) as f32).clamp(0.0, 1.0);
    (z as f32 + frac * 0.998) / 3.0
}

pub fn t_to_when(t: f32) -> (&'static str, i64) {
    let z = ((t * 3.0).floor() as usize).min(2);
    let (stage, _, lo, hi) = ZONES[z];
    let frac = ((t * 3.0 - z as f32) / 0.998).clamp(0.0, 1.0);
    (stage, lo + (frac * (hi - lo) as f32).round() as i64)
}

fn round2(v: f64) -> f64 {
    let r = (v * 100.0).round() / 100.0;
    if r == 0.0 { 0.0 } else { r }
}

pub(super) enum Field {
    Line(Entity<InputState>),
    Para(Entity<TextareaState>),
}

pub struct Form {
    pub(super) fields: HashMap<String, Field>,
    pub(super) valence: (Entity<SliderState>, Entity<InputState>),
    pub(super) probability: (Entity<SliderState>, Entity<InputState>),
    pub(super) when: Entity<SliderState>,
    pub(super) links: Vec<(usize, NodeSelect)>,
    pub(super) merged: NodeSelect,
    pub(super) price_add: NodeSelect,
    pub(super) id_input: Entity<InputState>,
    pub(super) notes: Entity<TextareaState>,
    _subs: Vec<Subscription>,
}

struct Builder<'a> {
    window: &'a mut Window,
    fields: HashMap<String, Field>,
    subs: Vec<Subscription>,
}

impl Builder<'_> {
    fn line(
        &mut self,
        ed: &Editor,
        path: DocPath,
        placeholder: &'static str,
        cx: &mut Context<Editor>,
    ) {
        let text = ed.store.text(&path);
        let state = cx.new(|cx| {
            InputState::new(self.window, cx)
                .placeholder(placeholder)
                .default_value(text)
        });
        let p = path.clone();
        self.subs.push(cx.subscribe_in(
            &state,
            self.window,
            move |this: &mut Editor, s, ev: &InputEvent, window, cx| {
                if matches!(ev, InputEvent::Change) {
                    let text = s.read(cx).value().to_string();
                    let nullable = matches!(p.last(), Some(Seg::Key(k)) if k == "counters");
                    let v = if nullable && text.trim().is_empty() {
                        Value::Null
                    } else {
                        Value::String(text)
                    };
                    this.store.set(&p, v);
                    this.touch(cx);
                } else if matches!(ev, InputEvent::PressEnter { .. } | InputEvent::Blur)
                    && matches!(p.last(), Some(Seg::Key(k)) if k == "url")
                {
                    this.fill_ref(p[..p.len() - 1].to_vec(), true, window, cx);
                }
            },
        ));
        self.fields.insert(path_id(&path), Field::Line(state));
    }

    fn para(
        &mut self,
        ed: &Editor,
        path: DocPath,
        placeholder: &'static str,
        cx: &mut Context<Editor>,
    ) {
        let text = ed.store.text(&path);
        let state = cx.new(|cx| {
            TextareaState::new(self.window, cx)
                .auto_grow(2, 16)
                .placeholder(placeholder)
                .default_value(text)
        });
        let p = path.clone();
        self.subs.push(cx.subscribe_in(
            &state,
            self.window,
            move |this: &mut Editor, s, ev: &InputEvent, _, cx| {
                if matches!(ev, InputEvent::Change) {
                    let text = s.read(cx).value().to_string();
                    this.store.set(&p, Value::String(text));
                    this.touch(cx);
                }
            },
        ));
        self.fields.insert(path_id(&path), Field::Para(state));
    }

    fn fields(&mut self, ed: &Editor, base: &DocPath, spec: &[F], cx: &mut Context<Editor>) {
        for f in spec {
            let at = |k: &str| {
                let mut p = base.clone();
                p.push(key(k));
                p
            };
            match *f {
                F::Line(k, ph) => self.line(ed, at(k), ph, cx),
                F::Para(k, ph) => self.para(ed, at(k), ph, cx),
                F::Choice(..) => {}
                F::Ref => match ed.store.get(&at("ref")) {
                    Some(Value::String(_)) => self.line(ed, at("ref"), "Link", cx),
                    _ => self.fields(ed, &at("ref"), REF_FIELDS, cx),
                },
            }
        }
    }

    fn slider(
        &mut self,
        min: f32,
        max: f32,
        step: f32,
        value: f32,
        cx: &mut Context<Editor>,
    ) -> Entity<SliderState> {
        cx.new(|_| {
            SliderState::new()
                .min(min)
                .max(max)
                .step(step)
                .default_value(value)
        })
    }

    fn node_select(
        &mut self,
        ed: &Editor,
        current: Option<&str>,
        except: &str,
        cx: &mut Context<Editor>,
    ) -> NodeSelect {
        let items: Vec<NodeItem> = ed
            .store
            .nodes()
            .iter()
            .filter_map(|n| {
                n["id"]
                    .as_str()
                    .map(|id| (id, n["label"].as_str().unwrap_or_default()))
            })
            .filter(|(id, _)| *id != except)
            .map(|(id, label)| NodeItem {
                id: id.to_string().into(),
                title: format!("{label}  ({id})").into(),
            })
            .collect();
        let window = &mut *self.window;
        let current: Option<SharedString> = current.map(|c| c.to_string().into());
        cx.new(|cx| {
            let mut s =
                SelectState::new(SearchableVec::new(items), None, window, cx).searchable(true);
            if let Some(c) = &current {
                s.set_selected_value(c, window, cx);
            }
            s
        })
    }
}

impl Form {
    pub fn set_text(&self, path: &[Seg], text: &str, window: &mut Window, cx: &mut App) {
        let text = text.to_string();
        match self.fields.get(&path_id(path)) {
            Some(Field::Line(s)) => s.update(cx, |s, cx| s.set_value(text, window, cx)),
            Some(Field::Para(s)) => s.update(cx, |s, cx| s.set_value(text, window, cx)),
            None => {}
        }
    }

    pub fn build(ed: &mut Editor, i: usize, window: &mut Window, cx: &mut Context<Editor>) -> Form {
        let mut b = Builder {
            window,
            fields: HashMap::new(),
            subs: Vec::new(),
        };
        let base = node_path(i);
        let n = node_value(&ed.store, i).clone();
        let id = n["id"].as_str().unwrap_or_default().to_string();
        let at = |k: &str| {
            let mut p = base.clone();
            p.push(key(k));
            p
        };

        b.line(ed, at("label"), "Label", cx);
        b.para(ed, at("summary"), "Summary, cite sources with [n]", cx);
        b.para(
            ed,
            at("control_note"),
            "How much people can change this",
            cx,
        );
        for (r, _) in n["refs"].as_array().into_iter().flatten().enumerate() {
            let mut p = at("refs");
            p.push(Seg::Idx(r));
            b.fields(ed, &p, REF_FIELDS, cx);
        }
        for spec in LISTS {
            for (r, _) in n[spec.key].as_array().into_iter().flatten().enumerate() {
                let mut p = at(spec.key);
                p.push(Seg::Idx(r));
                b.fields(ed, &p, spec.fields, cx);
            }
        }

        let id_input = cx.new(|cx| InputState::new(b.window, cx).default_value(id.clone()));
        b.subs.push(cx.subscribe_in(
            &id_input,
            b.window,
            move |this: &mut Editor, s, ev: &InputEvent, window, cx| {
                if matches!(ev, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                    let Some(i) = this.selected else { return };
                    let new_id = s.read(cx).value().trim().to_string();
                    let old = this.selected_id().unwrap_or_default();
                    if new_id == old {
                        return;
                    }
                    match this.store.rename(i, &new_id) {
                        Ok(()) => {
                            if this.touched.remove(&old) {
                                this.touched.insert(new_id);
                            }
                            this.status = "Renamed".into();
                            this.rebuild(window, cx);
                        }
                        Err(e) => {
                            this.status = e.into();
                            s.update(cx, |s, cx| s.set_value(old, window, cx));
                            cx.notify();
                        }
                    }
                }
            },
        ));

        let note = ed.store.note(&id);
        let notes = cx.new(|cx| {
            TextareaState::new(b.window, cx)
                .auto_grow(3, 20)
                .placeholder("Notes")
                .default_value(note)
        });
        b.subs.push(cx.subscribe_in(
            &notes,
            b.window,
            move |this: &mut Editor, s, ev: &InputEvent, _, cx| {
                if matches!(ev, InputEvent::Change)
                    && let Some(id) = this.selected_id()
                {
                    let text = s.read(cx).value().to_string();
                    this.store.set_note(&id, text);
                    cx.notify();
                }
            },
        ));

        let v = n["valence"].as_f64().unwrap_or(0.0);
        let valence = number(&mut b, at("valence"), -5.0, 5.0, 0.01, v, 1.0, cx);
        let p = n["probability"].as_f64().unwrap_or(0.5);
        let probability = number(&mut b, at("probability"), 0.0, 1.0, 0.01, p, 100.0, cx);

        let t = when_to_t(
            n["stage"].as_str().unwrap_or_default(),
            n["year"].as_i64().unwrap_or(2027),
        );
        let when = b.slider(0.0, 1.0, 0.001, t, cx);
        let (sp, yp) = (at("stage"), at("year"));
        b.subs.push(
            cx.subscribe(&when, move |this: &mut Editor, _, ev: &SliderEvent, cx| {
                if let SliderEvent::Change(v) = ev {
                    let (stage, year) = t_to_when(v.start());
                    this.store.set(&sp, json!(stage));
                    this.store.set(&yp, json!(year));
                    this.touch(cx);
                }
            }),
        );

        let mut links = Vec::new();
        for (li, l) in ed.store.links().to_vec().iter().enumerate() {
            let (other, side) = if l["source"] == id.as_str() {
                (l["target"].as_str().unwrap_or_default(), "target")
            } else if l["target"] == id.as_str() {
                (l["source"].as_str().unwrap_or_default(), "source")
            } else {
                continue;
            };
            let sel = b.node_select(ed, Some(other), &id, cx);
            let path = vec![key("links"), Seg::Idx(li), key(side)];
            b.subs.push(cx.subscribe_in(
                &sel,
                b.window,
                move |this: &mut Editor, _, ev: &SelectEvent<SearchableVec<NodeItem>>, _, cx| {
                    let SelectEvent::Confirm(Some(v)) = ev else {
                        return;
                    };
                    this.store.set(&path, json!(v.to_string()));
                    this.touch(cx);
                },
            ));
            b.fields(
                ed,
                &vec![key("links"), Seg::Idx(li)],
                &[F::Para("why", "Why")],
                cx,
            );
            links.push((li, sel));
        }

        let merged = b.node_select(ed, n["merged"].as_str(), &id, cx);
        let mp = at("merged");
        b.subs.push(cx.subscribe_in(
            &merged,
            b.window,
            move |this: &mut Editor, _, ev: &SelectEvent<SearchableVec<NodeItem>>, _, cx| {
                let SelectEvent::Confirm(v) = ev;
                match v {
                    Some(v) => this.store.set(&mp, json!(v.to_string())),
                    None => this.store.remove(&mp),
                }
                this.touch(cx);
            },
        ));

        let price_add = b.node_select(ed, None, &id, cx);
        let pp = at("price_of");
        b.subs.push(cx.subscribe_in(
            &price_add,
            b.window,
            move |this: &mut Editor, _, ev: &SelectEvent<SearchableVec<NodeItem>>, window, cx| {
                let SelectEvent::Confirm(Some(v)) = ev else {
                    return;
                };
                let has = this
                    .store
                    .get(&pp)
                    .and_then(Value::as_array)
                    .is_some_and(|a| a.iter().any(|x| x == v.as_ref()));
                if !has {
                    this.store.push(&pp, json!(v.to_string()));
                    this.touch(cx);
                }
                this.rebuild(window, cx);
            },
        ));

        Form {
            fields: b.fields,
            valence,
            probability,
            when,
            links,
            merged,
            price_add,
            id_input,
            notes,
            _subs: b.subs,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn number(
    b: &mut Builder,
    path: DocPath,
    min: f32,
    max: f32,
    step: f32,
    value: f64,
    shown: f64,
    cx: &mut Context<Editor>,
) -> (Entity<SliderState>, Entity<InputState>) {
    let slider = b.slider(min, max, step, value as f32, cx);
    let fmt = move |v: f64| {
        if shown == 1.0 {
            format!("{v:.2}")
        } else {
            format!("{:.0}", v * shown)
        }
    };
    let input = cx.new(|cx| InputState::new(b.window, cx).default_value(fmt(value)));
    let (p, inp) = (path.clone(), input.clone());
    b.subs.push(cx.subscribe_in(
        &slider,
        b.window,
        move |this: &mut Editor, _, ev: &SliderEvent, window, cx| {
            if let SliderEvent::Change(v) = ev {
                let v = round2(v.start() as f64);
                this.store.set(&p, json!(v));
                inp.update(cx, |s, cx| s.set_value(fmt(v), window, cx));
                this.touch(cx);
            }
        },
    ));
    let (p, sl) = (path, slider.clone());
    b.subs.push(cx.subscribe_in(
        &input,
        b.window,
        move |this: &mut Editor, s, ev: &InputEvent, window, cx| {
            if !matches!(ev, InputEvent::Change) {
                return;
            }
            let Ok(v) = s.read(cx).value().trim().parse::<f64>() else {
                return;
            };
            let v = round2((v / shown).clamp(min as f64, max as f64));
            this.store.set(&p, json!(v));
            sl.update(cx, |s, cx| s.set_value(v as f32, window, cx));
            this.touch(cx);
        },
    ));
    (slider, input)
}
