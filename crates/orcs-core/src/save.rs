//! Формат сохранения. JSON с номером версии и полным состоянием партии, включая ГПСЧ
//! и незавершённые решения (бой, финальный ответ, согласие на помощь, тайник).

use serde::{Deserialize, Serialize};

use crate::game::Game;

/// Формат 4 хранит две колоды, рынок, деньги, перки и районы. Старые смешанные партии не переносятся.
pub const SAVE_VERSION: u32 = 4;

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("сохранение повреждено: {0}")]
    Format(#[from] serde_json::Error),
    #[error("неподдерживаемая версия сохранения {0}; ожидается {SAVE_VERSION}")]
    Version(u32),
    #[error("в сохранении нет партии")]
    Empty,
}

#[derive(Serialize)]
struct SaveRef<'a> {
    version: u32,
    #[serde(flatten)]
    game: &'a Game,
}

#[derive(Deserialize)]
struct Header {
    version: u32,
}

#[derive(Deserialize)]
struct SaveOwned {
    #[serde(flatten)]
    game: Game,
}

impl Game {
    pub fn to_save_json(&self) -> String {
        serde_json::to_string(&SaveRef {
            version: SAVE_VERSION,
            game: self,
        })
        .expect("состояние сериализуется")
    }

    pub fn from_save_json(json: &str) -> Result<Game, SaveError> {
        let header: Header = serde_json::from_str(json)?;
        if header.version != SAVE_VERSION {
            return Err(SaveError::Version(header.version));
        }
        let save: SaveOwned = serde_json::from_str(json)?;
        if save.game.players.is_empty() {
            return Err(SaveError::Empty);
        }
        Ok(save.game)
    }
}
