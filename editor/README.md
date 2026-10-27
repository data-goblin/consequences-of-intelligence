# coi-editor

Desktop editor for the consequences in `data.json`.

```mermaid
flowchart LR
  E[coi-editor] -->|save| D[data.json]
  E -->|private notes| N[notes.json]
  E -->|score button| S[score command]
  S -->|writes valence| D
  D -->|build| P[data-goblins.com/consequences]
```

## Run

```sh
cargo run --release
cargo run --release -- path/to/coi-editor.config.yaml
```

The editor reads `~/.config/coi-editor/coi-editor.config.yaml` when you give no path. Without a config file, it edits `../data.json` and keeps notes in `~/.local/share/coi-editor/notes.json`.

## Config

```yaml
data: ~/git/consequences-of-intelligence/data.json
notes: ~/.local/share/coi-editor/notes.json
score:
  label: Score
  command: [python3, score.py, "{ids}"]
  cwd: ~/path/to/scorer
```

- `{ids}` expands to the ids you changed since the last score, or to the selected id
- The score command gets the data path in `COI_DATA`
- The editor saves before it runs the command and reloads the file after

## Use

- Select a consequence in the list, or add one with New
- Drag the sliders to set valence, probability and when; the left end of When is "already here"
- Edit the id and press Enter to rename it; links, `price_of` and `merged` follow the new id
- Save with Ctrl+S
- When `data.json` changes on disk, the editor does not overwrite it; use Reload
