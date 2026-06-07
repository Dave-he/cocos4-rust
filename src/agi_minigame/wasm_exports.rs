//! Round 48 — WASM bridge for the `scene_gen` POC slice.
//!
//! This module is compiled only when the `wasm-bindings` Cargo feature
//! is on. It exposes a JSON-in / JSON-out shim around the canonical
//! `theme_to_scene` so the AGI-miniGame TypeScript layer can call into
//! the Rust implementation without each TS mirror having to keep parity
//! by hand.
//!
//! Why JSON-bridge instead of `wasm-bindgen` structured bindings?
//! Round 48 keeps scope tight (PRD: M-L, 5-6h): one function, one shim,
//! the TS mirror lives on as a fallback when the WASM module fails to
//! load. Rounds 49/50/51 are slated to (1) extend coverage to
//! `build_generation_config_with_mood` and `mood_palette`, (2) switch
//! to `serde-wasm-bindgen` for typed bindings, and (3) drop the TS
//! mirror once WASM is mandatory.
//!
//! Test strategy: the JSON shim is pure Rust (no `wasm_bindgen!` macros
//! exercised at unit-test time), so the `#[cfg(test)]` block runs under
//! plain `cargo test --features wasm-bindings` with no wasm runtime.

#![cfg(feature = "wasm-bindings")]

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::scene_gen::{
    theme_to_scene, BiomeId, EventStep, MusicMood, NpcArchetype, SceneBlueprint, ThemeInput,
    VisualStyle,
};

// ---------------------------------------------------------------------------
// JSON shapes — string-tagged enums so the TS side stays human-readable
// and matches the existing `SceneGen.ts` literal types byte-for-byte.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ThemeInputJson {
    pub visual_style: String,
    pub music_mood: String,
    pub difficulty: f32,
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventStepJson {
    pub kind: String,
    pub delay_secs: u32,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SceneBlueprintJson {
    pub wfc_tile_weights: [u8; 8],
    pub biome_id: String,
    pub base_npc_density: f32,
    pub npc_density: f32,
    pub npc_count: u32,
    pub event_chain: Vec<EventStepJson>,
    pub music_bpm: u16,
    pub npc_archetype_hints: Vec<String>,
}

// ---------------------------------------------------------------------------
// Enum ↔ string mapping. Strings are canonical across Rust and TS — same
// lowercase tags used in `SceneGen.ts` literal types.
// ---------------------------------------------------------------------------

fn visual_style_from_str(s: &str) -> Result<VisualStyle, String> {
    match s {
        "cyberpunk" => Ok(VisualStyle::Cyberpunk),
        "fantasy" => Ok(VisualStyle::Fantasy),
        "space" => Ok(VisualStyle::Space),
        "underwater" => Ok(VisualStyle::Underwater),
        "desert" => Ok(VisualStyle::Desert),
        "dungeon" => Ok(VisualStyle::Dungeon),
        other => Err(format!("unknown visual_style: {}", other)),
    }
}

fn music_mood_from_str(s: &str) -> Result<MusicMood, String> {
    match s {
        "epic" => Ok(MusicMood::Epic),
        "mysterious" => Ok(MusicMood::Mysterious),
        "cheerful" => Ok(MusicMood::Cheerful),
        "tense" => Ok(MusicMood::Tense),
        "melancholic" => Ok(MusicMood::Melancholic),
        "pulse" => Ok(MusicMood::Pulse),
        other => Err(format!("unknown music_mood: {}", other)),
    }
}

fn biome_id_to_str(b: BiomeId) -> &'static str {
    match b {
        BiomeId::Cyberpunk => "cyberpunk",
        BiomeId::Forest => "forest",
        BiomeId::Desert => "desert",
        BiomeId::Ice => "ice",
        BiomeId::Space => "space",
        BiomeId::Dungeon => "dungeon",
    }
}

fn npc_archetype_to_str(a: NpcArchetype) -> &'static str {
    match a {
        NpcArchetype::Robot => "robot",
        NpcArchetype::Mage => "mage",
        NpcArchetype::Beast => "beast",
        NpcArchetype::Astronaut => "astronaut",
        NpcArchetype::Alien => "alien",
        NpcArchetype::Siren => "siren",
        NpcArchetype::Diver => "diver",
        NpcArchetype::Scorpion => "scorpion",
        NpcArchetype::Nomad => "nomad",
        NpcArchetype::Skeleton => "skeleton",
        NpcArchetype::Lich => "lich",
    }
}

// ---------------------------------------------------------------------------
// JSON ↔ native conversions. The blueprint side is one-way (Rust →
// JSON) because the WASM consumer reads only; the theme side is the
// other way around (JSON → Rust).
// ---------------------------------------------------------------------------

fn theme_input_from_json(j: ThemeInputJson) -> Result<ThemeInput, String> {
    Ok(ThemeInput {
        visual_style: visual_style_from_str(&j.visual_style)?,
        music_mood: music_mood_from_str(&j.music_mood)?,
        difficulty: j.difficulty,
        seed: j.seed,
    })
}

fn scene_blueprint_to_json(s: SceneBlueprint) -> SceneBlueprintJson {
    SceneBlueprintJson {
        wfc_tile_weights: s.wfc_tile_weights,
        biome_id: biome_id_to_str(s.biome_id).to_owned(),
        base_npc_density: s.base_npc_density,
        npc_density: s.npc_density,
        npc_count: s.npc_count,
        event_chain: s
            .event_chain
            .into_iter()
            .map(|e: EventStep| EventStepJson {
                kind: e.kind,
                delay_secs: e.delay_secs,
                payload: e.payload,
            })
            .collect(),
        music_bpm: s.music_bpm,
        npc_archetype_hints: s
            .npc_archetype_hints
            .into_iter()
            .map(|a| npc_archetype_to_str(a).to_owned())
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// Internal core — pure Rust, no `wasm_bindgen` runtime. Lets us unit-test
// the JSON bridge under plain `cargo test --features wasm-bindings`.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorJson {
    error: String,
}

pub(crate) fn theme_to_scene_json_internal(theme_json: &str) -> String {
    fn handle(theme_json: &str) -> Result<String, String> {
        let parsed: ThemeInputJson =
            serde_json::from_str(theme_json).map_err(|e| format!("parse: {}", e))?;
        let theme = theme_input_from_json(parsed)?;
        let blueprint = theme_to_scene(theme);
        let out = scene_blueprint_to_json(blueprint);
        serde_json::to_string(&out).map_err(|e| format!("serialize: {}", e))
    }

    match handle(theme_json) {
        Ok(s) => s,
        Err(msg) => serde_json::to_string(&ErrorJson { error: msg })
            .unwrap_or_else(|_| String::from(r#"{"error":"unknown"}"#)),
    }
}

// ---------------------------------------------------------------------------
// WASM exports — thin shims around the internal helpers.
// ---------------------------------------------------------------------------

/// Round 48 — canonical entry point for the AGI-miniGame TS layer.
///
/// `theme_json` must be a JSON object with the shape
/// `{ visual_style, music_mood, difficulty, seed }`. The return is
/// either a `SceneBlueprintJson` JSON object on success or
/// `{ "error": "..." }` on failure (parse / unknown enum / serialize).
/// This shim never panics — failures are always surfaced as JSON.
#[wasm_bindgen]
pub fn theme_to_scene_json(theme_json: &str) -> String {
    theme_to_scene_json_internal(theme_json)
}

/// Round 48 — health check. Returns the version stamp the AGI-miniGame
/// game layer uses to confirm the WASM module loaded the right build.
#[wasm_bindgen]
pub fn wasm_module_version() -> String {
    String::from("0.1.0-round48")
}

// ---------------------------------------------------------------------------
// Tests — pure Rust, no wasm runtime. Run with
// `cargo test --features wasm-bindings agi_minigame::wasm_exports`.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_input_json_roundtrip() {
        // The JSON shape is canonical — string tags for enums match
        // the SceneGen.ts literal types byte-for-byte.
        let j = ThemeInputJson {
            visual_style: "cyberpunk".to_owned(),
            music_mood: "pulse".to_owned(),
            difficulty: 0.5,
            seed: 42,
        };
        let native = theme_input_from_json(j.clone()).expect("parses");
        assert_eq!(native.difficulty, 0.5);
        assert_eq!(native.seed, 42);
        // Unknown tag → error.
        let bad = ThemeInputJson {
            visual_style: "alien_planet".to_owned(),
            music_mood: "pulse".to_owned(),
            difficulty: 0.5,
            seed: 1,
        };
        assert!(theme_input_from_json(bad).is_err());
    }

    #[test]
    fn scene_blueprint_to_json_keeps_canonical_fields() {
        let theme = ThemeInput {
            visual_style: VisualStyle::Cyberpunk,
            music_mood: MusicMood::Pulse,
            difficulty: 0.5,
            seed: 7,
        };
        let bp = theme_to_scene(theme);
        let j = scene_blueprint_to_json(bp.clone());
        assert_eq!(j.wfc_tile_weights, bp.wfc_tile_weights);
        assert_eq!(j.biome_id, "cyberpunk");
        assert_eq!(j.npc_count, bp.npc_count);
        assert_eq!(j.music_bpm, bp.music_bpm);
        assert_eq!(j.event_chain.len(), bp.event_chain.len());
        assert_eq!(j.npc_archetype_hints.len(), bp.npc_archetype_hints.len());
        assert_eq!(j.npc_archetype_hints[0], "robot");
    }

    #[test]
    fn theme_to_scene_json_returns_valid_scene_for_cyberpunk_pulse() {
        // The shim takes a JSON string and returns one — round-trip
        // both sides via serde_json so the test is independent of
        // wasm-bindgen runtime.
        let input = r#"{"visual_style":"cyberpunk","music_mood":"pulse","difficulty":0.5,"seed":1}"#;
        let out = theme_to_scene_json_internal(input);
        let parsed: SceneBlueprintJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("output must parse as SceneBlueprintJson: {}", out));
        assert_eq!(parsed.biome_id, "cyberpunk");
        assert!(parsed.npc_count > 0, "cyberpunk @0.5 should spawn NPCs");
        assert!(
            (60..=160).contains(&parsed.music_bpm),
            "BPM must be in [60, 160]"
        );
        assert!(
            (3..=5).contains(&parsed.event_chain.len()),
            "event chain length must be in [3, 5]"
        );
    }

    #[test]
    fn theme_to_scene_json_returns_error_on_bad_input() {
        // Malformed JSON → returns `{"error":"parse: ..."}` rather
        // than panicking. The TS layer reads `.error` and falls back
        // to the in-process mirror.
        let out = theme_to_scene_json_internal("not json at all");
        let err: ErrorJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("bad-input must serialize as ErrorJson: {}", out));
        assert!(err.error.starts_with("parse:"));
        // Unknown enum tag → also error.
        let out2 = theme_to_scene_json_internal(
            r#"{"visual_style":"alien_planet","music_mood":"pulse","difficulty":0.5,"seed":1}"#,
        );
        let err2: ErrorJson =
            serde_json::from_str(&out2).expect("unknown enum must serialize as ErrorJson");
        assert!(err2.error.contains("alien_planet"));
    }
}
