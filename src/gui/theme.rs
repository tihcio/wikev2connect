use egui::{Color32, FontId, Rounding, Stroke, Visuals};
use std::collections::HashMap;
use std::fs;

// ── KDE colour palette (read from ~/.config/kdeglobals at startup) ──────────

#[derive(Clone, Debug)]
pub struct Palette {
    pub win_bg:     Color32,
    pub view_bg:    Color32,
    pub view_alt:   Color32,
    pub win_fg:     Color32,
    pub muted_fg:   Color32,
    pub hdr_bg:     Color32,
    pub accent:     Color32,
    pub accent_text:Color32,
    pub sel_bg:     Color32,
    pub btn_bg:     Color32,
    pub border:     Color32,
    pub danger:     Color32,
    pub ok:         Color32,
    pub warn:       Color32,
    pub card_bg:    Color32,
    pub card_shadow:Color32,
}

impl Default for Palette {
    fn default() -> Self {
        // Breeze Light + olive accent (fallback if kdeglobals unreadable)
        Self {
            win_bg:      Color32::from_rgb(239, 240, 241),
            view_bg:     Color32::from_rgb(255, 255, 255),
            view_alt:    Color32::from_rgb(247, 247, 247),
            win_fg:      Color32::from_rgb(35,  38,  41),
            muted_fg:    Color32::from_rgb(120, 126, 133),
            hdr_bg:      Color32::from_rgb(222, 224, 226),
            accent:      Color32::from_rgb(114, 115,  57),
            accent_text: Color32::from_rgb(255, 255, 255),
            sel_bg:      Color32::from_rgb(156, 156, 116),
            btn_bg:      Color32::from_rgb(252, 252, 252),
            border:      Color32::from_rgb(196, 198, 200),
            danger:      Color32::from_rgb(192,  57,  43),
            ok:          Color32::from_rgb( 39, 174,  96),
            warn:        Color32::from_rgb(211, 162,  23),
            card_bg:     Color32::from_rgb(255, 255, 255),
            card_shadow: Color32::from_rgba_premultiplied(0, 0, 0, 18),
        }
    }
}

pub fn load_palette() -> Palette {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let path = format!("{}/.config/kdeglobals", home);

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Palette::default(),
    };

    let mut map: HashMap<(String, String), String> = HashMap::new();
    let mut section = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = line.to_string();
        } else if let Some((k, v)) = line.split_once('=') {
            map.insert((section.clone(), k.trim().to_string()), v.trim().to_string());
        }
    }

    let get = |sec: &str, key: &str| -> Option<Color32> {
        map.get(&(sec.to_string(), key.to_string()))
            .and_then(|v| parse_rgb(v))
    };

    let mut p = Palette::default();
    if let Some(c) = get("[Colors:Window]",    "BackgroundNormal")    { p.win_bg  = c; }
    if let Some(c) = get("[Colors:Window]",    "ForegroundNormal")    { p.win_fg  = c; }
    if let Some(c) = get("[Colors:View]",      "BackgroundNormal")    { p.view_bg = c; }
    if let Some(c) = get("[Colors:View]",      "BackgroundAlternate") { p.view_alt= c; }
    if let Some(c) = get("[Colors:Header]",    "BackgroundNormal")    { p.hdr_bg  = c; }
    if let Some(c) = get("[Colors:Button]",    "BackgroundNormal")    { p.btn_bg  = c; }
    if let Some(c) = get("[Colors:Selection]", "BackgroundNormal")    { p.sel_bg  = c; }

    let accent = get("[Colors:Window]", "DecorationFocus")
        .unwrap_or(p.sel_bg);
    p.accent = accent;

    // card is always white regardless of theme variant
    p.card_bg = p.view_bg;

    // muted is win_fg with reduced opacity simulation (blend toward win_bg)
    p.muted_fg = blend(p.win_fg, p.win_bg, 0.45);

    // border: slightly darker than win_bg
    p.border = darken(p.win_bg, 28);

    p
}

/// Apply palette as egui Visuals + Style
pub fn apply(ctx: &egui::Context, p: &Palette) {
    let mut vis = Visuals::light();

    vis.override_text_color = Some(p.win_fg);
    vis.panel_fill           = p.win_bg;
    vis.window_fill          = p.view_bg;
    vis.window_stroke        = Stroke::new(1.0, p.border);
    vis.window_rounding      = Rounding::same(8.0);

    // Widgets
    vis.widgets.noninteractive.bg_fill   = p.win_bg;
    vis.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.win_fg);
    vis.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.border);

    vis.widgets.inactive.bg_fill   = p.btn_bg;
    vis.widgets.inactive.fg_stroke = Stroke::new(1.0, p.win_fg);
    vis.widgets.inactive.bg_stroke = Stroke::new(1.0, p.border);
    vis.widgets.inactive.rounding  = Rounding::same(5.0);

    vis.widgets.hovered.bg_fill   = lighten(p.btn_bg, 6);
    vis.widgets.hovered.fg_stroke = Stroke::new(1.5, p.accent);
    vis.widgets.hovered.bg_stroke = Stroke::new(1.5, p.accent);
    vis.widgets.hovered.rounding  = Rounding::same(5.0);

    vis.widgets.active.bg_fill   = p.sel_bg;
    vis.widgets.active.fg_stroke = Stroke::new(1.0, p.accent_text);
    vis.widgets.active.rounding  = Rounding::same(5.0);

    vis.widgets.open.bg_fill   = p.hdr_bg;
    vis.widgets.open.fg_stroke = Stroke::new(1.0, p.win_fg);
    vis.widgets.open.rounding  = Rounding::same(5.0);

    vis.selection.bg_fill = p.sel_bg;
    vis.selection.stroke  = Stroke::new(1.0, p.accent);

    vis.hyperlink_color = p.accent;

    ctx.set_visuals(vis);

    // Font sizes
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (egui::TextStyle::Small,    FontId::proportional(11.0)),
        (egui::TextStyle::Body,     FontId::proportional(14.0)),
        (egui::TextStyle::Button,   FontId::proportional(14.0)),
        (egui::TextStyle::Heading,  FontId::proportional(20.0)),
        (egui::TextStyle::Monospace,FontId::monospace(13.0)),
    ].into();
    // Comfortable padding
    style.spacing.item_spacing    = egui::vec2(8.0, 6.0);
    style.spacing.button_padding  = egui::vec2(12.0, 6.0);
    style.spacing.window_margin   = egui::Margin::same(16.0);
    style.spacing.indent          = 14.0;

    ctx.set_style(style);
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn parse_rgb(s: &str) -> Option<Color32> {
    let p: Vec<&str> = s.split(',').collect();
    if p.len() == 3 {
        let r = p[0].trim().parse::<u8>().ok()?;
        let g = p[1].trim().parse::<u8>().ok()?;
        let b = p[2].trim().parse::<u8>().ok()?;
        Some(Color32::from_rgb(r, g, b))
    } else {
        None
    }
}

pub fn darken(c: Color32, amount: u8) -> Color32 {
    Color32::from_rgb(
        c.r().saturating_sub(amount),
        c.g().saturating_sub(amount),
        c.b().saturating_sub(amount),
    )
}

pub fn lighten(c: Color32, amount: u8) -> Color32 {
    Color32::from_rgb(
        c.r().saturating_add(amount),
        c.g().saturating_add(amount),
        c.b().saturating_add(amount),
    )
}

fn blend(fg: Color32, bg: Color32, t: f32) -> Color32 {
    let lerp = |a: u8, b: u8| -> u8 { (a as f32 * (1.0 - t) + b as f32 * t) as u8 };
    Color32::from_rgb(lerp(fg.r(), bg.r()), lerp(fg.g(), bg.g()), lerp(fg.b(), bg.b()))
}
