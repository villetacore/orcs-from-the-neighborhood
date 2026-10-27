//! Реальные границы новой экономики и развития, включая атомарность отказов.
mod common;
use common::*;
use orcs_core::{ChoiceKind, ChoiceOption as Opt, Game, Phase};

fn perk(g: &mut Game, who: usize, key: &str) {
    g.players[who].perks.push(id(&format!("perk_{key}")));
}
fn resolve_choices(g: &mut Game) {
    while let Some(q) = g.current_choice() {
        g.choose(q.options[0].clone()).unwrap();
    }
}

#[test]
fn separate_decks_and_market_are_conserved() {
    let mut g = Game::new();
    g.start(["rashid", "elvira", "boris", "aki"], 99).unwrap();
    assert!(g.trouble_deck.iter().all(|id| matches!(
        g.card(id).card_type,
        orcs_core::CardType::Enemy | orcs_core::CardType::Event
    )));
    assert!(g.players.iter().all(|p| p.hand.iter().all(|id| g.card(id).price > 0)));
    assert_eq!(g.market.len(), 3);
    assert!(conserved(&g));
}

#[test]
fn rejected_purchase_does_not_move_anything() {
    let mut g = fixture();
    g.players[0].coins = 0;
    let card = g.market[0].clone();
    let before = g.to_save_json();
    assert!(g.buy(&card, None).is_err());
    assert_eq!(before, g.to_save_json());
    assert!(g.buy(&id("missing"), None).is_err());
    assert_eq!(before, g.to_save_json());
}

#[test]
fn barter_is_one_operation_and_replenishes() {
    let mut g = valid_battle("zombie");
    g.phase = Phase::Explore;
    g.encounter = None;
    // Вернуть врага в правильную колоду после тестовой постановки боя.
    g.trouble_deck.push(id("zombie"));
    set_hand(&mut g, 0, &["helmet"]);
    g.players[0].coins = 20;
    let card = g.market[0].clone();
    let price = g.purchase_price(&card, true);
    g.buy(&card, Some(&id("helmet"))).unwrap();
    assert_eq!(g.players[0].shop_ops, 1);
    assert_eq!(g.players[0].coins, 20 - price);
    assert!(g.players[0].holds(&card));
    assert_eq!(g.market.len(), 3);
    assert!(conserved(&g));
}

#[test]
fn shop_limit_is_shared_between_preparation_and_actions() {
    let mut g = fixture();
    g.players[0].coins = 30;
    g.phase = Phase::Explore;
    let first = g.market[0].clone();
    g.buy(&first, None).unwrap();
    g.phase = Phase::Actions;
    let second = g.market[0].clone();
    g.buy(&second, None).unwrap();
    let before = g.to_save_json();
    let third = g.market[0].clone();
    assert!(g.buy(&third, None).is_err());
    assert_eq!(before, g.to_save_json());
}

#[test]
fn all_discounts_and_cashback_preserve_minimum_and_operation_cap() {
    let mut g = fixture();
    g.players[0].level = 9;
    g.players[0].coins = 100;
    for k in ["haggle", "cashback", "auction"] {
        perk(&mut g, 0, k);
    }
    g.district = some("district_market");
    extract(&mut g, "tape");
    g.market.insert(0, id("tape"));
    assert_eq!(g.purchase_price(&id("tape"), false), 1);
    g.buy(&id("tape"), None).unwrap();
    assert_eq!(g.players[0].coins, 99);
    g.sell(&id("tape")).unwrap();
    assert_eq!(g.players[0].coins, 100);
    let id = g.market[0].clone();
    g.refresh_market(&id).unwrap();
    assert!(g.shop_check().is_err());
}

#[test]
fn work_recovers_empty_wallet_without_combat_or_levels() {
    let mut g = fixture();
    g.players[0].coins = 0;
    g.phase = Phase::Explore;
    perk(&mut g, 0, "worker");
    g.players[0].level = 3;
    g.district = some("district_swamp");
    g.work().unwrap();
    assert_eq!(g.players[0].coins, 4);
    assert_eq!(g.players[0].level, 3);
    assert!(g.explore().is_err());
    g.players[0].hand.push(id("zombie"));
    assert!(g.play(&id("zombie")).is_err());
}

#[test]
fn paid_help_moves_money_only_after_consent_and_does_not_take_loot() {
    let mut g = valid_battle("zombie");
    set_hand(&mut g, 1, &["helmet"]);
    g.players[0].level = 2;
    let balances = (g.players[0].coins, g.players[1].coins);
    g.request_paid_help().unwrap();
    g.choose(Opt::Player(1)).unwrap();
    assert_eq!(balances, (g.players[0].coins, g.players[1].coins));
    let mut declined = g.clone();
    declined.choose(Opt::Decline).unwrap();
    assert_eq!(balances, (declined.players[0].coins, declined.players[1].coins));
    let mut g = Game::from_save_json(&g.to_save_json()).unwrap();
    g.choose(Opt::card("helmet")).unwrap();
    assert_eq!(
        (balances.0 - 2, balances.1 + 2),
        (g.players[0].coins, g.players[1].coins)
    );
    let reward = g.deck.last().unwrap().clone();
    complete_battle(&mut g);
    assert!(g.players[0].holds(&reward));
    assert!(!g.players[1].holds(&reward));
    assert!(conserved(&g));
}

#[test]
fn cancelled_paid_help_never_refunds_or_reopens_contract() {
    let mut g = valid_battle("zombie");
    set_hand(&mut g, 1, &["helmet", "magic"]);
    g.request_paid_help().unwrap();
    g.choose(Opt::Player(1)).unwrap();
    g.choose(Opt::card("helmet")).unwrap();
    let paid = g.players[0].coins;
    g.pass_reaction().unwrap();
    g.play(&id("magic")).unwrap();
    assert!(g.helper.is_none());
    assert_eq!(g.players[0].coins, paid);
    assert!(g.request_help().is_err());
}

#[test]
fn perks_unlock_once_and_suspend_after_level_loss() {
    let mut g = fixture();
    g.players[0].level = 2;
    g.players[0].hand = ids(&["energy"]);
    g.play(&id("energy")).unwrap();
    assert!(matches!(g.current_choice().unwrap().kind, ChoiceKind::Perk { tier: 3 }));
    g.choose(Opt::card("perk_solo")).unwrap();
    assert!(g.has_perk(0, "solo"));
    g.players[0].level = 2;
    assert!(!g.has_perk(0, "solo"));
    g.players[0].hand = ids(&["energy"]);
    g.play(&id("energy")).unwrap();
    assert!(g.choices.is_empty());
    assert_eq!(g.players[0].perks.len(), 1);
    assert!(g.has_perk(0, "solo"));
}

#[test]
fn perk_choices_are_unique_between_players_and_survive_save() {
    let mut g = fixture();
    g.players[0].level = 2;
    g.players[0].hand = ids(&["energy"]);
    g.play(&id("energy")).unwrap();
    let mut g = Game::from_save_json(&g.to_save_json()).unwrap();
    g.choose(Opt::card("perk_haggle")).unwrap();
    g.active = 1;
    g.players[1].level = 2;
    g.players[1].hand = ids(&["energy"]);
    g.play(&id("energy")).unwrap();
    assert!(!g.current_choice().unwrap().options.contains(&Opt::card("perk_haggle")));
}

#[test]
fn final_challenge_waits_for_next_own_turn_and_only_it_can_win() {
    let mut g = fixture();
    g.players[0].level = 8;
    g.players[0].unlocked_tiers = vec![3, 5, 7];
    g.begin_battle(id("zombie"));
    g.resolve_battle();
    assert_eq!(g.players[0].level, 9);
    assert!(g.challenge().is_err());
    resolve_choices(&mut g);
    g.end_turn().unwrap();
    g.phase = Phase::Actions;
    g.end_turn().unwrap();
    assert!(g.challenge_check().is_ok());
    g.players[0].weapon = some("club");
    g.players[0].armor = some("vest");
    g.challenge().unwrap();
    g.enemy_mod = -2;
    g.resolve_battle();
    resolve_choices(&mut g);
    assert_eq!(g.winner, Some(0));
    assert!(!g.trouble_discard.contains(&id("boss")));
}

#[test]
fn rounds_follow_initial_player_even_when_not_zero() {
    let mut g = fixture();
    g.active = 1;
    g.first_player = 1;
    let before = g.district.clone();
    g.end_turn().unwrap();
    assert_eq!(g.district, before);
    g.phase = Phase::Actions;
    g.end_turn().unwrap();
    assert_ne!(g.district, before);
}

#[test]
fn djinn_paid_boost_uses_shared_once_per_battle_limit() {
    let mut g = fixture();
    g.players[0].hero = id("rashid");
    g.begin_battle(id("zombie"));
    let before = g.hero_strength();
    g.paid_boost().unwrap();
    assert_eq!(g.hero_strength(), before + 2);
    assert_eq!(g.players[0].coins, 3);
    assert!(g.paid_boost().is_err());
}

#[test]
fn scouting_is_private_and_can_bury_exact_top_card() {
    let mut g = fixture();
    let top = g.trouble_deck.last().unwrap().clone();
    g.scout().unwrap();
    assert!(matches!(g.current_choice().unwrap().kind, ChoiceKind::Scout));
    assert!(!g.journal.last().unwrap().contains(&g.card(&top).name));
    let mut g = Game::from_save_json(&g.to_save_json()).unwrap();
    g.choose(Opt::Bottom).unwrap();
    assert_eq!(g.trouble_deck.first(), Some(&top));
}

#[test]
fn monetary_penalty_cannot_make_wallet_negative_and_insurance_applies() {
    let mut g = fixture();
    g.players[0].coins = 1;
    g.begin_battle(id("moth"));
    g.resolve_battle();
    assert_eq!(g.players[0].coins, 0);
    let mut g = fixture();
    g.players[0].level = 5;
    perk(&mut g, 0, "insurance");
    g.players[0].coins = 1;
    g.begin_battle(id("moth"));
    g.enemy_mod = 20;
    g.resolve_battle();
    assert_eq!(g.players[0].coins, 1);
}

#[test]
fn unknown_and_foreign_sales_reject_without_mutation() {
    let mut g = fixture();
    let before = g.to_save_json();
    assert!(g.sell(&id("missing")).is_err());
    assert!(g.sell(&id("boss")).is_err());
    assert_eq!(g.to_save_json(), before);
}
