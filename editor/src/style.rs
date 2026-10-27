use gpui::{App, Hsla, Rgba, rgb};
use gpui_component::{Theme, ThemeMode};

pub fn paper() -> Rgba {
    rgb(0xf4ecd8)
}
pub fn paper_deep() -> Rgba {
    rgb(0xe9dcbc)
}
pub fn paper_card() -> Rgba {
    rgb(0xf8f2e3)
}
pub fn ink() -> Rgba {
    rgb(0x2a2118)
}
pub fn ink_soft() -> Rgba {
    rgb(0x6e5c45)
}
pub fn sepia() -> Rgba {
    rgb(0x7a4f2a)
}
pub fn rule() -> Rgba {
    rgb(0xbba57e)
}
pub fn boon() -> Rgba {
    rgb(0x2c5a8a)
}
pub fn bane() -> Rgba {
    rgb(0x963828)
}
pub fn lead() -> Rgba {
    rgb(0x2f6b5a)
}
pub fn counter() -> Rgba {
    rgb(0xa8641e)
}
pub fn twin() -> Rgba {
    rgb(0x64478a)
}

pub fn tone(v: f64) -> Rgba {
    if v >= 0.5 {
        boon()
    } else if v <= -0.5 {
        bane()
    } else {
        ink_soft()
    }
}

pub fn theme_ink(theme: &str) -> Rgba {
    rgb(match theme {
        "ip" => 0xc28e45,
        "power" => 0xb06a83,
        "mind" => 0x8f74b0,
        "security" => 0x6782a8,
        "information" => 0x4e9c9c,
        "world" => 0x74a05a,
        "work" => 0xa9a23f,
        _ => 0x999999,
    })
}

fn h(c: Rgba) -> Hsla {
    c.into()
}

pub fn apply_theme(cx: &mut App) {
    Theme::change(ThemeMode::Light, None, cx);
    let t = Theme::global_mut(cx);
    t.font_family = "Noto Serif".into();
    t.font_size = gpui::px(15.);
    t.radius = gpui::px(4.);
    t.radius_lg = gpui::px(6.);
    t.shadow = false;
    let c = &mut t.colors;
    c.background = h(paper());
    c.foreground = h(ink());
    c.border = h(rule());
    c.input = h(rule());
    c.ring = h(sepia());
    c.caret = h(ink());
    c.selection = h(paper_deep());
    c.muted = h(paper_deep());
    c.muted_foreground = h(ink_soft());
    c.accent = h(paper_deep());
    c.accent_foreground = h(ink());
    c.popover = h(paper_card());
    c.popover_foreground = h(ink());
    c.list = h(paper_card());
    c.list_hover = h(paper_deep());
    c.list_active = h(paper_deep());
    c.list_active_border = h(rule());
    c.primary = h(sepia());
    c.primary_hover = h(ink());
    c.primary_active = h(ink());
    c.primary_foreground = h(paper());
    c.button = h(paper_card());
    c.button_hover = h(paper_deep());
    c.button_active = h(paper_deep());
    c.button_foreground = h(ink());
    c.button_primary = h(sepia());
    c.button_primary_hover = h(ink());
    c.button_primary_active = h(ink());
    c.button_primary_foreground = h(paper());
    c.secondary = h(paper_card());
    c.secondary_hover = h(paper_deep());
    c.secondary_active = h(paper_deep());
    c.secondary_foreground = h(ink());
    c.slider_bar = h(sepia());
    c.slider_thumb = h(paper_card());
    c.scrollbar = h(paper());
    c.scrollbar_thumb = h(rule());
    c.scrollbar_thumb_hover = h(sepia());
    c.link = h(boon());
    c.link_hover = h(ink());
    c.danger = h(bane());
    c.window_border = h(rule());
}
