use crate::store::Store;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    Leads,
    Counters,
    Twin,
    Shared(usize),
}

pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub kind: EdgeKind,
}

pub struct Graph {
    pub nodes: Vec<usize>,
    pub pos: Vec<(f32, f32)>,
    pub edges: Vec<Edge>,
}

const MAX_SHARED: usize = 10;

fn kind_of(k: &str) -> EdgeKind {
    match k {
        "counters" => EdgeKind::Counters,
        "twin" => EdgeKind::Twin,
        _ => EdgeKind::Leads,
    }
}

pub fn build(store: &Store, shared: &BTreeMap<(usize, usize), usize>, center: usize) -> Graph {
    let id_of = |i: usize| {
        store.nodes()[i]["id"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    };
    let index: BTreeMap<String, usize> = (0..store.nodes().len()).map(|i| (id_of(i), i)).collect();
    let links: Vec<(usize, usize, EdgeKind)> = store
        .links()
        .iter()
        .filter_map(|l| {
            let a = *index.get(l["source"].as_str()?)?;
            let b = *index.get(l["target"].as_str()?)?;
            Some((a, b, kind_of(l["kind"].as_str().unwrap_or_default())))
        })
        .collect();

    let mut members: BTreeSet<usize> = BTreeSet::from([center]);
    for &(a, b, _) in &links {
        if a == center {
            members.insert(b);
        }
        if b == center {
            members.insert(a);
        }
    }
    let mut by_refs: Vec<(usize, usize)> = shared
        .iter()
        .filter_map(|(&(a, b), &n)| match (a == center, b == center) {
            (true, _) => Some((b, n)),
            (_, true) => Some((a, n)),
            _ => None,
        })
        .collect();
    by_refs.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
    members.extend(by_refs.iter().take(MAX_SHARED).map(|x| x.0));

    let mut nodes: Vec<usize> = vec![center];
    nodes.extend(members.iter().copied().filter(|&i| i != center));
    let slot: BTreeMap<usize, usize> = nodes.iter().enumerate().map(|(s, &i)| (i, s)).collect();

    let mut edges: Vec<Edge> = links
        .iter()
        .filter_map(|&(a, b, kind)| {
            Some(Edge {
                from: *slot.get(&a)?,
                to: *slot.get(&b)?,
                kind,
            })
        })
        .collect();
    for (&(a, b), &n) in shared {
        if let (Some(&x), Some(&y)) = (slot.get(&a), slot.get(&b)) {
            edges.push(Edge {
                from: x,
                to: y,
                kind: EdgeKind::Shared(n),
            });
        }
    }

    let pos = layout(nodes.len(), &edges);
    Graph { nodes, pos, edges }
}

fn layout(n: usize, edges: &[Edge]) -> Vec<(f32, f32)> {
    let mut p: Vec<(f32, f32)> = (0..n)
        .map(|i| {
            if i == 0 {
                return (0.0, 0.0);
            }
            let a = i as f32 * 2.399_963;
            let r = 0.35 + 0.1 * ((i * 7 % 5) as f32 / 5.0);
            (a.cos() * r, a.sin() * r)
        })
        .collect();
    if n < 2 {
        return vec![(0.5, 0.5); n];
    }
    let k = (1.0 / n as f32).sqrt() * 0.9;
    let mut t = 0.12;
    for _ in 0..400 {
        let mut d = vec![(0.0f32, 0.0f32); n];
        for i in 0..n {
            for j in i + 1..n {
                let (dx, dy) = (p[i].0 - p[j].0, p[i].1 - p[j].1);
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let f = k * k / dist;
                let (fx, fy) = (dx / dist * f, dy / dist * f);
                d[i].0 += fx;
                d[i].1 += fy;
                d[j].0 -= fx;
                d[j].1 -= fy;
            }
        }
        for e in edges {
            let (i, j) = (e.from, e.to);
            if i == j {
                continue;
            }
            let (dx, dy) = (p[i].0 - p[j].0, p[i].1 - p[j].1);
            let dist = (dx * dx + dy * dy).sqrt().max(0.01);
            let w = if matches!(e.kind, EdgeKind::Shared(_)) {
                0.5
            } else {
                1.0
            };
            let f = dist * dist / k * w;
            let (fx, fy) = (dx / dist * f, dy / dist * f);
            d[i].0 -= fx;
            d[i].1 -= fy;
            d[j].0 += fx;
            d[j].1 += fy;
        }
        for i in 1..n {
            d[i].0 -= p[i].0 * 0.08;
            d[i].1 -= p[i].1 * 0.08;
            let len = (d[i].0 * d[i].0 + d[i].1 * d[i].1).sqrt().max(1e-6);
            let step = len.min(t);
            p[i].0 += d[i].0 / len * step;
            p[i].1 += d[i].1 / len * step;
        }
        t *= 0.985;
    }
    let ext = p
        .iter()
        .fold(0.0f32, |m, q| m.max(q.0.abs()).max(q.1.abs()))
        .max(0.05);
    p.iter()
        .map(|q| (0.5 + q.0 / ext * 0.5, 0.5 + q.1 / ext * 0.5))
        .collect()
}
