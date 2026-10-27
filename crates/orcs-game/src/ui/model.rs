//! Модель представления стола. Чистые функции без egui и Bevy: что показать и что разрешить.
//! Отрисовка в `table.rs` только раскладывает эти данные, поэтому их можно проверять тестами.

use orcs_core::{CardId, Choice, ChoiceKind, ChoiceOption, Effect, Game, Phase};

/// Кнопка с причиной, по которой она недоступна.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub label: String,
    pub enabled: bool,
    pub hint: String,
}

impl Action {
    fn new(label: &str, check: Result<(), String>, ok_hint: &str) -> Self {
        match check {
            Ok(()) => Self {
                label: label.into(),
                enabled: true,
                hint: ok_hint.into(),
            },
            Err(reason) => Self {
                label: label.into(),
                enabled: false,
                hint: reason,
            },
        }
    }
}

fn check(result: orcs_core::RuleResult) -> Result<(), String> {
    result.map_err(|e| e.0)
}

/// Ширма нужна, если руку сейчас должен видеть не тот, кто её открыл.
pub fn curtain_needed(game: &Game, private_owner: Option<usize>) -> bool {
    game.is_running() && private_owner != Some(game.owner())
}

pub fn phase_title(game: &Game) -> &'static str {
    if game.final_response() {
        return "02  /  ФИНАЛЬНЫЙ ОТВЕТ ГЕРОЯ";
    }
    match game.phase {
        Phase::Explore => "01  /  ПОДГОТОВКА И ИССЛЕДОВАНИЕ",
        Phase::Combat => "02  /  БОЙ И ВМЕШАТЕЛЬСТВО",
        Phase::Actions => "03  /  СНАРЯЖЕНИЕ И ДЕЙСТВИЯ",
    }
}

/// Кнопки под рукой текущего владельца для выбранной карты.
#[derive(Clone, Debug)]
pub struct HandActions {
    pub play: Action,
    pub recycle: Action,
    pub trade: Action,
    pub discard: Action,
}

pub fn hand_actions(game: &Game, selected: Option<&CardId>) -> HandActions {
    let none = || Err("Выберите карту с руки.".to_owned());
    let with = |f: &dyn Fn(&CardId) -> orcs_core::RuleResult| selected.map_or_else(none, |id| check(f(id)));
    // Обмен, замена и сброс — действия активного героя; в бою или чужом решении они закрыты.
    let own_turn = game.owner() == game.active;
    let gate = |r: Result<(), String>| {
        if own_turn {
            r
        } else {
            Err("Сейчас решение другого игрока.".into())
        }
    };
    HandActions {
        play: Action::new(
            "Сыграть / экипировать",
            with(&|id| game.play_check(id)),
            "Разыграть выбранную карту",
        ),
        recycle: Action::new(
            "Заменить",
            gate(with(&|id| game.recycle_check(id))),
            "Сбросить и взять новую. Раз за ход.",
        ),
        trade: Action::new(
            "Обмен",
            gate(with(&|id| game.trade_check(id))),
            "Предложить карту сопернику",
        ),
        discard: Action::new(
            "Сбросить лишнюю",
            gate(with(&|id| game.discard_excess_check(id))),
            "Сбросить до лимита 5",
        ),
    }
}

/// Подсказка в инспекторе для выбранной карты из своей руки.
pub fn play_hint(game: &Game, id: &CardId) -> String {
    match game.play_check(id) {
        Ok(()) => "Можно сыграть сейчас — кнопкой или перетаскиванием на стол.".into(),
        Err(reason) => reason.0,
    }
}

#[derive(Clone, Debug)]
pub struct CombatView {
    pub score: String,
    pub breakdown: String,
    pub turn: String,
    pub reaction: String,
    pub chaos: String,
    pub helper: Option<String>,
    /// Кнопка помощи видна только до первого круга вмешательства.
    pub help: Option<Action>,
    pub dragon: bool,
    pub confirm: String,
}

pub const CONFIRM_BATTLE: &str = "ПОДТВЕРДИТЬ БОЙ";
pub const PASS_TURN: &str = "ПЕРЕДАТЬ ОЧЕРЕДЬ →";
pub const OFFER_HELP: &str = "ПОМОЩЬ ЗА ДОБЫЧУ";

pub fn combat_view(game: &Game) -> Option<CombatView> {
    let enemy = game.encounter.as_ref()?;
    let final_response = game.final_response();
    let chaos_used = game.chaos_used.contains(&game.owner());
    Some(CombatView {
        score: format!("ГЕРОЙ {}  :  {} ВРАГ", game.hero_strength(), game.enemy_strength()),
        breakdown: game.strength_text(),
        turn: format!(
            "{}{}",
            if final_response {
                "Последнее слово: "
            } else {
                "Вмешательство: "
            },
            game.player_name(game.reaction)
        ),
        reaction: game.reaction_text(),
        chaos: format!(
            "Хаос: {}. Расходники героя доступны и в финальном ответе.",
            if chaos_used {
                "уже сыгран"
            } else {
                "ещё можно сыграть одну карту"
            }
        ),
        helper: game
            .helper
            .map(|h| format!("Помощник: {}. {}", game.player_name(h), game.help_payment_text())),
        help: (game.reaction_step == 0 && !game.help_requested).then(|| {
            Action::new(
                OFFER_HELP,
                check(game.help_check()),
                "+2 силы за сброс карты помощником. Первая добыча — помощнику.",
            )
        }),
        dragon: game.card(enemy).effect == Effect::Dragon,
        confirm: if final_response { CONFIRM_BATTLE } else { PASS_TURN }.into(),
    })
}

#[derive(Clone, Debug)]
pub struct DecisionView {
    pub title: String,
    /// Для «Бабка всё видела»: названия карт открытой руки.
    pub revealed: Vec<String>,
    pub options: Vec<(String, ChoiceOption)>,
}

pub fn decision_view(game: &Game, choice: &Choice) -> DecisionView {
    let revealed = match choice.kind {
        ChoiceKind::Reveal { subject } => game.players[subject]
            .hand
            .iter()
            .map(|id| game.card(id).name.clone())
            .collect(),
        _ => Vec::new(),
    };
    DecisionView {
        title: choice.title.clone(),
        revealed,
        options: choice
            .options
            .iter()
            .map(|o| (game.option_label(choice, o), o.clone()))
            .collect(),
    }
}

/// Карты, которые можно тащить на стол: только при открытой руке и без незавершённых решений.
pub fn draggable(game: &Game, id: &CardId) -> bool {
    game.choices.is_empty() && game.play_check(id).is_ok()
}

/// «1 КАРТА», «2 КАРТЫ», «5 КАРТ».
pub fn cards_word(n: u32) -> String {
    let word = match (n % 10, n % 100) {
        (1, 11) | (2..=4, 12..=14) | (0 | 5..=9, _) => "КАРТ",
        (1, _) => "КАРТА",
        _ => "КАРТЫ",
    };
    format!("{n} {word}")
}

pub fn hand_title(game: &Game) -> String {
    let who = game.owner();
    let size = game.players[who].hand.len();
    format!("РУКА: {}   /   {}", game.player_name(who), cards_word(size as u32))
}

pub fn roster_line(game: &Game, p: usize) -> String {
    format!(
        "УР. {} · РУКА {}/{} · {} КР.",
        game.players[p].level,
        game.players[p].hand.len(),
        game.hand_limit(p),
        game.players[p].coins
    )
}

#[cfg(test)]
mod tests {
    //! Перенос `tests/test_ui.gd`: ширма, финальный ответ, кнопка помощи и согласие помощника.
    use super::*;

    fn four_players() -> Game {
        let mut g = Game::new();
        g.start(["vityok", "zina", "stalker", "goblin"], 42).unwrap();
        g
    }

    fn force_battle(g: &mut Game, enemy: &str) {
        let id = CardId::from(enemy);
        g.deck.retain(|c| *c != id);
        g.discard.retain(|c| *c != id);
        for p in &mut g.players {
            p.hand.retain(|c| *c != id);
        }
        g.begin_battle(id);
    }

    #[test]
    fn initial_hand_is_concealed() {
        let g = four_players();
        assert!(curtain_needed(&g, None));
        assert!(!curtain_needed(&g, Some(g.owner())));
    }

    #[test]
    fn owner_change_hides_hands() {
        let mut g = four_players();
        let viewer = g.owner();
        g.choices.push_front(Choice {
            player: (viewer + 1) % 4,
            kind: ChoiceKind::Reveal { subject: g.active },
            title: "Скрытый выбор".into(),
            options: vec![ChoiceOption::Ok],
        });
        assert!(curtain_needed(&g, Some(viewer)), "смена владельца закрывает стол");
    }

    #[test]
    fn final_response_is_explicit_and_help_closed() {
        let mut g = four_players();
        force_battle(&mut g, "zombie");
        let opening = combat_view(&g).unwrap();
        assert_eq!(opening.confirm, PASS_TURN);
        assert!(opening.help.is_some(), "в начале боя есть кнопка помощи");
        while !g.final_response() {
            g.pass_reaction().unwrap();
        }
        let last = combat_view(&g).unwrap();
        assert_eq!(last.confirm, CONFIRM_BATTLE, "финальное подтверждение явное");
        assert!(last.help.is_none(), "после первого круга помощь не предлагают");
        assert_eq!(phase_title(&g), "02  /  ФИНАЛЬНЫЙ ОТВЕТ ГЕРОЯ");
    }

    #[test]
    fn help_consent_switches_owner_and_can_be_declined() {
        let mut g = four_players();
        g.active = 0;
        force_battle(&mut g, "zombie");
        let target = g.players.iter().skip(1).position(|p| !p.hand.is_empty()).unwrap() + 1;
        g.request_help().unwrap();
        g.choose(ChoiceOption::Player(target)).unwrap();
        assert!(curtain_needed(&g, Some(0)), "согласие помощника — под ширмой");
        let view = decision_view(&g, g.current_choice().unwrap());
        assert!(view.options.iter().any(|(label, _)| label == "Отказаться"));
    }

    #[test]
    fn hand_actions_explain_restrictions() {
        let mut g = four_players();
        let actions = hand_actions(&g, None);
        assert!(!actions.play.enabled && actions.play.hint == "Выберите карту с руки.");
        force_battle(&mut g, "zombie");
        let card = g.players[g.owner()].hand[0].clone();
        let actions = hand_actions(&g, Some(&card));
        assert!(!actions.recycle.enabled, "в бою замена закрыта");
    }

    #[test]
    fn winner_removes_curtain() {
        let mut g = four_players();
        g.winner = Some(0);
        assert!(!curtain_needed(&g, None));
    }

    #[test]
    fn card_counts_agree_in_number() {
        for (n, expected) in [
            (0, "0 КАРТ"),
            (1, "1 КАРТА"),
            (2, "2 КАРТЫ"),
            (3, "3 КАРТЫ"),
            (5, "5 КАРТ"),
            (11, "11 КАРТ"),
            (12, "12 КАРТ"),
            (21, "21 КАРТА"),
            (22, "22 КАРТЫ"),
        ] {
            assert_eq!(cards_word(n), expected);
        }
    }
}
