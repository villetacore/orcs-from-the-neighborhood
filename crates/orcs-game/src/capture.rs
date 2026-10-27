//! `--capture`: снимок окна после загрузки иллюстраций — для документации и визуальной проверки.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};

use crate::assets::CardArt;

#[derive(Resource, Clone, Debug)]
pub struct CaptureOptions {
    pub path: PathBuf,
    /// Сколько кадров подождать, чтобы PNG успели загрузиться.
    pub delay_frames: u32,
}

pub struct CapturePlugin(pub CaptureOptions);

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.0.clone())
            .add_systems(Update, capture_after_delay);
    }
}

fn capture_after_delay(
    mut commands: Commands,
    options: Res<CaptureOptions>,
    server: Res<AssetServer>,
    art: Res<CardArt>,
    mut frames: Local<u32>,
    mut done: Local<bool>,
) {
    *frames += 1;
    // Снимок один раз: после задержки и загрузки всех иллюстраций (но не дольше 10 задержек).
    let loading = art.handles().iter().any(|h| !server.is_loaded_with_dependencies(h));
    if *done || *frames < options.delay_frames || (loading && *frames < options.delay_frames * 10) {
        return;
    }
    *done = true;
    let path = options.path.clone();
    info!("снимок окна → {}", path.display());
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(|_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            exit.write(AppExit::Success);
        });
}
