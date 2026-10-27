//! Партия как ресурс Bevy и команды игроков. Интерфейс не меняет `Game` напрямую:
//! он отправляет [`GameCommand`], а система [`apply_commands`] вызывает движок, пишет ошибку
//! в подсказку и сохраняет партию после каждого успешного действия.

use bevy::prelude::*;
use orcs_core::{CardId, CardType, ChoiceOption, Game, RuleResult};

use crate::persistence::SaveStore;
use crate::ui::Screen;

#[derive(Resource, Default)]
pub struct Session {
    pub game: Game,
}

/// Локальное состояние интерфейса, не входящее в правила.
#[derive(Resource)]
pub struct TableUi {
    pub selected: Option<CardId>,
    /// Чья рука открыта после ширмы.
    pub private_owner: Option<usize>,
    pub toast: Option<String>,
    /// Герои, выбранные в меню.
    pub chosen: Vec<CardId>,
    pub gallery_filter: Option<CardType>,
    pub popup: Option<CardId>,
    pub discard_open: bool,
    pub shop_open: bool,
}

impl Default for TableUi {
    fn default() -> Self {
        Self {
            selected: None,
            private_owner: None,
            toast: None,
            chosen: vec!["vityok".into(), "zina".into()],
            gallery_filter: None,
            popup: None,
            discard_open: false,
            shop_open: false,
        }
    }
}

#[derive(Message, Clone, Debug, PartialEq)]
pub enum GameCommand {
    NewGame(Vec<CardId>),
    Continue,
    Explore,
    Work,
    Challenge,
    Buy(CardId, Option<CardId>),
    Sell(CardId),
    Refresh(CardId),
    Scout,
    PaidHelp,
    PaidBoost,
    Play(CardId),
    Recycle(CardId),
    Trade(CardId),
    DiscardExcess(CardId),
    EndTurn,
    PassReaction,
    RequestHelp,
    Choose(ChoiceOption),
}

/// Ход правил для команды. Команды меню (`NewGame`, `Continue`) обрабатываются системой.
pub fn execute(game: &mut Game, command: &GameCommand) -> RuleResult {
    match command {
        GameCommand::Explore => game.explore(),
        GameCommand::Work => game.work(),
        GameCommand::Challenge => game.challenge(),
        GameCommand::Buy(id, payment) => game.buy(id, payment.as_ref()),
        GameCommand::Sell(id) => game.sell(id),
        GameCommand::Refresh(id) => game.refresh_market(id),
        GameCommand::Scout => game.scout(),
        GameCommand::PaidHelp => game.request_paid_help(),
        GameCommand::PaidBoost => game.paid_boost(),
        GameCommand::Play(id) => game.play(id),
        GameCommand::Recycle(id) => game.recycle(id),
        GameCommand::Trade(id) => game.trade(id),
        GameCommand::DiscardExcess(id) => game.discard_excess(id),
        GameCommand::EndTurn => game.end_turn(),
        GameCommand::PassReaction => game.pass_reaction(),
        GameCommand::RequestHelp => game.request_help(),
        GameCommand::Choose(option) => game.choose(option.clone()),
        GameCommand::NewGame(_) | GameCommand::Continue => Ok(()),
    }
}

pub fn apply_commands(
    mut commands: MessageReader<GameCommand>,
    mut session: ResMut<Session>,
    mut ui: ResMut<TableUi>,
    store: Res<SaveStore>,
    mut screen: ResMut<NextState<Screen>>,
) {
    for command in commands.read() {
        match command {
            GameCommand::NewGame(heroes) => match session.game.start(heroes.iter().cloned(), 0) {
                Ok(()) => {
                    *ui = TableUi {
                        chosen: ui.chosen.clone(),
                        ..TableUi::default()
                    };
                    ui.toast = store.save(&session.game).err();
                    screen.set(Screen::Table);
                }
                Err(e) => ui.toast = Some(e.0),
            },
            GameCommand::Continue => match store.load() {
                Ok(game) => {
                    session.game = game;
                    *ui = TableUi {
                        chosen: ui.chosen.clone(),
                        ..TableUi::default()
                    };
                    screen.set(Screen::Table);
                }
                Err(e) => ui.toast = Some(e),
            },
            _ => match execute(&mut session.game, command) {
                Ok(()) => {
                    ui.selected = None;
                    ui.toast = store.save(&session.game).err();
                }
                Err(e) => ui.toast = Some(e.0),
            },
        }
    }
}
