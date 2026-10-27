use serde::Serialize;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Seg {
    Key(String),
    Idx(usize),
}

pub type DocPath = Vec<Seg>;

pub fn key(k: &str) -> Seg {
    Seg::Key(k.to_string())
}

pub fn node_path(node: usize) -> DocPath {
    vec![key("nodes"), Seg::Idx(node)]
}

pub fn path_id(path: &[Seg]) -> String {
    path.iter()
        .map(|s| match s {
            Seg::Key(k) => k.clone(),
            Seg::Idx(i) => i.to_string(),
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub enum SaveError {
    ChangedOnDisk,
    Io(String),
}

pub struct Store {
    pub doc: Value,
    pub notes: Map<String, Value>,
    path: PathBuf,
    notes_path: PathBuf,
    mtime: Option<SystemTime>,
    trailing_newline: bool,
    pub dirty: bool,
}

fn mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

fn to_text(v: &Value) -> String {
    let mut out = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(
        &mut out,
        serde_json::ser::PrettyFormatter::with_indent(b" "),
    );
    v.serialize(&mut ser).expect("json values always serialize");
    String::from_utf8(out).expect("serde_json writes utf-8")
}

impl Store {
    pub fn load(path: &Path, notes_path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let doc: Value =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        if !doc["nodes"].is_array() || !doc["links"].is_array() {
            return Err(format!("{}: no nodes or links array", path.display()));
        }
        let notes = match std::fs::read_to_string(notes_path) {
            Ok(t) => serde_json::from_str::<Map<String, Value>>(&t)
                .map_err(|e| format!("{}: {e}", notes_path.display()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Map::new(),
            Err(e) => return Err(format!("{}: {e}", notes_path.display())),
        };
        Ok(Self {
            doc,
            notes,
            path: path.to_path_buf(),
            notes_path: notes_path.to_path_buf(),
            mtime: mtime(path),
            trailing_newline: text.ends_with('\n'),
            dirty: false,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn changed_on_disk(&self) -> bool {
        mtime(&self.path) != self.mtime
    }

    pub fn save(&mut self) -> Result<(), SaveError> {
        if self.changed_on_disk() {
            return Err(SaveError::ChangedOnDisk);
        }
        let mut text = to_text(&self.doc);
        if self.trailing_newline {
            text.push('\n');
        }
        std::fs::write(&self.path, text).map_err(|e| SaveError::Io(e.to_string()))?;
        if let Some(dir) = self.notes_path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| SaveError::Io(e.to_string()))?;
        }
        let notes = Value::Object(
            self.notes
                .iter()
                .filter(|(_, v)| v.as_str().is_some_and(|s| !s.trim().is_empty()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        );
        std::fs::write(&self.notes_path, to_text(&notes) + "\n")
            .map_err(|e| SaveError::Io(e.to_string()))?;
        self.mtime = mtime(&self.path);
        self.dirty = false;
        Ok(())
    }

    pub fn nodes(&self) -> &[Value] {
        self.doc["nodes"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn links(&self) -> &[Value] {
        self.doc["links"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn themes(&self) -> Vec<(String, String)> {
        self.doc["themes"]
            .as_object()
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or(k).to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.nodes().iter().position(|n| n["id"] == id)
    }

    pub fn get(&self, path: &[Seg]) -> Option<&Value> {
        let mut v = &self.doc;
        for s in path {
            v = match s {
                Seg::Key(k) => v.get(k)?,
                Seg::Idx(i) => v.get(*i)?,
            };
        }
        Some(v)
    }

    pub fn text(&self, path: &[Seg]) -> String {
        match self.get(path) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            _ => String::new(),
        }
    }

    pub fn set(&mut self, path: &[Seg], value: Value) {
        let mut v = &mut self.doc;
        for s in path {
            v = match s {
                Seg::Key(k) => {
                    if !v.is_object() {
                        *v = Value::Object(Map::new());
                    }
                    v.as_object_mut()
                        .expect("just made an object")
                        .entry(k.clone())
                        .or_insert(Value::Null)
                }
                Seg::Idx(i) => match v.get_mut(*i) {
                    Some(x) => x,
                    None => return,
                },
            };
        }
        if *v != value {
            *v = value;
            self.dirty = true;
        }
    }

    pub fn remove(&mut self, path: &[Seg]) {
        let Some((last, parent)) = path.split_last() else {
            return;
        };
        let mut v = &mut self.doc;
        for s in parent {
            v = match s {
                Seg::Key(k) => match v.get_mut(k) {
                    Some(x) => x,
                    None => return,
                },
                Seg::Idx(i) => match v.get_mut(*i) {
                    Some(x) => x,
                    None => return,
                },
            };
        }
        let removed = match (last, v) {
            (Seg::Key(k), Value::Object(m)) => m.shift_remove(k).is_some(),
            (Seg::Idx(i), Value::Array(a)) if *i < a.len() => {
                a.remove(*i);
                true
            }
            _ => false,
        };
        self.dirty |= removed;
    }

    pub fn push(&mut self, path: &[Seg], item: Value) {
        let mut list = match self.get(path) {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        list.push(item);
        self.set(path, Value::Array(list));
    }

    pub fn remove_item(&mut self, list: &[Seg], index: usize) {
        let mut p = list.to_vec();
        p.push(Seg::Idx(index));
        self.remove(&p);
        if self
            .get(list)
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
            && list.len() > 2
        {
            self.remove(list);
        }
    }

    pub fn note(&self, id: &str) -> String {
        self.notes
            .get(id)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    }

    pub fn set_note(&mut self, id: &str, text: String) {
        if self.note(id) != text {
            self.notes.insert(id.to_string(), Value::String(text));
            self.dirty = true;
        }
    }

    pub fn add_node(&mut self) -> usize {
        let mut n = 1;
        while self.index_of(&format!("new{n}")).is_some() {
            n += 1;
        }
        let theme = self
            .themes()
            .first()
            .map(|t| t.0.clone())
            .unwrap_or_default();
        let node = json!({
            "id": format!("new{n}"),
            "label": "",
            "valence": 0.0,
            "probability": 0.5,
            "year": 2027,
            "evidence": "speculative",
            "summary": "",
            "refs": [],
            "theme": theme,
            "stage": "expected",
        });
        self.push(&[key("nodes")], node);
        self.nodes().len() - 1
    }

    pub fn delete_node(&mut self, index: usize) {
        let id = self.nodes()[index]["id"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let links: Vec<Value> = self
            .links()
            .iter()
            .filter(|l| l["source"] != id && l["target"] != id)
            .cloned()
            .collect();
        self.set(&[key("links")], Value::Array(links));
        self.remove(&node_path(index));
        for i in 0..self.nodes().len() {
            let mut p = node_path(i);
            p.push(key("price_of"));
            if let Some(Value::Array(a)) = self.get(&p) {
                let kept: Vec<Value> = a.iter().filter(|v| **v != id).cloned().collect();
                if kept.len() != a.len() {
                    if kept.is_empty() {
                        self.remove(&p)
                    } else {
                        self.set(&p, Value::Array(kept))
                    }
                }
            }
            let mut m = node_path(i);
            m.push(key("merged"));
            if self.get(&m).is_some_and(|v| *v == id) {
                self.remove(&m);
            }
        }
        self.notes.shift_remove(&id);
    }

    pub fn rename(&mut self, index: usize, new_id: &str) -> Result<(), String> {
        let old = self.nodes()[index]["id"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        if old == new_id {
            return Ok(());
        }
        if new_id.is_empty()
            || !new_id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            return Err("id: lowercase letters and digits only".into());
        }
        if self.index_of(new_id).is_some() {
            return Err(format!("id {new_id} exists"));
        }
        let to = Value::String(new_id.to_string());
        let swap = |v: &mut Value| {
            if *v == old {
                *v = to.clone();
            }
        };
        if let Some(links) = self.doc["links"].as_array_mut() {
            for l in links {
                swap(&mut l["source"]);
                swap(&mut l["target"]);
            }
        }
        if let Some(nodes) = self.doc["nodes"].as_array_mut() {
            for n in nodes.iter_mut() {
                if let Some(m) = n.get_mut("merged") {
                    swap(m);
                }
                if let Some(Value::Array(a)) = n.get_mut("price_of") {
                    a.iter_mut().for_each(swap);
                }
            }
            nodes[index]["id"] = to.clone();
        }
        if let Some(note) = self.notes.shift_remove(&old) {
            self.notes.insert(new_id.to_string(), note);
        }
        self.dirty = true;
        Ok(())
    }

    pub fn shared_refs(&self) -> BTreeMap<(usize, usize), usize> {
        let mut by_url: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
        for (i, n) in self.nodes().iter().enumerate() {
            for r in n["refs"].as_array().into_iter().flatten() {
                if let Some(u) = r["url"].as_str().filter(|u| !u.is_empty()) {
                    by_url
                        .entry(u.trim_end_matches('/').to_string())
                        .or_default()
                        .insert(i);
                }
            }
        }
        let mut pairs = BTreeMap::new();
        for set in by_url.values() {
            let v: Vec<usize> = set.iter().copied().collect();
            for (a, &x) in v.iter().enumerate() {
                for &y in &v[a + 1..] {
                    *pairs.entry((x, y)).or_insert(0) += 1;
                }
            }
        }
        pairs
    }
}
