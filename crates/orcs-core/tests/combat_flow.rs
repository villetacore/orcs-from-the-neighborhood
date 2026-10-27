//! Подготовка, финальный ответ, помощь, тайник и котлета: сценарии игрового цикла.

mod common;

use common::*;
use orcs_core::{ChoiceKind, ChoiceOption as Opt, Game, Phase};

#[test]
fn preparation_allows_only_equipment() {
    let mut g = fixture();
    g.phase = Phase::Explore;
    g.players[0].hand = ids(&["rebar", "salt", "energy", "zombie", "portal"]);
    assert!(g.play(&id("rebar")).is_ok(), "снаряжение до исследования");
    for c in ["energy", "zombie", "portal"] {
        assert!(g.play(&id(c)).is_err(), "{c} нельзя до исследования");
    }
    g.trouble_deck = ids(&["zombie"]);
    assert!(g.explore().is_ok());
    assert!(
        g.hero_strength() == 3 && g.can_win(),
        "надетое в подготовку работает в первой встрече"
    );
}

#[test]
fn preparation_trade_and_shared_limits() {
    let mut g = fixture();
    g.phase = Phase::Explore;
    g.players[0].hand = ids(&["rebar", "stash"]);
    g.players[1].hand = ids(&["helmet"]);
    assert!(g.trade(&id("rebar")).is_ok());
    g.choose(Opt::Player(1)).unwrap();
    g.choose(Opt::card("helmet")).unwrap();
    assert!(g.players[0].holds(&id("stash")), "обмен в подготовку не тратит тайник");
    assert!(g.recycle(&id("stash")).is_ok(), "замена в подготовку");
    g.phase = Phase::Actions;
    assert!(
        g.trade(&id("helmet")).is_err() && g.recycle(&id("helmet")).is_err(),
        "лимиты общие на ход"
    );
}

#[test]
fn energy_skips_preparation_but_not_exploration() {
    let mut g = fixture();
    g.phase = Phase::Explore;
    g.players[0].skip_now = true;
    g.players[0].hand = ids(&["rebar"]);
    let rebar = id("rebar");
    assert!(g.play(&rebar).is_err() && g.trade(&rebar).is_err() && g.recycle(&rebar).is_err());
    assert!(g.explore().is_ok());
}

#[test]
fn final_response_lets_hero_save_the_battle() {
    let mut g = fixture();
    g.players[0].level = 3;
    g.players[0].hand = ids(&["brine", "salt", "bullet"]);
    g.players[1].hand = ids(&["noise"]);
    g.begin_battle(id("zombie"));
    g.pass_reaction().unwrap();
    g.play(&id("noise")).unwrap();
    g.pass_reaction().unwrap();
    assert!(g.final_response() && g.owner() == 0 && g.phase == Phase::Combat && !g.can_win());
    assert!(
        g.play(&id("brine")).is_ok() && g.can_win(),
        "финальный расходник спасает бой"
    );
    assert!(
        g.play(&id("salt")).is_ok() && g.play(&id("bullet")).is_err(),
        "лимит хаоса сохраняется"
    );
    assert!(g.pass_reaction().is_ok() && g.phase == Phase::Actions && g.players[0].level == 4);
}

#[test]
fn hero_chaos_not_repeated_in_final_response() {
    let mut g = fixture();
    g.players[0].hand = ids(&["salt", "rumor"]);
    g.begin_battle(id("zombie"));
    g.play(&id("salt")).unwrap();
    g.pass_reaction().unwrap();
    g.pass_reaction().unwrap();
    assert!(g.final_response() && g.play(&id("rumor")).is_err());
}

#[test]
fn clockwise_order_with_four_players() {
    let mut g = Game::new();
    g.start(["vityok", "zina", "stalker", "goblin"], 44).unwrap();
    g.active = 2;
    extract(&mut g, "zombie");
    g.begin_battle(id("zombie"));
    for expected in [2, 3, 0, 1] {
        assert_eq!(g.owner(), expected, "очередь по часовой");
        g.pass_reaction().unwrap();
    }
    assert!(g.owner() == 2 && g.final_response());
    assert!(conserved(&g));
}

#[test]
fn help_offer_consent_and_decline() {
    let mut g = valid_battle("zombie");
    set_hand(&mut g, 1, &["helmet"]);
    assert!(g.request_help().is_ok() && g.owner() == 0, "помощника выбирает герой");
    g.choose(Opt::Cancel).unwrap();
    assert!(!g.help_requested, "отмена не тратит предложение");
    g.request_help().unwrap();
    g.choose(Opt::Player(1)).unwrap();
    assert!(g.owner() == 1 && g.choices[0].kind == ChoiceKind::HelpAccept);
    let h = Game::from_save_json(&g.to_save_json()).unwrap();
    assert!(
        h.owner() == 1 && conserved(&h),
        "незавершённое согласие переживает сохранение"
    );
    g.choose(Opt::Decline).unwrap();
    assert!(g.players[1].hand == ids(&["helmet"]) && g.helper.is_none() && g.request_help().is_err());
}

#[test]
fn help_contract_victory() {
    let mut g = valid_battle("zombie");
    g.players[0].level = 5;
    set_hand(&mut g, 1, &["helmet", "friend", "noise"]);
    g.request_help().unwrap();
    g.choose(Opt::Player(1)).unwrap();
    g.choose(Opt::card("helmet")).unwrap();
    assert!(
        g.helper == Some(1)
            && g.discard.contains(&id("helmet"))
            && !g.players[1].witch_used
            && !g.chaos_used.contains(&1),
        "оплата помощи не тратит хаос и не включает защиту"
    );
    assert!(g.hero_strength() == 7 && conserved(&g));
    g.pass_reaction().unwrap();
    assert!(
        g.play(&id("friend")).is_err() && g.play(&id("noise")).is_ok(),
        "помощник может предать"
    );
    g.pass_reaction().unwrap();
    let reward = g.deck.last().cloned().unwrap();
    g.pass_reaction().unwrap();
    assert!(g.players[1].holds(&reward) && g.players[0].level == 6 && g.players[1].level == 1);
    assert!(conserved(&g));
}

#[test]
fn helper_avoids_defeat_penalty() {
    let mut g = valid_battle("elevator_boss");
    set_hand(&mut g, 1, &["helmet", "salt"]);
    g.request_help().unwrap();
    g.choose(Opt::Player(1)).unwrap();
    g.choose(Opt::card("helmet")).unwrap();
    complete_battle(&mut g);
    assert!(g.players[1].hand == ids(&["salt"]) && g.players[1].level == 1);
    assert!(conserved(&g));
}

#[test]
fn cancelled_contract_loses_bonus_and_loot() {
    let mut g = valid_battle("zombie");
    set_hand(&mut g, 1, &["helmet", "magic"]);
    g.request_help().unwrap();
    g.choose(Opt::Player(1)).unwrap();
    g.choose(Opt::card("helmet")).unwrap();
    g.pass_reaction().unwrap();
    assert!(g.play(&id("magic")).is_ok() && g.helper.is_none() && g.discard.contains(&id("helmet")));
    assert!(conserved(&g));
}

#[test]
fn help_must_precede_battle_cards() {
    let mut g = fixture();
    g.players[0].hand = ids(&["salt"]);
    g.begin_battle(id("zombie"));
    g.play(&id("salt")).unwrap();
    assert!(g.request_help().is_err());
}

#[test]
fn stash_keeps_one_and_buries_other() {
    let mut g = Game::new();
    g.start(["vityok", "zina"], 55).unwrap();
    g.active = 0;
    g.phase = Phase::Actions;
    set_hand(&mut g, 0, &["stash"]);
    extract(&mut g, "rebar");
    g.deck.push(id("rebar"));
    extract(&mut g, "helmet");
    g.deck.push(id("helmet"));
    assert!(g.play(&id("stash")).is_ok());
    assert_eq!(g.choices[0].options, vec![Opt::card("helmet"), Opt::card("rebar")]);
    assert!(conserved(&g), "карты тайника вне колоды учтены");
    let mut h = Game::from_save_json(&g.to_save_json()).unwrap();
    assert!(conserved(&h), "тайник переживает сохранение");
    h.choose(Opt::card("rebar")).unwrap();
    assert!(h.players[0].hand == ids(&["rebar"]) && h.deck[0] == id("helmet") && !h.discard.contains(&id("helmet")));
    assert!(conserved(&h));
}

#[test]
fn stash_with_small_deck() {
    let mut g = fixture();
    g.players[0].hand = ids(&["stash"]);
    g.deck.clear();
    g.discard.clear();
    assert!(g.play(&id("stash")).is_ok() && g.choices.is_empty() && g.discard == ids(&["stash"]));

    let mut g = fixture();
    g.players[0].hand = ids(&["stash"]);
    g.deck = ids(&["rebar"]);
    g.discard.clear();
    g.play(&id("stash")).unwrap();
    assert_eq!(g.choices[0].options, vec![Opt::card("rebar")]);
    g.choose(Opt::card("rebar")).unwrap();
    assert!(g.players[0].hand == ids(&["rebar"]) && g.deck.is_empty());
}

#[test]
fn cutlet_draws_two_then_discards_one() {
    let mut g = Game::new();
    g.start(["vityok", "zina"], 88).unwrap();
    g.active = 0;
    g.phase = Phase::Actions;
    set_hand(&mut g, 0, &["cutlet", "salt"]);
    assert!(g.play(&id("cutlet")).is_ok() && g.players[0].hand.len() == 3 && conserved(&g));
    g.choose(Opt::card("salt")).unwrap();
    assert!(g.players[0].hand.len() == 2 && conserved(&g));
}
