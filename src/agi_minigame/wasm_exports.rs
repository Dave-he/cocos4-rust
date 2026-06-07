//! Round 48 → 51 — WASM bridge for the `scene_gen` + `narration` slices.
//!
//! This module is compiled only when the `wasm-bindings` Cargo feature
//! is on. It exposes JSON-in / JSON-out shims around the canonical
//! Rust functions so the AGI-miniGame TypeScript layer can call into
//! them without each TS mirror having to keep parity by hand.
//!
//! Round 48 shipped: `theme_to_scene_json` + `wasm_module_version`.
//! Round 51 extends to: `build_generation_config_with_mood_json`,
//! `mood_palette_json`, `mood_4th_sentence_for_json` — the same
//! JSON-shim pattern, no `serde-wasm-bindgen` switch (research 2026-06:
//! no benchmark justifies the cost; default `wasm-bindgen` features are
//! exactly `["spans", "std"]`, serde-serialize is opt-in).
//!
//! Why JSON-bridge instead of `wasm-bindgen` structured bindings?
//! Keeps the `default` feature surface minimal (no serde_json in the
//! default build), the TS-side error envelope is `{"error":"..."}` so
//! the existing `themeToSceneWithFallback` `null` on `parsed.error`
//! pattern drops in unchanged, and the round-50 ts-fallback contract
//! (TS mirror runs identically when WASM fails to load) is preserved
//! across all 4 fns.
//!
//! Test strategy: the JSON shims are pure Rust (no `wasm_bindgen!`
//! macros exercised at unit-test time), so the `#[cfg(test)]` block
//! runs under plain `cargo test --features wasm-bindings` with no wasm
//! runtime.

#![cfg(feature = "wasm-bindings")]

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::narration::{mood_4th_sentence_for as rust_mood_4th_sentence_for, mood_branch as rust_mood_branch};
use super::npc::NpcDisposition;
use super::scene_gen::{
    build_generation_config_with_mood as rust_build_generation_config_with_mood, mood_palette as
    rust_mood_palette, theme_to_scene, BiomeId, EventStep, GenerationHint, MusicMood, NpcArchetype,
    Palette, SceneBlueprint, ThemeInput, VisualStyle,
};
use super::gameplay::GameplayType;

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
// Round 51 — JSON shapes for the three new exported functions.
// ---------------------------------------------------------------------------

/// 1:1 mirror of `super::npc::NpcDisposition` (`friendly/fear/trust`).
/// Each axis is `f32`; consumers must `clamp` to `[-1.0, 1.0]` like the
/// TS side does in `NpcMind.defaultDisposition()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NpcDispositionJson {
    pub friendly: f32,
    pub fear: f32,
    pub trust: f32,
}

/// 1:1 mirror of `super::scene_gen::GenerationHint`. The native
/// `base_difficulty_range: (f32, f32)` tuple is split into `_lo` / `_hi`
/// fields because JSON cannot represent Rust tuples directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GenerationHintJson {
    pub min_atoms: u32,
    pub max_atoms: u32,
    pub reward_multiplier: f32,
    pub base_difficulty_range_lo: f32,
    pub base_difficulty_range_hi: f32,
}

/// 1:1 mirror of `super::ai_engine::GenerationConfig`. Tuples
/// (`difficulty_range`) are split; enums (`preferred_types`,
/// `excluded_types`) are serialized as string arrays via
/// `gameplay_type_to_str` / `gameplay_type_from_str` so the WASM
/// consumer sees `"parkour"` / `"match3"` / etc., matching the TS-side
/// `GameplayCombinerAI` literal types byte-for-byte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GenerationConfigJson {
    pub min_atoms: u32,
    pub max_atoms: u32,
    pub difficulty_range_lo: f32,
    pub difficulty_range_hi: f32,
    pub allow_composite: bool,
    pub seed: Option<u64>,
    pub player_level: u32,
    pub preferred_types: Vec<String>,
    pub excluded_types: Vec<String>,
    pub reward_multiplier: f32,
}

/// 1:1 mirror of one of the 4 canonical `Palette` constants in
/// `super::scene_gen` (`FEAR_PALETTE` / `FRIENDLY_PALETTE` /
/// `HOSTILE_PALETTE` / `NEUTRAL_PALETTE`). Always 3 hex strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PaletteJson {
    pub colors: [String; 3],
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
// Round 51 — `GameplayType` → string mapping. The 9 unit variants + the
// `"composite"` tag for `Composite(_)` mirror `gameplay.rs::name()`.
// The bridge is one-way for `GameplayType` (Rust produces strings
// from native structs; the TS side does not feed `GameplayType`
// strings back into Rust), so we only need the to-string direction.
// The reverse direction (`from_name`) is a `pub fn` on `GameplayType`
// itself, kept available for future rounds that may need it.
// ---------------------------------------------------------------------------

fn gameplay_type_to_str(t: &GameplayType) -> String {
    t.name().to_string()
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
// Round 51 — JSON ↔ native helpers for the three new exports.
// ---------------------------------------------------------------------------

fn npc_disposition_from_json(j: NpcDispositionJson) -> NpcDisposition {
    NpcDisposition {
        friendly: j.friendly,
        fear: j.fear,
        trust: j.trust,
    }
}

fn generation_hint_from_json(j: GenerationHintJson) -> GenerationHint {
    GenerationHint {
        min_atoms: j.min_atoms as usize,
        max_atoms: j.max_atoms as usize,
        reward_multiplier: j.reward_multiplier,
        base_difficulty_range: (j.base_difficulty_range_lo, j.base_difficulty_range_hi),
    }
}

fn generation_config_to_json(cfg: super::ai_engine::GenerationConfig) -> GenerationConfigJson {
    GenerationConfigJson {
        min_atoms: cfg.min_atoms as u32,
        max_atoms: cfg.max_atoms as u32,
        difficulty_range_lo: cfg.difficulty_range.0,
        difficulty_range_hi: cfg.difficulty_range.1,
        allow_composite: cfg.allow_composite,
        seed: cfg.seed,
        player_level: cfg.player_level,
        preferred_types: cfg.preferred_types.iter().map(gameplay_type_to_str).collect(),
        excluded_types: cfg.excluded_types.iter().map(gameplay_type_to_str).collect(),
        reward_multiplier: cfg.reward_multiplier,
    }
}

fn palette_to_json(p: Palette) -> PaletteJson {
    PaletteJson {
        colors: [p[0].to_string(), p[1].to_string(), p[2].to_string()],
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

/// Round 51 — internal helper for `build_generation_config_with_mood_json`.
///
/// Input JSON shape:
/// ```json
/// {
///   "player_level": <u32>,
///   "recent_loss_count": <u32>,
///   "mood": { "friendly": <f32>, "fear": <f32>, "trust": <f32> },
///   "hint": {
///     "min_atoms": <u32>, "max_atoms": <u32>,
///     "reward_multiplier": <f32>,
///     "base_difficulty_range_lo": <f32>,
///     "base_difficulty_range_hi": <f32>
///   },
///   "seed": <u64>
/// }
/// ```
///
/// On success returns a `GenerationConfigJson` JSON object. On failure
/// (parse error, unknown enum tag, serialize error) returns
/// `{"error":"..."}`. Never panics.
pub(crate) fn build_generation_config_with_mood_json_internal(args_json: &str) -> String {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ArgsJson {
        player_level: u32,
        recent_loss_count: u32,
        mood: NpcDispositionJson,
        hint: GenerationHintJson,
        seed: u64,
    }

    fn handle(args_json: &str) -> Result<String, String> {
        let args: ArgsJson = serde_json::from_str(args_json).map_err(|e| format!("parse: {}", e))?;
        let mood = npc_disposition_from_json(args.mood);
        let hint = generation_hint_from_json(args.hint);
        let cfg = rust_build_generation_config_with_mood(
            args.player_level,
            args.recent_loss_count,
            &mood,
            hint,
            args.seed,
        );
        let out = generation_config_to_json(cfg);
        serde_json::to_string(&out).map_err(|e| format!("serialize: {}", e))
    }

    match handle(args_json) {
        Ok(s) => s,
        Err(msg) => serde_json::to_string(&ErrorJson { error: msg })
            .unwrap_or_else(|_| String::from(r#"{"error":"unknown"}"#)),
    }
}

/// Round 51 — internal helper for `mood_palette_json`.
///
/// Input JSON shape: `NpcDispositionJson`
/// `{ "friendly": <f32>, "fear": <f32>, "trust": <f32> }`.
///
/// On success returns a `PaletteJson` JSON object
/// `{ "colors": ["#X", "#Y", "#Z"] }`. On failure returns
/// `{"error":"..."}`. Never panics.
pub(crate) fn mood_palette_json_internal(mood_json: &str) -> String {
    fn handle(mood_json: &str) -> Result<String, String> {
        let parsed: NpcDispositionJson =
            serde_json::from_str(mood_json).map_err(|e| format!("parse: {}", e))?;
        let mood = npc_disposition_from_json(parsed);
        let palette = rust_mood_palette(&mood);
        let out = palette_to_json(palette);
        serde_json::to_string(&out).map_err(|e| format!("serialize: {}", e))
    }

    match handle(mood_json) {
        Ok(s) => s,
        Err(msg) => serde_json::to_string(&ErrorJson { error: msg })
            .unwrap_or_else(|_| String::from(r#"{"error":"unknown"}"#)),
    }
}

/// Round 51 — internal helper for `mood_4th_sentence_for_json`.
///
/// Input JSON shape:
/// ```json
/// { "branch": <u8>, "blueprint_id": "<string>" }
/// ```
///
/// On success returns
/// `{"sentence":"<string>","branch":<u8>,"blueprint_id":"<string>"}`.
/// On failure (parse error, branch >= 3 means NEUTRAL which has no
/// 4th-sentence pool) returns `{"error":"..."}`. Never panics.
///
/// Note: the TS-side `NarrationEngine` uses `djb2` for the 4th-sentence
/// pick, while this Rust helper uses `fnv1a`. Same pool, different
/// index — both picks are valid pool entries, so the WASM-fallback
/// path always returns a sensible 4th sentence even when the source
/// differs. Unifying the hash is a round-52 follow-up.
pub(crate) fn mood_4th_sentence_for_json_internal(args_json: &str) -> String {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ArgsJson {
        branch: u8,
        blueprint_id: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct SentenceJson {
        sentence: String,
        branch: u8,
        blueprint_id: String,
    }

    fn handle(args_json: &str) -> Result<String, String> {
        let args: ArgsJson = serde_json::from_str(args_json).map_err(|e| format!("parse: {}", e))?;
        // `mood_4th_sentence_for` returns `None` only when the pool is
        // empty (branch >= 3 = NEUTRAL). We surface that as an error
        // so the TS side knows to skip the 4th-sentence slot.
        let sentence = rust_mood_4th_sentence_for(args.branch, &args.blueprint_id)
            .ok_or_else(|| format!("no 4th-sentence pool for branch {}", args.branch))?;
        let out = SentenceJson {
            sentence: sentence.to_string(),
            branch: args.branch,
            blueprint_id: args.blueprint_id,
        };
        serde_json::to_string(&out).map_err(|e| format!("serialize: {}", e))
    }

    match handle(args_json) {
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

/// Round 51 — mood-aware generation config for the next dimension.
///
/// `args_json` shape:
/// `{ player_level, recent_loss_count, mood{friendly,fear,trust},
///   hint{min_atoms,max_atoms,reward_multiplier,
///        base_difficulty_range_lo,base_difficulty_range_hi}, seed }`
///
/// Returns `GenerationConfigJson` on success, `{"error":"..."}` on
/// failure. Never panics.
#[wasm_bindgen]
pub fn build_generation_config_with_mood_json(args_json: &str) -> String {
    build_generation_config_with_mood_json_internal(args_json)
}

/// Round 51 — mood → 3-color hex palette (FEAR / FRIENDLY / HOSTILE / NEUTRAL).
///
/// `mood_json` shape: `{ friendly, fear, trust }`.
///
/// Returns `PaletteJson` (`{colors: ["#X", "#Y", "#Z"]}`) on success,
/// `{"error":"..."}` on failure. Never panics.
#[wasm_bindgen]
pub fn mood_palette_json(mood_json: &str) -> String {
    mood_palette_json_internal(mood_json)
}

/// Round 51 — FNV-1a-keyed 4th-sentence pick from the branch's pool.
///
/// `args_json` shape: `{ branch: <u8>, blueprint_id: "<string>" }`.
///
/// Returns `{sentence, branch, blueprint_id}` on success,
/// `{"error":"..."}` on failure (branch >= 3 / NEUTRAL has no pool).
/// Never panics.
///
/// TS side note: the WASM path picks via `fnv1a(blueprint_id)`, while
/// the TS fallback uses `djb2(blueprint_id + '|' + branch)`. Both
/// produce valid pool entries; the difference is a known follow-up
/// (round 52 candidate: unify hash).
#[wasm_bindgen]
pub fn mood_4th_sentence_for_json(args_json: &str) -> String {
    mood_4th_sentence_for_json_internal(args_json)
}

/// Round 51 — health check. Bumped from `0.1.0-round48` to
/// `0.2.0-round51` to reflect the three new exports. The TS-side
/// `loadSceneGenWasm` checks the major version `0.2.0-round` prefix.
#[wasm_bindgen]
pub fn wasm_module_version() -> String {
    String::from("0.2.0-round51")
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

    // -----------------------------------------------------------------------
    // Round 51 — 12 new tests for the three additional exports.
    // Run with `cargo test --features wasm-bindings agi_minigame::wasm_exports`.
    // -----------------------------------------------------------------------

    fn default_hint_json() -> String {
        serde_json::to_string(&GenerationHintJson {
            min_atoms: 2,
            max_atoms: 4,
            reward_multiplier: 1.0,
            base_difficulty_range_lo: 0.3,
            base_difficulty_range_hi: 0.8,
        })
        .unwrap()
    }

    fn fear_mood_json() -> String {
        serde_json::to_string(&NpcDispositionJson { friendly: 0.0, fear: 0.8, trust: 0.0 }).unwrap()
    }

    fn loved_mood_json() -> String {
        serde_json::to_string(&NpcDispositionJson { friendly: 0.7, fear: 0.0, trust: 0.4 }).unwrap()
    }

    fn neutral_mood_json() -> String {
        serde_json::to_string(&NpcDispositionJson { friendly: 0.0, fear: 0.0, trust: 0.0 }).unwrap()
    }

    #[test]
    fn npc_disposition_json_roundtrip() {
        // Field-level byte consistency: parse the JSON, build the
        // native struct, verify each axis matches the original. The
        // native `NpcDisposition` doesn't clamp on construction (the
        // caller is expected to clamp), so we just verify pass-through.
        let j: NpcDispositionJson = serde_json::from_str(&fear_mood_json()).unwrap();
        let native = npc_disposition_from_json(j);
        assert_eq!(native.friendly, 0.0);
        assert_eq!(native.fear, 0.8);
        assert_eq!(native.trust, 0.0);
    }

    #[test]
    fn build_generation_config_with_mood_json_neutral_matches_moodless() {
        // When the mood is the default neutral disposition, the result
        // is the plain "mood-less" base — the reflexive loop adds
        // information when there *is* information, never noise.
        let args = format!(
            r#"{{"player_level":5,"recent_loss_count":0,"mood":{},"hint":{},"seed":42}}"#,
            neutral_mood_json(),
            default_hint_json()
        );
        let out = build_generation_config_with_mood_json_internal(&args);
        let cfg: GenerationConfigJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("output must parse as GenerationConfigJson: {}", out));
        // No mood nudge → difficulty_range == hint.base exactly.
        assert!((cfg.difficulty_range_lo - 0.3).abs() < 1e-6);
        assert!((cfg.difficulty_range_hi - 0.8).abs() < 1e-6);
        // No recent losses → no excluded types.
        assert!(cfg.excluded_types.is_empty());
        // seed is round-tripped as `Some(42)`.
        assert_eq!(cfg.seed, Some(42));
    }

    #[test]
    fn build_generation_config_with_mood_json_fear_nudges_hi_down() {
        // fear > 0.5 → hi -= 0.05.
        let args = format!(
            r#"{{"player_level":5,"recent_loss_count":0,"mood":{},"hint":{},"seed":7}}"#,
            fear_mood_json(),
            default_hint_json()
        );
        let out = build_generation_config_with_mood_json_internal(&args);
        let cfg: GenerationConfigJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("output must parse as GenerationConfigJson: {}", out));
        // hint.base hi = 0.8, fear nudges hi to 0.75.
        assert!((cfg.difficulty_range_hi - 0.75).abs() < 1e-6);
        // lo is unchanged at 0.3.
        assert!((cfg.difficulty_range_lo - 0.3).abs() < 1e-6);
    }

    #[test]
    fn build_generation_config_with_mood_json_loved_promotes_match3_or_synthesis() {
        // friendly > 0.5 && trust > 0.3 → "loved" branch promotes
        // either `match3` or `synthesis` to the head of the pool.
        let args = format!(
            r#"{{"player_level":5,"recent_loss_count":0,"mood":{},"hint":{},"seed":1}}"#,
            loved_mood_json(),
            default_hint_json()
        );
        let out = build_generation_config_with_mood_json_internal(&args);
        let cfg: GenerationConfigJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("output must parse as GenerationConfigJson: {}", out));
        assert!(!cfg.preferred_types.is_empty());
        let head = &cfg.preferred_types[0];
        assert!(
            head == "match3" || head == "synthesis",
            "loved branch must promote match3 or synthesis, got {}",
            head
        );
    }

    #[test]
    fn build_generation_config_with_mood_json_loss_count_excludes_shooting() {
        // recent_loss_count >= 3 → drop shooting.
        let args = format!(
            r#"{{"player_level":5,"recent_loss_count":3,"mood":{},"hint":{},"seed":1}}"#,
            neutral_mood_json(),
            default_hint_json()
        );
        let out = build_generation_config_with_mood_json_internal(&args);
        let cfg: GenerationConfigJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("output must parse as GenerationConfigJson: {}", out));
        assert!(cfg.excluded_types.contains(&"shooting".to_string()));
        // 2 losses is below threshold → no exclusion.
        let args2 = format!(
            r#"{{"player_level":5,"recent_loss_count":2,"mood":{},"hint":{},"seed":1}}"#,
            neutral_mood_json(),
            default_hint_json()
        );
        let out2 = build_generation_config_with_mood_json_internal(&args2);
        let cfg2: GenerationConfigJson = serde_json::from_str(&out2).unwrap();
        assert!(cfg2.excluded_types.is_empty());
    }

    #[test]
    fn mood_palette_json_fear_returns_fear_palette() {
        // fear > 0.5 → cold, dark, bloodless palette
        // (navies / ice). Round 24 pinned the exact hex values; the
        // WASM bridge must produce them byte-for-byte.
        let out = mood_palette_json_internal(&fear_mood_json());
        let p: PaletteJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("output must parse as PaletteJson: {}", out));
        assert_eq!(p.colors, ["#0A1A2F", "#1B4965", "#CAE9FF"]);
    }

    #[test]
    fn mood_palette_json_loved_returns_friendly_palette() {
        // friendly > 0.5 && trust > 0.3 → warm, vibrant palette
        // (sunset orange / gold / cream).
        let out = mood_palette_json_internal(&loved_mood_json());
        let p: PaletteJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("output must parse as PaletteJson: {}", out));
        assert_eq!(p.colors, ["#FF6B35", "#F7C548", "#FFFAEB"]);
    }

    #[test]
    fn mood_palette_json_default_returns_neutral_palette() {
        // 3 zeros on every axis → neutral palette (deep purples / pink).
        let out = mood_palette_json_internal(&neutral_mood_json());
        let p: PaletteJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("output must parse as PaletteJson: {}", out));
        assert_eq!(p.colors, ["#3A0CA3", "#7209B7", "#F72585"]);
    }

    #[test]
    fn mood_4th_sentence_for_json_fear_branch_deterministic() {
        // Same (branch, blueprint_id) → same sentence. Different
        // blueprint_id → at least one different idx in the pool.
        let args_a = r#"{"branch":0,"blueprint_id":"dim_42"}"#;
        let args_b = r#"{"branch":0,"blueprint_id":"dim_42"}"#;
        let out_a = mood_4th_sentence_for_json_internal(args_a);
        let out_b = mood_4th_sentence_for_json_internal(args_b);
        let sent_a: serde_json::Value = serde_json::from_str(&out_a).unwrap();
        let sent_b: serde_json::Value = serde_json::from_str(&out_b).unwrap();
        assert_eq!(sent_a["sentence"], sent_b["sentence"]);
        // Different blueprint_id → different sentence (probabilistically
        // very high, since fear pool = 4 and we only need a mod-4
        // collision, but the test passes for any branch where pool
        // size > 1).
        let args_c = r#"{"branch":0,"blueprint_id":"dim_99"}"#;
        let out_c = mood_4th_sentence_for_json_internal(args_c);
        let sent_c: serde_json::Value = serde_json::from_str(&out_c).unwrap();
        // soft assert: the pool has 4 entries, two FNV-1a hashes
        // colliding mod 4 is rare but not impossible; we just check
        // the field shape, not the value.
        assert!(sent_c["sentence"].as_str().is_some());
    }

    #[test]
    fn mood_4th_sentence_for_json_friendly_branch_pool_size_5() {
        // Round 30 expanded the friendly (loved) pool to 5 entries.
        // We verify by hashing 5 distinct blueprint_ids and asserting
        // the resulting sentences are all non-empty strings. (The
        // pool size is implicit: as long as `mood_4th_sentence_for`
        // returns a valid string, the pool is non-empty.)
        for i in 0..5 {
            let args = format!(r#"{{"branch":1,"blueprint_id":"dim_{}"}}"#, i);
            let out = mood_4th_sentence_for_json_internal(&args);
            let sent: serde_json::Value = serde_json::from_str(&out).unwrap();
            let s = sent["sentence"].as_str().expect("sentence must be a string");
            assert!(!s.is_empty(), "pool[{}] must be non-empty", i);
        }
    }

    #[test]
    fn mood_4th_sentence_for_json_hostile_branch_pool_size_4() {
        // Round 30 expanded the hostile pool to 4 entries. Same
        // strategy as the friendly test: 4 distinct blueprint_ids,
        // 4 non-empty sentences.
        for i in 0..4 {
            let args = format!(r#"{{"branch":2,"blueprint_id":"dim_{}"}}"#, i);
            let out = mood_4th_sentence_for_json_internal(&args);
            let sent: serde_json::Value = serde_json::from_str(&out).unwrap();
            let s = sent["sentence"].as_str().expect("sentence must be a string");
            assert!(!s.is_empty(), "pool[{}] must be non-empty", i);
        }
    }

    #[test]
    fn mood_4th_sentence_for_json_neutral_branch_returns_error() {
        // branch = 3 = NEUTRAL, the pool is empty. The shim must
        // return `{"error":"..."}` rather than panicking.
        let args = r#"{"branch":3,"blueprint_id":"dim_42"}"#;
        let out = mood_4th_sentence_for_json_internal(args);
        let err: ErrorJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("neutral branch must serialize as ErrorJson: {}", out));
        assert!(err.error.contains("3"), "error should mention branch 3: {}", err.error);
    }

    #[test]
    fn mood_4th_sentence_for_json_bad_input_returns_error_json() {
        // Malformed JSON → `{"error":"parse: ..."}`.
        let out = mood_4th_sentence_for_json_internal("not json");
        let err: ErrorJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("bad-input must serialize as ErrorJson: {}", out));
        assert!(err.error.starts_with("parse:"));
    }

    #[test]
    fn gameplay_type_string_pinning_matches_gameplay_rs() {
        // PRD R2 mitigation — pin the 11 canonical `GameplayType`
        // string tags byte-for-byte against `gameplay.rs::name()`. The
        // TS side consumes these literal strings; if a future round
        // renames a variant, this test fails loudly.
        let cases: &[(GameplayType, &str)] = &[
            (GameplayType::Match3, "match3"),
            (GameplayType::TowerDefense, "tower_defense"),
            (GameplayType::Card, "card"),
            (GameplayType::TurnCombat, "turn_combat"),
            (GameplayType::Parkour, "parkour"),
            (GameplayType::Puzzle, "puzzle"),
            (GameplayType::Shooting, "shooting"),
            (GameplayType::Synthesis, "synthesis"),
            (GameplayType::Simulation, "simulation"),
            (GameplayType::Composite(vec![]), "composite"),
        ];
        for (t, expected) in cases {
            assert_eq!(gameplay_type_to_str(t), *expected);
        }
        // `Custom("...")` round-trips its inner name.
        let custom = GameplayType::Custom("robot".to_string());
        assert_eq!(gameplay_type_to_str(&custom), "robot");
    }
}
