//! Пути проекта и автосохранение партии.

use std::path::PathBuf;

use bevy::prelude::Resource;
use directories::ProjectDirs;
use orcs_core::Game;

/// Корень проекта с `assets/` и `data/`. Порядок поиска: `ORCS_ROOT`, папки над исполняемым файлом,
/// текущая рабочая папка. Абсолютный путь машины сборки в бинарник не встраивается.
pub fn project_root() -> PathBuf {
    if let Some(root) = std::env::var_os("ORCS_ROOT") {
        return root.into();
    }
    if let Ok(exe) = std::env::current_exe() {
        for dir in exe.ancestors().skip(1) {
            if dir.join("assets/art").is_dir() {
                return dir.to_path_buf();
            }
            // Standard macOS .app layout: Contents/MacOS + Contents/Resources.
            let resources = dir.join("Resources");
            if resources.join("assets/art").is_dir() {
                return resources;
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Файл автосохранения: `<data_dir>/table_save.json`
/// (на macOS — `~/Library/Application Support/com.villetacore.orcs-from-the-neighborhood/`).
pub fn default_save_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "villetacore", "orcs-from-the-neighborhood").map(|d| d.data_dir().join("table_save.json"))
}

#[derive(Resource, Clone, Debug)]
pub struct SaveStore {
    /// `None` — сохранение отключено (режим превью).
    pub path: Option<PathBuf>,
}

impl SaveStore {
    pub fn exists(&self) -> bool {
        self.path.as_ref().is_some_and(|p| p.is_file())
    }

    pub fn save(&self, game: &Game) -> Result<(), String> {
        let Some(path) = &self.path else { return Ok(()) };
        let write = || -> std::io::Result<()> {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            // Через временный файл, чтобы сбой записи не портил прежнее сохранение.
            let tmp = path.with_extension("json.tmp");
            std::fs::write(&tmp, game.to_save_json())?;
            std::fs::rename(tmp, path)
        };
        write().map_err(|e| format!("Не удалось записать сохранение: {e}"))
    }

    pub fn load(&self) -> Result<Game, String> {
        let path = self.path.as_ref().ok_or("Сохранение отключено.")?;
        let json = std::fs::read_to_string(path).map_err(|e| format!("Сохранение не удалось прочитать: {e}"))?;
        Game::from_save_json(&json).map_err(|e| format!("Сохранение не удалось прочитать: {e}"))
    }
}
