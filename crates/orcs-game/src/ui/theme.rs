//! Палитра «Байки ржавого двора», стили egui и шрифты.

use std::path::Path;
use std::sync::Arc;

use bevy_egui::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame, Margin, RichText, Stroke,
};
use orcs_core::CardType;

pub const BACKGROUND: Color32 = Color32::from_rgb(0x18, 0x23, 0x21);
pub const TEXT: Color32 = Color32::from_rgb(0xe6, 0xdc, 0xbe);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0xaa, 0xa9, 0x85);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x9f, 0xa4, 0x8d);
pub const GOLD: Color32 = Color32::from_rgb(0xd7, 0xba, 0x81);
pub const GOLD_LIGHT: Color32 = Color32::from_rgb(0xdf, 0xc1, 0x8a);
pub const RUST: Color32 = Color32::from_rgb(0xd7, 0x8b, 0x55);
pub const AMBER: Color32 = Color32::from_rgb(0xe5, 0xba, 0x77);
pub const WARNING: Color32 = Color32::from_rgb(0xcd, 0x87, 0x6a);
pub const PANEL: Color32 = Color32::from_rgb(0x24, 0x32, 0x2c);
pub const PANEL_ACTIVE: Color32 = Color32::from_rgb(0x3b, 0x48, 0x38);
pub const PANEL_DEEP: Color32 = Color32::from_rgb(0x1c, 0x29, 0x24);
pub const TABLE: Color32 = Color32::from_rgb(0x25, 0x3c, 0x32);
pub const LOG: Color32 = Color32::from_rgb(0x1d, 0x2e, 0x28);
pub const CURTAIN: Color32 = Color32::from_rgb(0x15, 0x23, 0x1f);
pub const WIN_BANNER: Color32 = Color32::from_rgb(0x53, 0x51, 0x32);
pub const PANEL_BORDER: Color32 = Color32::from_rgb(0x46, 0x53, 0x49);
pub const CARD_PAPER: Color32 = Color32::from_rgb(0xde, 0xd0, 0xaa);
pub const CARD_MAT: Color32 = Color32::from_rgb(0xc4, 0xb6, 0x8e);
pub const CARD_INK: Color32 = Color32::from_rgb(0x26, 0x33, 0x2f);
pub const CARD_STATS: Color32 = Color32::from_rgb(0x4c, 0x43, 0x2e);
pub const CARD_TEXT: Color32 = Color32::from_rgb(0x34, 0x37, 0x2d);
pub const CARD_SELECTED: Color32 = Color32::from_rgb(0xf5, 0xc7, 0x80);

/// Полужирное начертание (если в системе нашёлся шрифт).
pub const BOLD: &str = "bold";

pub fn card_color(card_type: CardType) -> Color32 {
    match card_type {
        CardType::Character => Color32::from_rgb(0x64, 0x88, 0x97),
        CardType::Enemy => Color32::from_rgb(0xa9, 0x56, 0x40),
        CardType::Equipment => Color32::from_rgb(0xaf, 0x94, 0x4b),
        CardType::Consumable => Color32::from_rgb(0x87, 0x99, 0x51),
        CardType::Event => Color32::from_rgb(0x8c, 0x72, 0x94),
        CardType::Perk => Color32::from_rgb(0x75, 0x8f, 0x62),
        CardType::District => Color32::from_rgb(0x63, 0x82, 0x91),
        CardType::Shop => Color32::from_rgb(0xb9, 0x8b, 0x47),
        CardType::Chaos => Color32::from_rgb(0xda, 0x82, 0x45),
    }
}

pub fn darken(color: Color32, amount: f32) -> Color32 {
    let k = 1.0 - amount;
    Color32::from_rgb(
        (color.r() as f32 * k) as u8,
        (color.g() as f32 * k) as u8,
        (color.b() as f32 * k) as u8,
    )
}

pub fn text(s: impl Into<String>, size: f32, color: Color32) -> RichText {
    RichText::new(s).size(size).color(color)
}

pub fn heading(s: impl Into<String>, size: f32, color: Color32) -> RichText {
    RichText::new(s)
        .font(FontId::new(size, FontFamily::Name(BOLD.into())))
        .color(color)
}

pub fn panel(fill: Color32) -> Frame {
    Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, PANEL_BORDER))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(16))
}

pub fn apply_style(ctx: &egui::Context) {
    ctx.set_fonts(fonts());
    ctx.all_styles_mut(configure);
}

fn configure(style: &mut egui::Style) {
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 9.0);
    style.spacing.interact_size.y = 34.0;
    for (text_style, size) in [
        (egui::TextStyle::Body, 16.0),
        (egui::TextStyle::Button, 16.0),
        (egui::TextStyle::Small, 12.0),
        (egui::TextStyle::Heading, 24.0),
    ] {
        style.text_styles.insert(text_style, FontId::proportional(size));
    }
    let v = &mut style.visuals;
    v.dark_mode = true;
    v.override_text_color = None;
    v.panel_fill = BACKGROUND;
    v.window_fill = PANEL;
    v.window_stroke = Stroke::new(1.0, PANEL_BORDER);
    v.extreme_bg_color = PANEL_DEEP;
    v.faint_bg_color = PANEL;
    v.selection.bg_fill = Color32::from_rgb(0x9b, 0x61, 0x3b);
    v.hyperlink_color = AMBER;
    let radius = CornerRadius::same(5);
    let button = |fill: Color32, border: Color32, fg: Color32| egui::style::WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(1.0, border),
        corner_radius: radius,
        fg_stroke: Stroke::new(1.0, fg),
        expansion: 0.0,
    };
    let fg = Color32::from_rgb(0xee, 0xe0, 0xbc);
    v.widgets.noninteractive = button(PANEL, PANEL_BORDER, TEXT);
    v.widgets.inactive = button(
        Color32::from_rgb(0x34, 0x43, 0x3c),
        Color32::from_rgb(0x67, 0x73, 0x65),
        fg,
    );
    v.widgets.hovered = button(
        Color32::from_rgb(0x4c, 0x5f, 0x4b),
        Color32::from_rgb(0xdd, 0xbd, 0x7c),
        fg,
    );
    v.widgets.active = button(
        Color32::from_rgb(0x9b, 0x61, 0x3b),
        Color32::from_rgb(0xdd, 0xbd, 0x7c),
        fg,
    );
    v.widgets.open = v.widgets.hovered;
    v.disabled_alpha = 0.45;
}

/// Встроенные шрифты egui + системные шрифты с кириллицей и типографскими символами, если они есть.
fn fonts() -> FontDefinitions {
    const REGULAR: &[&str] = &[
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];
    const BOLD_FILES: &[&str] = &[
        "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
        "C:\\Windows\\Fonts\\arialbd.ttf",
    ];
    const SYMBOLS: &[&str] = &[
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Apple Symbols.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "C:\\Windows\\Fonts\\seguisym.ttf",
    ];
    let mut defs = FontDefinitions::default();
    let mut load = |name: &str, candidates: &[&str]| -> Option<String> {
        let bytes = candidates.iter().find_map(|p| std::fs::read(Path::new(p)).ok())?;
        defs.font_data
            .insert(name.to_owned(), Arc::new(FontData::from_owned(bytes)));
        Some(name.to_owned())
    };
    let regular = load("system-regular", REGULAR);
    let bold = load("system-bold", BOLD_FILES);
    let symbols = load("system-symbols", SYMBOLS);
    let proportional = defs.families.entry(FontFamily::Proportional).or_default();
    if let Some(regular) = regular {
        proportional.insert(0, regular);
    }
    proportional.extend(symbols);
    let mut bold_family = proportional.clone();
    if let Some(bold) = bold {
        bold_family.insert(0, bold);
    }
    defs.families.insert(FontFamily::Name(BOLD.into()), bold_family);
    defs
}
