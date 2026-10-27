//! # orcs-core
//!
//! Движок правил карточной RPG «Орки с района». Чистая логика без интерфейса:
//! любой клиент (Bevy, тесты, будущие боты или сеть) управляет партией через методы [`Game`].
//!
//! * [`cards`] — каталог карт из `data/cards.json`;
//! * [`state`] — игроки, фазы, бонусы и отложенные решения;
//! * [`Game`] — партия; её правила разнесены по модулям `combat`, `effects`, `play`, `choices`, `turn`;
//! * [`save`] — формат сохранения.

pub mod cards;
mod choices;
mod combat;
mod economy;
mod effects;
mod game;
mod play;
pub mod rng;
pub mod save;
pub mod state;
mod turn;

pub use cards::{CardDb, CardDef, CardId, CardType, Effect, Penalty, Slot, Tag};
pub use combat::{DRAGON_THRESHOLD, HELP_BONUS};
pub use game::{
    Game, HAND_LIMIT, MAX_PLAYERS, MIN_PLAYERS, NON_COMBAT_LEVEL_CAP, Rejection, RuleResult, STARTING_HAND, WIN_LEVEL,
};
pub use save::{SAVE_VERSION, SaveError};
pub use state::{Boost, BoostSource, Choice, ChoiceKind, ChoiceOption, Phase, Player, StrengthPart};

/// Название игры — для окна, меню и печатной колоды.
pub const GAME_TITLE: &str = "Орки с района";
