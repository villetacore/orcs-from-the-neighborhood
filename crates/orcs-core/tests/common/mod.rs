#![allow(dead_code)]
//! Общие помощники для тестов правил.

use std::collections::HashSet;

use orcs_core::{CardId, CardType, Game, Phase};

pub fn id(s: &str) -> CardId {
    CardId::from(s)
}

pub fn ids(list: &[&str]) -> Vec<CardId> {
    list.iter().map(|s| id(s)).collect()
}

pub fn some(s: &str) -> Option<CardId> {
    Some(id(s))
}

/// Двое игроков, ход первого, фаза действий, пустые руки и слоты.
pub fn fixture() -> Game {
    let mut g = Game::new();
    g.start(["vityok", "zina"], 91).unwrap();
    g.active = 0;
    g.district = None;
    g.phase = Phase::Actions;
    g.choices.clear();
    for p in &mut g.players {
        p.hand.clear();
        p.weapon = None;
        p.armor = None;
    }
    g
}

/// Все 56 игровых карт на месте и ни одна не задвоена.
pub fn conserved(g: &Game) -> bool {
    let mut perk_ids = HashSet::new();
    if !g
        .players
        .iter()
        .all(|p| p.perks.len() <= 3 && p.perks.iter().all(|id| perk_ids.insert(id.clone())))
    {
        return false;
    }
    if let Some(current) = &g.district {
        let district_ids: HashSet<_> = g.districts.iter().chain(std::iter::once(current)).collect();
        if district_ids.len() != 6 || g.districts.len() != 5 {
            return false;
        }
    }
    let held = g.cards_in_play();
    if held.len() != 56 {
        return false;
    }
    let mut seen = HashSet::new();
    held.iter()
        .all(|c| g.cards.get(c).is_some_and(|d| d.card_type != CardType::Character) && seen.insert(c.clone()))
}

/// Убирает карту из всех мест, чтобы положить её куда нужно тесту.
pub fn extract(g: &mut Game, card: &str) {
    let card = id(card);
    g.deck.retain(|c| *c != card);
    g.discard.retain(|c| *c != card);
    g.trouble_deck.retain(|c| *c != card);
    g.trouble_discard.retain(|c| *c != card);
    g.market.retain(|c| *c != card);
    for p in &mut g.players {
        p.hand.retain(|c| *c != card);
        if p.weapon.as_ref() == Some(&card) {
            p.weapon = None;
        }
        if p.armor.as_ref() == Some(&card) {
            p.armor = None;
        }
    }
}

/// Заменяет руку, сохраняя все карты в игре (старая рука уходит в сброс).
pub fn set_hand(g: &mut Game, who: usize, cards: &[&str]) {
    let old = std::mem::take(&mut g.players[who].hand);
    g.discard.extend(old);
    for c in cards {
        extract(g, c);
        g.players[who].hand.push(id(c));
    }
}

pub fn valid_battle(enemy: &str) -> Game {
    let mut g = Game::new();
    g.start(["vityok", "zina"], 76).unwrap();
    g.active = 0;
    g.district = None;
    extract(&mut g, enemy);
    g.begin_battle(id(enemy));
    g
}

pub fn complete_battle(g: &mut Game) {
    for _ in 0..=g.players.len() {
        let _ = g.pass_reaction();
    }
}
