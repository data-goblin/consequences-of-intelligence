mod app;
mod config;
mod form;
mod glyph;
mod graph;
mod side;
mod store;
mod style;

use gpui::*;
use gpui_component::Root;
use std::path::PathBuf;

fn main() {
    let cfg_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(config::default_path);
    let cfg = match config::load(&cfg_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let store = match store::Store::load(&cfg.data, &cfg.notes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    gpui_platform::application().run(move |cx: &mut App| {
        gpui_component::init(cx);
        style::apply_theme(cx);
        cx.bind_keys([KeyBinding::new("ctrl-s", app::Save, None)]);
        let bounds = Bounds::centered(None, size(px(1480.), px(940.)), cx);
        let opts = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Consequences of Intelligence".into()),
                ..Default::default()
            }),
            app_id: Some("coi-editor".into()),
            ..Default::default()
        };
        cx.open_window(opts, |window, cx| {
            let editor = cx.new(|cx| app::Editor::new(cfg, store, window, cx));
            cx.new(|cx| Root::new(editor, window, cx))
        })
        .expect("open window");
        cx.activate(true);
    });
}
