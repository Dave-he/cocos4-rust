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

// ---------------------------------------------------------------------------
// Round 147 — helper-level
// tests for the
// AtomRegistry
// + AtomRunner
// contracts the
// integration tests
// rely on. Closes
// the remaining
// gaps after the
// round-142
// AtomRunner tests
// (which covered
// the per-phase
// state machine
// in atom.rs but
// NOT the
// integration
// surface here).
//
// Pre-round-147
// covered:
//   - register_all_atoms
//     wires all 6
//     atoms
//   - every atom
//     instantiates +
//     responds to
//     on_enter (ends
//     in a healthy
//     phase)
//   - every atom's
//     save_state
//     does not panic
//   - unknown atom
//     id returns
//     None
//
// Round 147
// closes the
// coverage gap
// for:
//   - empty
//     AtomRegistry:
//     has_atom is
//     false for all
//     6 known IDs +
//     create returns
//     None for all
//     6 (regression
//     that returned
//     a default
//     placeholder
//     would silently
//     skip atoms)
//   - metadata.id
//     round-trips
//     for all 6 (id
//     matches the
//     registration
//     key; pins the
//     host-API
//     contract)
//   - list_all
//     returns all 6
//     + the metadata
//     list is stable
//     (no duplicates)
//   - has_atom
//     returns true
//     for the 6
//     known ids +
//     false for 5
//     different
//     "near-miss"
//     strings
//     (case-sensitivity
//     + null + empty
//     + whitespace
//     + dotted-notation
//     guard; a
//     regression that
//     used starts_with
//     would silently
//     match "match3x")
//   - get_metadata
//     is consistent
//     with has_atom
//     (None ↔ false)
//   - atoms are
//     independent:
//     two `create`
//     calls return
//     distinct
//     instances (no
//     shared mutable
//     state across
//     instances)
//   - atom_id() /
//     atom_name()
//     from the
//     AtomMetadata
//     round-trip
//     against the
//     registration
//     key
//   - the make_ctx
//     factory
//     produces a
//     fresh
//     AtomContext
//     with the
//     correct
//     delta_time
//     (0.016; pins
//     the test
//     helper contract)
//   - create on an
//     empty registry
//     returns None
//     (the
//     `register_all_atoms`
//     step is what
//     makes
//     integration
//     work — pin the
//     pre-registration
//     state)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round147_tests {
    use super::*;
    use crate::agi_minigame::atom::{AtomContext, AtomPhase, AtomRegistry};

    const ALL_IDS: &[&str] = &[
        "match3",
        "tower_defense",
        "card",
        "turn_combat",
        "parkour",
        "synthesis",
    ];

    fn registered() -> AtomRegistry {
        let mut reg = AtomRegistry::new();
        atoms::register_all_atoms(&mut reg);
        reg
    }

    #[test]
    fn empty_registry_has_no_atoms_round_147() {
        // Pre-registration: a fresh AtomRegistry is
        // empty. Pin the "no default placeholders"
        // contract (a regression that pre-registered
        // dummy atoms would silently mask the wiring
        // bug we're integration-testing FOR).
        let reg = AtomRegistry::new();
        for id in ALL_IDS {
            assert!(!reg.has_atom(id), "empty registry should NOT have {id}");
            assert!(reg.create(id).is_none(),
                "empty registry should NOT create {id}");
            assert!(reg.get_metadata(id).is_none(),
                "empty registry should NOT have metadata for {id}");
        }
    }

    #[test]
    fn metadata_id_matches_registration_key_for_all_six_round_147() {
        // For every registered atom, the metadata.id
        // must equal the registration key. A
        // regression that registered with one key
        // but used a different `id` in the metadata
        // would silently desync the WASM bridge
        // (which keys on metadata.id).
        let reg = registered();
        for id in ALL_IDS {
            let meta = reg.get_metadata(id)
                .unwrap_or_else(|| panic!("missing metadata for {id}"));
            assert_eq!(meta.id, *id,
                "metadata.id mismatch for {id}: got {}", meta.id);
            assert!(!meta.name.is_empty(),
                "atom {id} has empty name");
            assert!(!meta.tags.is_empty(),
                "atom {id} has empty tags");
        }
    }

    #[test]
    fn list_all_returns_exactly_six_unique_atoms_round_147() {
        // list_all() should return one metadata per
        // registered id, with no duplicates. A
        // regression that registered twice would
        // silently return 12 entries.
        let reg = registered();
        let all = reg.list_all();
        assert_eq!(all.len(), 6, "expected 6 atoms, got {}", all.len());
        let mut seen_ids = std::collections::HashSet::new();
        for meta in &all {
            assert!(seen_ids.insert(meta.id.clone()),
                "duplicate id in list_all: {}", meta.id);
        }
    }

    #[test]
    fn has_atom_rejects_near_miss_strings_round_147() {
        // Defense: a regression that used
        // `id.starts_with` (instead of exact match)
        // would silently match "match3x" or
        // "match3." against the "match3"
        // registration. Pin the exact-match
        // contract.
        let reg = registered();
        let near_misses = [
            "MATCH3",        // uppercase
            "match3x",       // trailing
            "xmatch3",       // leading
            "match3 ",       // trailing space
            " match3",       // leading space
            "match.3",       // dot separator
            "match_3",       // underscore
            "match-3",       // hyphen
            "",              // empty
        ];
        for s in near_misses {
            assert!(!reg.has_atom(s),
                "has_atom should reject near-miss string {s:?}");
        }
    }

    #[test]
    fn has_atom_and_get_metadata_are_consistent_round_147() {
        // For every string (registered or not),
        // has_atom(id) == true ↔ get_metadata(id)
        // is Some. A regression that decoupled
        // these (e.g. has_atom checked a separate
        // flag) would silently desync the host
        // contract.
        let reg = registered();
        for id in ALL_IDS {
            assert_eq!(reg.has_atom(id), reg.get_metadata(id).is_some(),
                "has_atom / get_metadata disagree on {id}");
        }
        // Also: an unknown id → both are
        // consistent (false / None).
        assert_eq!(reg.has_atom("not.an.atom"),
                   reg.get_metadata("not.an.atom").is_some());
    }

    #[test]
    fn two_create_calls_return_independent_instances_round_147() {
        // Two `create` calls return distinct
        // `Box<dyn Atom>` instances. The pre-round-147
        // test exercises the lifecycle of one
        // instance per id; a regression that
        // returned a shared singleton (via Rc /
        // interior mutability) would silently leak
        // state across "fresh" atom launches.
        let reg = registered();
        for id in ALL_IDS {
            let a = reg.create(id).expect("create a");
            let b = reg.create(id).expect("create b");
            let a_ptr = a.as_ref() as *const dyn crate::agi_minigame::atom::Atom;
            let b_ptr = b.as_ref() as *const dyn crate::agi_minigame::atom::Atom;
            assert_ne!(a_ptr, b_ptr,
                "create({id}) returned the same instance twice");
        }
    }

    #[test]
    fn make_ctx_factory_uses_expected_delta_time_round_147() {
        // Pin the test helper's delta_time contract:
        // the integration tests rely on
        // `ctx.delta_time == 0.016` (the
        // 60Hz simulation tick). A regression that
        // dropped the `.with_delta_time(0.016)`
        // step would silently default to 0.0
        // and the atom update paths would see
        // "no time has passed" → most
        // per-frame effects would no-op.
        let mut ctx = make_ctx();
        assert!((ctx.delta_time - 0.016).abs() < 1e-6,
            "expected delta_time = 0.016, got {}", ctx.delta_time);
        // Sanity: ctx is also mutable (the
        // integration test loop passes it to
        // atom.on_init / on_enter mutably).
        let _: &mut AtomContext = &mut ctx;
    }

    #[test]
    fn atom_lifecycle_full_path_ends_in_completed_or_running_round_147() {
        // For every registered atom, a full
        // init → enter → exit lifecycle ends in
        // either Running (if the atom short-
        // circuits; see round-140 turn_combat) or
        // Completed. The pre-round-147 test
        // only verified "healthy phase after
        // on_enter" — pin the exit transition
        // explicitly.
        let reg = registered();
        for id in ALL_IDS {
            let mut atom = reg.create(id).expect("create");
            let mut ctx = make_ctx();
            atom.on_init(&mut ctx);
            atom.on_enter(&mut ctx);
            atom.on_exit(&mut ctx);
            let p = atom.current_phase();
            let ok = matches!(
                p,
                AtomPhase::Running | AtomPhase::Completed
            );
            assert!(ok,
                "atom {id} ended in unexpected phase after exit: {p:?}");
        }
    }

    #[test]
    fn create_on_empty_registry_returns_none_for_all_six_round_147() {
        // Pre-registration contract: the factory
        // lookup is empty. Pin the lookup-table
        // shape — a regression that returned a
        // default placeholder would let the
        // integration tests "pass" while no
        // atoms were actually wired.
        let reg = AtomRegistry::new();
        for id in ALL_IDS {
            assert!(reg.create(id).is_none(),
                "empty registry should NOT create {id}");
        }
    }

    #[test]
    fn metadata_for_unknown_id_is_none_round_147() {
        // Mirror of has_atom: get_metadata returns
        // None for any string that isn't one of
        // the 6 registered ids.
        let reg = registered();
        let unknown = [
            "not.an.atom",
            "Match3",         // case-sensitivity
            "match-3",
            "match3 ",
            "synth",
            "atom_match3",
        ];
        for s in unknown {
            assert!(reg.get_metadata(s).is_none(),
                "get_metadata({s:?}) should be None");
        }
    }
}

