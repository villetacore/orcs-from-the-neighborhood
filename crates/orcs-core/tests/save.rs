//! Сохранение и загрузка партии.

mod common;

use common::*;
use orcs_core::{ChoiceKind, Effect, Game, Phase, SAVE_VERSION, SaveError};

#[test]
fn pending_portal_survives_save() {
    let mut g = Game::new();
    g.start(["vityok", "zina"], 123).unwrap();
    g.phase = Phase::Actions;
    g.event_effect(Effect::Portal);
    let mut h = Game::from_save_json(&g.to_save_json()).unwrap();
    assert!(conserved(&h) && h.choices[0].kind == ChoiceKind::Portal);
    let first = h.choices[0].options[0].clone();
    assert!(h.choose(first).is_ok() && conserved(&h));
}

#[test]
fn final_response_survives_save() {
    let mut g = valid_battle("zombie");
    g.pass_reaction().unwrap();
    g.pass_reaction().unwrap();
    let h = Game::from_save_json(&g.to_save_json()).unwrap();
    assert!(h.final_response() && h.owner() == 0 && conserved(&h));
}

#[test]
fn restored_game_continues_identically() {
    let mut g = Game::new();
    g.start(["vityok", "zina", "stalker"], 9).unwrap();
    let mut h = Game::from_save_json(&g.to_save_json()).unwrap();
    for _ in 0..20 {
        assert_eq!(g.draw(), h.draw(), "ГПСЧ и колода восстановлены");
    }
}

#[test]
fn rejects_foreign_and_broken_saves() {
    assert!(matches!(
        Game::from_save_json(r#"{"version":2}"#),
        Err(SaveError::Version(2))
    ));
    assert!(Game::from_save_json(&format!(r#"{{"version":{SAVE_VERSION}}}"#)).is_err());
    assert!(Game::from_save_json("не json").is_err());
}
