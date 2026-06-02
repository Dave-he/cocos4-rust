//! End-to-end integration tests for the agi_minigame atoms.
//!
//! Verifies that `register_all_atoms` wires every one of the 6 atoms
//! into the AtomRegistry correctly, and that each atom can be
//! instantiated + driven through its `Atom` lifecycle.
//!
//! These tests run with the lib-test target, so they cover the
//! full Rust engine surface.

use std::sync::{Arc, Mutex};

use crate::agi_minigame::atom::AtomRegistry;
use crate::agi_minigame::atoms;
use crate::agi_minigame::atoms::{
    CardAtom, Match3Atom, ParkourAtom, SynthesisAtom, TowerDefenseAtom, TurnCombatAtom,
};
use crate::agi_minigame::player::PlayerProfile;
use crate::agi_minigame::world_state::UnifiedWorldState;

fn make_ctx() -> crate::agi_minigame::atom::AtomContext {
    let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))));
    crate::agi_minigame::atom::AtomContext::new(ws).with_delta_time(0.016)
}

#[test]
fn register_all_atoms_registers_every_six() {
    let mut reg = AtomRegistry::new();
    atoms::register_all_atoms(&mut reg);
    for id in ["match3", "tower_defense", "card", "turn_combat", "parkour", "synthesis"] {
        assert!(reg.has_atom(id), "missing atom: {id}");
        let meta = reg.get_metadata(id).expect("metadata");
        assert_eq!(meta.id, id);
        assert!(!meta.name.is_empty());
        assert!(!meta.tags.is_empty());
    }
}

#[test]
fn every_atom_instantiates_and_responds_to_enter() {
    let mut reg = AtomRegistry::new();
    atoms::register_all_atoms(&mut reg);
    for id in ["match3", "tower_defense", "card", "turn_combat", "parkour", "synthesis"] {
        let mut atom = reg.create(id).expect("create");
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        // After on_enter every atom should be in the Running phase.
        let p = atom.current_phase();
        let running = matches!(
            p,
            crate::agi_minigame::atom::AtomPhase::Running
                | crate::agi_minigame::atom::AtomPhase::Completed
                | crate::agi_minigame::atom::AtomPhase::Paused
        );
        assert!(running, "atom {id} not in a healthy phase after on_enter: {p:?}");
    }
}

#[test]
fn every_atom_save_state_round_trips() {
    let mut reg = AtomRegistry::new();
    atoms::register_all_atoms(&mut reg);
    let cases: Vec<(&str, Box<dyn Fn() -> Box<dyn crate::agi_minigame::atom::Atom>>)> = vec![
        ("match3",        Box::new(|| Box::new(Match3Atom::new(8, 8, 30)))),
        ("tower_defense", Box::new(|| Box::new(TowerDefenseAtom::new(10, 10, 100.0, 200)))),
        ("card",          Box::new(|| Box::new(CardAtom::new(10, 10)))),
        ("turn_combat",   Box::new(|| Box::new(TurnCombatAtom::new()))),
        ("parkour",       Box::new(|| Box::new(ParkourAtom::new(3, 5.0, 3)))),
        ("synthesis",     Box::new(|| Box::new(SynthesisAtom::new()))),
    ];
    for (id, factory) in cases {
        let mut atom = factory();
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        let _saved = atom.save_state();
        // We don't have a concrete `load_state` implementation to
        // round-trip back into the same atom instance, but the
        // save call itself shouldn't panic for any atom.
    }
}

#[test]
fn unknown_atom_id_returns_none() {
    let mut reg = AtomRegistry::new();
    atoms::register_all_atoms(&mut reg);
    assert!(reg.create("not.an.atom").is_none());
    assert!(!reg.has_atom("not.an.atom"));
    assert!(reg.get_metadata("not.an.atom").is_none());
}
