//! Полные автоматические партии на 2–4 игроков с проверкой сохранности карт после каждого действия.
//! Стратегия существует только для тестов и не является игровым ботом.

mod common;

use common::conserved;
use orcs_core::{CardType, Effect, Game, Phase};

const HEROES: [[&str; 4]; 3] = [
    ["vityok", "zina", "stalker", "goblin"],
    ["rashid", "elvira", "boris", "aki"],
    ["aki", "goblin", "rashid", "zina"],
];
const STEP_LIMIT: usize = 15_000;

fn better_gear(g: &Game, card: &orcs_core::CardDef) -> bool {
    let slot = card.slot.expect("у снаряжения есть слот");
    match g.players[g.active].gear(slot) {
        None => true,
        Some(worn) => g.card(worn).bonus < card.bonus,
    }
}

fn step(g: &mut Game) {
    if let Some(q) = g.current_choice() {
        let first = q.options[0].clone();
        g.choose(first).unwrap();
        return;
    }
    match g.phase {
        Phase::Explore => {
            let purchase = g
                .market
                .iter()
                .find(|id| {
                    let c = g.card(id);
                    g.buy_check(id, None).is_ok()
                        && g.players[g.active].purchases == 0
                        && c.card_type == CardType::Equipment
                        && better_gear(g, c)
                })
                .cloned();
            if let Some(id) = purchase {
                g.buy(&id, None).unwrap();
                return;
            }
            let hand = g.players[g.active].hand.clone();
            let equipped = hand.iter().any(|id| {
                let card = g.card(id).clone();
                card.card_type == CardType::Equipment && better_gear(g, &card) && g.play(id).is_ok()
            });
            if !equipped {
                if g.challenge_check().is_ok() {
                    g.challenge().unwrap();
                } else {
                    g.explore().unwrap();
                }
            }
        }
        Phase::Combat => {
            let p = g.owner();
            let mut did = !g.can_win() && g.help_check().is_ok() && g.request_help().is_ok();
            if !did {
                let hand = g.players[p].hand.clone();
                did = hand.iter().any(|id| {
                    let card = g.card(id);
                    let wanted = (p == g.active && card.card_type == CardType::Consumable)
                        || (card.card_type == CardType::Chaos
                            && matches!(
                                card.effect,
                                Effect::EnemyDown | Effect::HeroUp | Effect::Friend | Effect::Noise | Effect::EnemyUp
                            ));
                    wanted && g.play(id).is_ok()
                });
            }
            if !did && !g.can_win() && g.paid_boost_check().is_ok() {
                g.paid_boost().unwrap();
                did = true;
            }
            if !did {
                g.pass_reaction().unwrap();
            }
        }
        Phase::Actions => {
            let a = g.active;
            let hand = g.players[a].hand.clone();
            let played = hand.iter().any(|id| {
                let card = g.card(id).clone();
                let wanted = (card.card_type == CardType::Equipment && better_gear(g, &card))
                    || matches!(card.card_type, CardType::Enemy | CardType::Event)
                    || matches!(
                        card.effect,
                        Effect::Energy | Effect::Cycle | Effect::Ambush | Effect::Stash
                    );
                wanted && g.play(id).is_ok()
            });
            if played {
                return;
            }
            let player = &g.players[a];
            if player.hand.len() > g.hand_limit(a) {
                let first = player.hand[0].clone();
                g.discard_excess(&first).unwrap();
            } else if !player.skip_now && !player.recycled && !player.hand.is_empty() {
                let first = player.hand[0].clone();
                g.recycle(&first).unwrap();
            } else {
                g.end_turn().unwrap();
            }
        }
    }
}

fn simulate(count: usize, seed: u64) {
    let mut g = Game::new();
    g.start(HEROES[((seed - 1) / 4) as usize][..count].iter().copied(), seed)
        .unwrap();
    let mut steps = 0;
    while g.winner.is_none() && steps < STEP_LIMIT {
        steps += 1;
        step(&mut g);
        assert!(
            conserved(&g),
            "сохранность карт: {count} игроков, сид {seed}, шаг {steps}"
        );
    }
    assert!(
        g.winner.is_some(),
        "партия доходит до победы: {count} игроков, сид {seed}, {} ходов",
        g.turn
    );
    println!(
        "Партия: {count} игроков, сид {seed}: {} ходов, {steps} действий",
        g.turn
    );
}

#[test]
fn full_games_reach_victory() {
    for count in 2..=4 {
        for seed in 1..=12 {
            simulate(count, seed);
        }
    }
}
