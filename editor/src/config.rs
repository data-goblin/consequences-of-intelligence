use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    data: Option<String>,
    notes: Option<String>,
    score: Option<RawScore>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScore {
    label: String,
    command: Vec<String>,
    cwd: Option<String>,
}

pub struct Score {
    pub label: String,
    pub command: Vec<String>,
    pub cwd: Option<PathBuf>,
}

pub struct Config {
    pub data: PathBuf,
    pub notes: PathBuf,
    pub score: Option<Score>,
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
}

fn expand(p: &str) -> PathBuf {
    match p.strip_prefix("~/") {
        Some(rest) => home().join(rest),
        None => PathBuf::from(p),
    }
}

fn xdg(var: &str, fallback: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(fallback))
}

pub fn default_path() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config").join("coi-editor/coi-editor.config.yaml")
}

pub fn load(path: &Path) -> Result<Config, String> {
    let raw: RawConfig = match std::fs::read_to_string(path) {
        Ok(text) => {
            serde_yaml_ng::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => RawConfig::default(),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    Ok(Config {
        data: raw
            .data
            .map(|p| expand(&p))
            .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../data.json")),
        notes: raw
            .notes
            .map(|p| expand(&p))
            .unwrap_or_else(|| xdg("XDG_DATA_HOME", ".local/share").join("coi-editor/notes.json")),
        score: raw.score.map(|s| Score {
            label: s.label,
            command: s.command,
            cwd: s.cwd.map(|p| expand(&p)),
        }),
    })
}
