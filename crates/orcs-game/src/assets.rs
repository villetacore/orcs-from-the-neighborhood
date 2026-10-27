//! Иллюстрации карт: загрузка PNG через `AssetServer` и регистрация текстур в egui.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiTextureHandle, egui};
use orcs_core::{CardDb, CardId};

pub const CARD_BACK: &str = "assets/art/card_back.png";

/// Текстуры egui для каждой карты и рубашки. Изображения подгружаются асинхронно;
/// пока PNG не готов, egui рисует пустой прямоугольник.
#[derive(Resource, Default)]
pub struct CardArt {
    textures: HashMap<CardId, egui::TextureId>,
    back: Option<egui::TextureId>,
    /// Сильные ссылки держат изображения в памяти всё время работы.
    handles: Vec<Handle<Image>>,
}

impl CardArt {
    pub fn texture(&self, id: &CardId) -> Option<egui::TextureId> {
        self.textures.get(id).copied()
    }

    pub fn back(&self) -> Option<egui::TextureId> {
        self.back
    }

    pub fn handles(&self) -> &[Handle<Image>] {
        &self.handles
    }
}

pub struct CardArtPlugin;

impl Plugin for CardArtPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CardArt>().add_systems(Startup, load_card_art);
    }
}

fn load_card_art(server: Res<AssetServer>, mut contexts: EguiContexts, mut art: ResMut<CardArt>) {
    let db = CardDb::shared();
    for card in db.iter() {
        let handle: Handle<Image> = server.load(card.image.clone());
        let texture = contexts.add_image(EguiTextureHandle::Strong(handle.clone()));
        art.textures.insert(card.id.clone(), texture);
        art.handles.push(handle);
    }
    let back: Handle<Image> = server.load(CARD_BACK);
    art.back = Some(contexts.add_image(EguiTextureHandle::Strong(back.clone())));
    art.handles.push(back);
}
