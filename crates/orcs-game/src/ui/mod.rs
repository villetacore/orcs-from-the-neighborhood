//! Интерфейс на egui. Экран — состояние Bevy [`Screen`]; каждый экран рисуется своим модулем
//! из общего контекста кадра [`View`]. Изменения правил уходят командами [`GameCommand`].

mod card;
mod gallery;
mod menu;
pub mod model;
mod rules;
mod table;
pub mod theme;
mod widgets;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiStartupSet, egui};
use orcs_core::Game;

use crate::assets::CardArt;
use crate::persistence::SaveStore;
use crate::session::{GameCommand, TableUi};

#[derive(States, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Screen {
    #[default]
    Menu,
    Table,
    Gallery,
    Rules,
}

/// Всё, что нужно экрану в одном кадре.
pub struct View<'a> {
    pub game: &'a Game,
    pub state: &'a mut TableUi,
    pub art: &'a CardArt,
    pub save_exists: bool,
    commands: Vec<GameCommand>,
    navigate: Option<Screen>,
}

impl View<'_> {
    pub fn send(&mut self, command: GameCommand) {
        self.commands.push(command);
    }

    pub fn go(&mut self, screen: Screen) {
        self.navigate = Some(screen);
    }

    /// Возврат из галереи и правил: к столу, если партия идёт, иначе в меню.
    pub fn go_back(&mut self) {
        self.go(if self.game.players.is_empty() {
            Screen::Menu
        } else {
            Screen::Table
        });
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Screen>()
            .add_systems(PreStartup, spawn_camera.before(EguiStartupSet::InitContexts))
            .add_systems(Startup, setup_style)
            .add_systems(EguiPrimaryContextPass, draw_ui);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_style(mut contexts: EguiContexts) -> Result {
    theme::apply_style(contexts.ctx_mut()?);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn draw_ui(
    mut contexts: EguiContexts,
    session: Res<crate::session::Session>,
    mut state: ResMut<TableUi>,
    art: Res<CardArt>,
    store: Res<SaveStore>,
    screen: Res<State<Screen>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut commands: MessageWriter<GameCommand>,
) -> Result {
    let ctx = contexts.ctx_mut()?.clone();
    let mut root = egui::Ui::new(
        ctx.clone(),
        "root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    widgets::paint_background(&root);
    let mut view = View {
        game: &session.game,
        state: &mut state,
        art: &art,
        save_exists: store.exists(),
        commands: Vec::new(),
        navigate: None,
    };
    match screen.get() {
        Screen::Menu => menu::show(&mut root, &mut view),
        Screen::Table if view.game.players.is_empty() => view.go(Screen::Menu),
        Screen::Table => table::show(&mut root, &mut view),
        Screen::Gallery => gallery::show(&mut root, &mut view),
        Screen::Rules => rules::show(&mut root, &mut view),
    }
    if let Some(target) = view.navigate {
        next_screen.set(target);
    }
    commands.write_batch(view.commands);
    Ok(())
}
