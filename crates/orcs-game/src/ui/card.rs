//! Лицо карты: иллюстрация отдельно, детерминированный читаемый текст поверх рамки.

use bevy_egui::egui::{
    self, Align2, Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2,
    text::LayoutJob,
};
use orcs_core::{CardDef, CardId, CardType};

use super::model::cards_word;
use super::theme;
use crate::assets::CardArt;

/// Размер компактной карты в руке.
pub const HAND_CARD: Vec2 = Vec2::new(162.0, 242.0);
/// Пропорции иллюстраций (запрос 3:4).
const ART_ASPECT: f32 = 3.0 / 4.0;

pub struct CardFace<'a> {
    card: &'a CardDef,
    art: &'a CardArt,
    size: Vec2,
    compact: bool,
    selected: bool,
    draggable: bool,
}

impl<'a> CardFace<'a> {
    pub fn new(card: &'a CardDef, art: &'a CardArt, size: Vec2) -> Self {
        Self {
            card,
            art,
            size,
            compact: false,
            selected: false,
            draggable: false,
        }
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Разрешить перетаскивание на стол. Полезная нагрузка — [`CardId`].
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let sense = if self.draggable {
            Sense::click_and_drag()
        } else {
            Sense::click()
        };
        let (rect, response) = ui.allocate_exact_size(self.size, sense);
        if self.draggable {
            response.dnd_set_drag_payload(self.card.id.clone());
            if response.dragged() {
                drag_preview(ui, &self.card.name);
            }
        }
        if ui.is_rect_visible(rect) {
            self.paint(ui, rect, response.hovered());
        }
        response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(tooltip(self.card))
    }

    fn paint(&self, ui: &Ui, rect: Rect, hovered: bool) {
        let card = self.card;
        let painter = ui.painter_at(rect.expand(8.0));
        let color = theme::card_color(card.card_type);
        let compact = self.compact;
        let border = if self.selected {
            Stroke::new(3.0, theme::CARD_SELECTED)
        } else if hovered {
            Stroke::new(2.5, theme::darken(color, 0.05))
        } else {
            Stroke::new(2.0, theme::darken(color, 0.25))
        };
        let radius = CornerRadius::same(9);
        painter.add(
            egui::Shadow {
                offset: [2, 4],
                blur: 10,
                spread: 0,
                color: Color32::from_black_alpha(100),
            }
            .as_shape(rect, radius),
        );
        painter.rect(rect, radius, theme::CARD_PAPER, border, StrokeKind::Inside);

        let inner = rect.shrink2(Vec2::new(9.0, 8.0));
        let width = inner.width();
        let gap = 5.0;
        let font = |size: f32| FontId::new(size, FontFamily::Proportional);
        let bold = |size: f32| FontId::new(size, FontFamily::Name(theme::BOLD.into()));

        let tag = format!("{}   /   {}", card.card_type.title(), card.id.as_str().to_uppercase());
        let tag = layout(
            ui,
            &tag,
            font(if compact { 10.0 } else { 12.0 }),
            theme::darken(color, 0.35),
            width,
            1,
        );
        let mut y = inner.top();
        painter.galley(Pos2::new(inner.left(), y), tag.clone(), Color32::PLACEHOLDER);
        y += tag.size().y + gap;

        let title = layout(
            ui,
            &card.name,
            bold(if compact { 15.0 } else { 21.0 }),
            theme::CARD_INK,
            width,
            2,
        );
        painter.galley(Pos2::new(inner.left(), y), title, Color32::PLACEHOLDER);
        y += if compact { 35.0 } else { 53.0 } + gap;

        let stats = layout(
            ui,
            &stats_line(card),
            bold(if compact { 13.0 } else { 17.0 }),
            theme::CARD_STATS,
            width,
            2,
        );
        let desc_rows = if compact { 3 } else { 6 };
        let desc = layout(
            ui,
            &card.rules_text(),
            font(if compact { 12.0 } else { 15.0 }),
            theme::CARD_TEXT,
            width,
            desc_rows,
        );
        let desc_h = if compact {
            desc.size().y
        } else {
            desc.size().y.max(65.0)
        };
        let desc_top = inner.bottom() - desc_h;
        let stats_top = desc_top - gap - stats.size().y;
        painter.galley(Pos2::new(inner.left(), stats_top), stats, Color32::PLACEHOLDER);
        painter.galley(Pos2::new(inner.left(), desc_top), desc, Color32::PLACEHOLDER);

        let pic = Rect::from_min_max(Pos2::new(inner.left(), y), Pos2::new(inner.right(), stats_top - gap));
        if pic.height() > 4.0 {
            paint_art(&painter, pic, card, self.art.texture(&card.id));
        }
    }
}

fn layout(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    width: f32,
    max_rows: usize,
) -> std::sync::Arc<egui::Galley> {
    let mut job = LayoutJob::simple(text.to_owned(), font, color, width);
    job.wrap.max_rows = max_rows;
    job.wrap.overflow_character = Some('…');
    ui.fonts_mut(|f| f.layout_job(job))
}

pub fn stats_line(card: &CardDef) -> String {
    match card.card_type {
        CardType::Enemy => {
            format!(
                "СИЛА {} · +{} УР. · {} · {} КР.",
                card.strength,
                card.level_reward,
                cards_word(card.loot_cards),
                card.coins
            )
        }
        CardType::Equipment => {
            let slot = card.slot.map_or("", |s| {
                if s == orcs_core::Slot::Weapon {
                    "ОРУЖИЕ"
                } else {
                    "БРОНЯ"
                }
            });
            format!("+{} СИЛЫ / {slot} / {} КР.", card.bonus, card.price)
        }
        CardType::Consumable | CardType::Chaos => format!("ЦЕНА {} КР. / ПРОДАЖА 1", card.price),
        CardType::Perk => format!("СТУПЕНЬ {} / ПОСТОЯННЫЙ", card.tier),
        _ => "✦  ОРКИ С РАЙОНА".to_owned(),
    }
}

pub fn tooltip(card: &CardDef) -> String {
    format!("{}\n{}\n{}", card.name, card.rules_text(), card.flavor)
}

/// Предметы показываются целиком, портреты кадрируются сверху, чтобы лица оставались в кадре.
pub fn paint_art(painter: &egui::Painter, target: Rect, card: &CardDef, texture: Option<egui::TextureId>) {
    let radius = CornerRadius::same(5);
    let Some(texture) = texture else {
        painter.rect_filled(target, radius, theme::CARD_MAT);
        painter.text(
            target.center(),
            Align2::CENTER_CENTER,
            "✦",
            FontId::proportional(50.0),
            theme::CARD_STATS,
        );
        return;
    };
    let full_uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    match card.card_type {
        CardType::Equipment | CardType::Consumable | CardType::Chaos => {
            painter.rect_filled(target, radius, theme::CARD_MAT);
            let scale = (target.width() / ART_ASPECT).min(target.height());
            let fitted = Vec2::new(scale * ART_ASPECT, scale);
            painter.image(
                texture,
                Rect::from_center_size(target.center(), fitted),
                full_uv,
                Color32::WHITE,
            );
        }
        _ => {
            let top_anchor = matches!(card.card_type, CardType::Enemy | CardType::Character);
            let uv = cover_uv(target.width() / target.height(), top_anchor);
            painter.image(texture, target, uv, Color32::WHITE);
        }
    }
}

/// UV-окно, которое заполняет цель с соотношением `ratio` без искажений.
fn cover_uv(ratio: f32, top_anchor: bool) -> Rect {
    if ART_ASPECT < ratio {
        let h = ART_ASPECT / ratio;
        let top = if top_anchor {
            (1.0 - h).min(0.02)
        } else {
            (1.0 - h) / 2.0
        };
        Rect::from_min_max(Pos2::new(0.0, top), Pos2::new(1.0, top + h))
    } else {
        let w = ratio / ART_ASPECT;
        let left = (1.0 - w) / 2.0;
        Rect::from_min_max(Pos2::new(left, 0.0), Pos2::new(left + w, 1.0))
    }
}

fn drag_preview(ui: &Ui, name: &str) {
    let Some(pos) = ui.ctx().pointer_interact_pos() else {
        return;
    };
    egui::Area::new(egui::Id::new("card-drag-preview"))
        .order(egui::Order::Tooltip)
        .fixed_pos(pos + Vec2::new(12.0, 12.0))
        .interactable(false)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(Color32::from_rgba_unmultiplied(0xde, 0xc8, 0x9a, 235))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(12, 6))
                .show(ui, |ui| ui.label(theme::heading(name, 20.0, theme::CARD_INK)));
        });
}

/// Карта, которую тащат на стол прямо сейчас.
pub fn dragged_card(ctx: &egui::Context) -> Option<CardId> {
    egui::DragAndDrop::payload::<CardId>(ctx).map(|id| (*id).clone())
}
