//! Базовые правила (перенесено из `tests/test_game.gd` прототипа на Godot).

mod common;

use std::collections::HashMap;

use common::*;
use orcs_core::{Boost, BoostSource, CardType, ChoiceKind, ChoiceOption as Opt, Effect, Game, Phase};

#[test]
fn catalog_and_initial_deal() {
    let mut g = Game::new();
    g.start(["vityok", "zina", "stalker", "goblin"], 42).unwrap();
    assert_eq!(g.cards.len(), 90, "90 карт");
    assert_eq!(g.deck.len(), 13, "28 Барахла минус 12 розданных и 3 в витрине");
    assert!(conserved(&g), "сохранность при раздаче");
    let mut counts: HashMap<CardType, usize> = HashMap::new();
    for c in g.cards.iter() {
        *counts.entry(c.card_type).or_default() += 1;
    }
    for (t, n) in orcs_core::cards::DECK_COMPOSITION {
        assert_eq!(counts[&t], n, "{t:?}");
    }
}

#[test]
fn start_validates_table() {
    let mut g = Game::new();
    assert!(g.start(["vityok"], 1).is_err());
    assert!(g.start(["vityok", "vityok"], 1).is_err());
    assert!(g.start(["vityok", "zombie"], 1).is_err());
    assert!(g.start(["vityok", "zina", "stalker", "goblin"], 1).is_ok());
}

#[test]
fn ties_chaos_limit_and_cancellation() {
    let mut g = fixture();
    g.players[0].level = 2;
    g.begin_battle(id("zombie"));
    assert!(g.can_win(), "ничья в пользу героя");
    g.players[0].hand = ids(&["salt", "bullet", "brine"]);
    assert!(g.play(&id("salt")).is_ok(), "хаос героя");
    assert!(g.play(&id("bullet")).is_err(), "одна карта хаоса на игрока");
    assert!(g.play(&id("brine")).is_ok(), "расходник после хаоса");
    g.pass_reaction().unwrap();
    g.players[1].hand = ids(&["magic"]);
    assert!(g.play(&id("magic")).is_ok(), "отмена соперником");
    assert_eq!(g.hero_strength(), 3, "отменён последний бонус, ранний остался");
}

#[test]
fn dragon_threshold_and_combat_victory() {
    let mut g = fixture();
    g.players[0].level = 9;
    g.players[0].unlocked_tiers = vec![3, 5, 7];
    g.begin_battle(id("dragon"));
    g.enemy_mod = -9;
    assert!(!g.can_win(), "дракону нужно 10 даже после ослабления");
    g.boosts = vec![Boost {
        amount: 1,
        source: BoostSource::Card(id("salt")),
    }];
    assert!(g.can_win(), "порог дракона достигнут");
    g.resolve_battle();
    assert!(
        g.winner.is_none() && g.players[0].level == 9,
        "обычный бой ограничен уровнем 9"
    );
}

#[test]
fn energy_cannot_win_and_skips_next_turn() {
    let mut g = fixture();
    g.players[0].level = 9;
    g.players[0].unlocked_tiers = vec![3, 5, 7];
    g.players[0].hand = ids(&["energy"]);
    assert!(g.play(&id("energy")).is_ok());
    assert!(
        g.players[0].level == 9 && g.winner.is_none(),
        "энергетик не даёт победу"
    );
    g.end_turn().unwrap();
    g.phase = Phase::Actions;
    g.end_turn().unwrap();
    g.phase = Phase::Actions;
    g.players[0].hand = ids(&["rebar"]);
    assert!(
        g.players[0].skip_now && g.play(&id("rebar")).is_err(),
        "энергетик пропускает действия"
    );
}

#[test]
fn cloak_offers_protection() {
    let mut g = fixture();
    g.players[0].armor = some("curtain");
    g.players[0].hand = ids(&["rebar"]);
    g.lose_card(0);
    assert_eq!(g.choices[0].kind, ChoiceKind::Protect);
    g.choose(Opt::Protect).unwrap();
    assert!(
        g.players[0].hand == ids(&["rebar"]) && g.players[0].armor.is_none(),
        "плащ сброшен"
    );
}

#[test]
fn witch_picks_substitute() {
    let mut g = fixture();
    g.players[1].hand = ids(&["helmet", "rusk"]);
    g.lose_card(1);
    assert_eq!(g.choices[0].kind, ChoiceKind::Witch);
    g.choose(Opt::card("rusk")).unwrap();
    assert!(g.players[1].hand == ids(&["helmet"]) && g.players[1].witch_used);
}

#[test]
fn trade_needs_consent_and_never_consumes_stash() {
    let mut g = fixture();
    g.players[0].hand = ids(&["rebar", "stash"]);
    g.players[1].hand = ids(&["helmet"]);
    assert!(g.trade(&id("rebar")).is_ok());
    g.choose(Opt::Player(1)).unwrap();
    assert_eq!(g.owner(), 1, "согласие — у получателя");
    g.choose(Opt::Decline).unwrap();
    assert_eq!(g.players[0].hand, ids(&["rebar", "stash"]), "отказ ничего не тратит");
    g.trade(&id("rebar")).unwrap();
    g.choose(Opt::Player(1)).unwrap();
    g.choose(Opt::card("helmet")).unwrap();
    assert_eq!(g.players[0].hand, ids(&["stash", "helmet"]));
    assert_eq!(g.players[1].hand, ids(&["rebar"]));
}

#[test]
fn kikimora_pipe() {
    let mut g = fixture();
    g.players[0].hand = ids(&["brine"]);
    g.begin_battle(id("kikimora"));
    g.choose(Opt::card("brine")).unwrap();
    assert!(
        g.players[0].hand.is_empty() && g.battle_penalty == 0,
        "сброс расходника вместо штрафа"
    );

    let mut g = fixture();
    g.begin_battle(id("kikimora"));
    g.choose(Opt::Penalty).unwrap();
    assert_eq!(g.battle_penalty, -1, "штраф кикиморы");
}

#[test]
fn club_victory_waits_for_cost() {
    let mut g = fixture();
    g.players[0].level = 9;
    g.players[0].weapon = some("club");
    g.players[0].hand = ids(&["helmet"]);
    g.players[0].unlocked_tiers = vec![3, 5, 7];
    g.begin_battle(id("boss"));
    g.enemy_mod = -10;
    g.resolve_battle();
    assert!(g.winner.is_none() && !g.choices.is_empty(), "победа ждёт цены дубины");
    let first = g.choices[0].options[0].clone();
    g.choose(first).unwrap();
    assert_eq!(g.winner, Some(0));
}

#[test]
fn hero_abilities() {
    let mut g = fixture();
    g.players[0].weapon = some("wrench");
    g.begin_battle(id("zombie"));
    assert_eq!(g.hero_strength(), 4, "механик: +1 за техснаряжение ровно один раз");
    g.players[0].silenced = true;
    assert_eq!(g.hero_strength(), 3, "молчание отключает способность");

    let mut g = fixture();
    g.players[0].hero = id("stalker");
    g.begin_battle(id("rat"));
    assert_eq!(g.hero_strength(), 2, "сталкер против мутанта");

    let mut g = fixture();
    g.players[0].weapon = some("slipper");
    g.begin_battle(id("zombie"));
    assert_eq!(g.hero_strength(), 4, "тапок против нежити");
}

#[test]
fn water_penalty_expires_at_own_turn() {
    let mut g = fixture();
    g.players[0].water = -1;
    g.end_turn().unwrap();
    g.phase = Phase::Actions;
    g.end_turn().unwrap();
    assert_eq!(g.players[0].water, 0);
}

#[test]
fn hand_limit() {
    let mut g = fixture();
    g.players[0].hand = ids(&["rebar", "helmet", "brine", "salt", "rumor", "luck"]);
    assert!(g.end_turn().is_err(), "лимит руки блокирует конец хода");
    assert!(g.discard_excess(&id("luck")).is_ok() && g.end_turn().is_ok());
}

#[test]
fn equipment_replaces_same_slot() {
    let mut g = fixture();
    g.players[0].hand = ids(&["helmet", "chain"]);
    g.play(&id("helmet")).unwrap();
    g.play(&id("chain")).unwrap();
    assert!(g.players[0].armor == some("chain") && g.discard.contains(&id("helmet")));
}

#[test]
fn conditional_chaos_rejected_without_consuming() {
    let mut g = fixture();
    g.players[0].hand = ids(&["friend", "battery", "magic"]);
    g.begin_battle(id("zombie"));
    for c in ["friend", "battery", "magic"] {
        assert!(g.play(&id(c)).is_err(), "{c}");
    }
    assert_eq!(g.players[0].hand.len(), 3, "неверный розыгрыш оставляет карты");
}

#[test]
fn every_card_effect_is_known_to_its_type() {
    // Защита от опечаток в data/cards.json: эффекты не должны «переезжать» между категориями.
    let g = Game::new();
    for c in g.cards.iter() {
        let ok = match c.card_type {
            CardType::Perk | CardType::District | CardType::Shop => c.effect == Effect::None,
            CardType::Character => {
                matches!(
                    c.effect,
                    Effect::Mechanic
                        | Effect::MutantHunter
                        | Effect::Merchant
                        | Effect::Witch
                        | Effect::Elf
                        | Effect::Dwarf
                        | Effect::Djinn
                        | Effect::Oni
                )
            }
            CardType::Enemy => matches!(
                c.effect,
                Effect::None
                    | Effect::Unarmored
                    | Effect::Pipe
                    | Effect::Armor
                    | Effect::Dragon
                    | Effect::Technical
                    | Effect::Boss
            ),
            CardType::Equipment => {
                matches!(
                    c.effect,
                    Effect::None | Effect::Undead | Effect::VictoryDiscard | Effect::Protect
                )
            }
            CardType::Consumable => matches!(
                c.effect,
                Effect::Boost | Effect::Cycle | Effect::Tape | Effect::Luck | Effect::Energy | Effect::Rusk
            ),
            CardType::Event => matches!(
                c.effect,
                Effect::Water
                    | Effect::Portal
                    | Effect::Silence
                    | Effect::Lift
                    | Effect::Raid
                    | Effect::Reveal
                    | Effect::Draw
                    | Effect::Rust
                    | Effect::Harvest
                    | Effect::Tax
            ),
            CardType::Chaos => matches!(
                c.effect,
                Effect::EnemyUp
                    | Effect::WeaponDown
                    | Effect::Ambush
                    | Effect::Friend
                    | Effect::Stash
                    | Effect::Cancel
                    | Effect::Battery
                    | Effect::EnemyDown
                    | Effect::HeroUp
                    | Effect::Noise
            ),
        };
        assert!(ok, "{}: {:?} для {:?}", c.id, c.effect, c.card_type);
    }
}

#[test]
fn enemy_rules_text_matches_penalty_data() {
    // Правило на лице врага берётся из поля penalty, а не дублируется вручную в тексте.
    let g = Game::new();
    for c in g.cards.iter().filter(|c| c.card_type == CardType::Enemy) {
        let penalty = c.penalty.expect("у врага есть последствие").description();
        assert!(c.rules_text().ends_with(penalty), "{}", c.id);
        assert!(
            !c.text.contains("При поражении"),
            "{}: последствие продублировано в тексте",
            c.id
        );
    }
}

#[test]
fn choice_titles_use_current_card_names() {
    let mut g = fixture();
    g.begin_battle(id("kikimora"));
    assert!(g.choices[0].title.starts_with(&g.card(&id("kikimora")).name));

    let mut g = fixture();
    g.players[0].level = 1;
    g.players[0].hand = ids(&["helmet", "rebar", "salt"]);
    g.begin_battle(id("witch"));
    g.enemy_mod = 20;
    g.resolve_battle();
    let name = &g.card(&id("witch")).name;
    assert!(
        g.choices.iter().any(|q| q.title.starts_with(name.as_str())),
        "вторая потеря названа по врагу"
    );
}
