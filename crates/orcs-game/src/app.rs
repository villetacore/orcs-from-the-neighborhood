//! Сборка приложения Bevy из плагинов и параметры запуска.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy_egui::EguiPlugin;
use orcs_core::GAME_TITLE;

use crate::assets::CardArtPlugin;
use crate::capture::{CaptureOptions, CapturePlugin};
use crate::persistence::{SaveStore, default_save_path, project_root};
use crate::session::{GameCommand, Session, TableUi, apply_commands};
use crate::ui::{Screen, UiPlugin};

/// Параметры командной строки.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Демонстрационная партия на троих (сид 43) сразу за столом; сохранение отключено.
    pub preview: bool,
    /// Стартовый экран.
    pub screen: Option<Screen>,
    /// Сделать снимок окна в этот PNG и выйти.
    pub capture: Option<PathBuf>,
    /// Другой файл сохранения.
    pub save: Option<PathBuf>,
}

impl Options {
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut options = Options::default();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            let mut value = || args.next().ok_or(format!("для {arg} нужно значение"));
            match arg.as_str() {
                "--preview" => options.preview = true,
                "--capture" => options.capture = Some(value()?.into()),
                "--save" => options.save = Some(value()?.into()),
                "--screen" => {
                    options.screen = Some(match value()?.as_str() {
                        "menu" => Screen::Menu,
                        "table" => Screen::Table,
                        "gallery" => Screen::Gallery,
                        "rules" => Screen::Rules,
                        other => return Err(format!("неизвестный экран {other}")),
                    })
                }
                "-h" | "--help" => return Err(USAGE.into()),
                other => return Err(format!("неизвестный параметр {other}\n{USAGE}")),
            }
        }
        Ok(options)
    }
}

pub const USAGE: &str = "orcs-from-the-neighborhood [--preview] [--screen menu|table|gallery|rules] [--capture out.png] [--save file.json] [--help | --version | --check-assets]";

pub fn build(options: Options) -> App {
    let mut app = App::new();
    let root = project_root();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: root.to_string_lossy().into_owned(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("{GAME_TITLE} · v{}", env!("CARGO_PKG_VERSION")),
                    resolution: WindowResolution::default(),
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(EguiPlugin::default())
    .insert_resource(ClearColor(Color::srgb_u8(0x18, 0x23, 0x21)))
    .add_message::<GameCommand>()
    .add_plugins((CardArtPlugin, UiPlugin))
    .add_systems(Startup, size_window)
    .add_systems(Update, apply_commands);

    let save = if options.preview {
        None
    } else {
        options.save.clone().or_else(default_save_path)
    };
    app.insert_resource(SaveStore { path: save });

    let mut session = Session::default();
    let mut ui = TableUi::default();
    if options.preview {
        session
            .game
            .start(["rashid", "elvira", "boris"], 43)
            .expect("демонстрационный стол");
        session.game.explore().expect("первое исследование");
        ui.private_owner = Some(session.game.owner());
    }
    app.insert_resource(session).insert_resource(ui);
    let start_screen = options
        .screen
        .unwrap_or(if options.preview { Screen::Table } else { Screen::Menu });
    app.insert_state(start_screen);

    if let Some(path) = options.capture {
        app.add_plugins(CapturePlugin(CaptureOptions { path, delay_frames: 90 }));
    }
    app
}

/// 1440×900 логических пикселей, как у прототипа; размер задаётся после создания окна,
/// чтобы учесть масштаб Retina.
fn size_window(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    window.resolution.set(1440.0, 900.0);
}
