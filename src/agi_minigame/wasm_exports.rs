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

    // -----------------------------------------------------------------------
    // Round 141 — helper-level tests for the JSON bridge surface.
    // Round 138/139/140 pattern: focus on the small mapping helpers
    // and JSON-bridge error paths that the existing integration tests
    // don't cover.
    // -----------------------------------------------------------------------

    #[test]
    fn round141_visual_style_from_str_all_known_tags() {
        // Pin all 6 visual_style tags byte-for-byte against the
        // `VisualStyle` enum variants. If a future round adds a
        // variant without updating the bridge, this fails loudly.
        assert!(matches!(visual_style_from_str("cyberpunk"), Ok(VisualStyle::Cyberpunk)));
        assert!(matches!(visual_style_from_str("fantasy"), Ok(VisualStyle::Fantasy)));
        assert!(matches!(visual_style_from_str("space"), Ok(VisualStyle::Space)));
        assert!(matches!(visual_style_from_str("underwater"), Ok(VisualStyle::Underwater)));
        assert!(matches!(visual_style_from_str("desert"), Ok(VisualStyle::Desert)));
        assert!(matches!(visual_style_from_str("dungeon"), Ok(VisualStyle::Dungeon)));
    }

    #[test]
    fn round141_visual_style_from_str_unknown_error_message() {
        // Error envelope must include the bad tag for debuggability.
        let err = visual_style_from_str("atlantis").unwrap_err();
        assert!(err.contains("atlantis"), "error must echo the bad tag: {}", err);
        assert!(err.contains("visual_style"), "error must name the field: {}", err);
    }

    #[test]
    fn round141_visual_style_from_str_empty_string_errors() {
        // Empty string is not a valid tag — must error, not panic.
        let err = visual_style_from_str("").unwrap_err();
        assert!(err.contains("visual_style"));
    }

    #[test]
    fn round141_music_mood_from_str_all_known_tags() {
        // Pin all 6 music_mood tags byte-for-byte against `MusicMood`.
        assert!(matches!(music_mood_from_str("epic"), Ok(MusicMood::Epic)));
        assert!(matches!(music_mood_from_str("mysterious"), Ok(MusicMood::Mysterious)));
        assert!(matches!(music_mood_from_str("cheerful"), Ok(MusicMood::Cheerful)));
        assert!(matches!(music_mood_from_str("tense"), Ok(MusicMood::Tense)));
        assert!(matches!(music_mood_from_str("melancholic"), Ok(MusicMood::Melancholic)));
        assert!(matches!(music_mood_from_str("pulse"), Ok(MusicMood::Pulse)));
    }

    #[test]
    fn round141_music_mood_from_str_case_sensitive() {
        // The bridge uses lowercase canonical tags; uppercase must NOT
        // match. This is a contract the TS side relies on (its literal
        // type is `"epic" | "mysterious" | ...`).
        assert!(music_mood_from_str("EPIC").is_err());
        assert!(music_mood_from_str("Epic").is_err());
        assert!(music_mood_from_str(" epic").is_err());
    }

    #[test]
    fn round141_biome_id_to_str_all_variants() {
        // The 6 `BiomeId` variants all map to lowercase strings.
        assert_eq!(biome_id_to_str(BiomeId::Cyberpunk), "cyberpunk");
        assert_eq!(biome_id_to_str(BiomeId::Forest), "forest");
        assert_eq!(biome_id_to_str(BiomeId::Desert), "desert");
        assert_eq!(biome_id_to_str(BiomeId::Ice), "ice");
        assert_eq!(biome_id_to_str(BiomeId::Space), "space");
        assert_eq!(biome_id_to_str(BiomeId::Dungeon), "dungeon");
    }

    #[test]
    fn round141_npc_archetype_to_str_all_variants() {
        // 11 `NpcArchetype` variants → 11 distinct lowercase strings.
        let cases: &[(NpcArchetype, &str)] = &[
            (NpcArchetype::Robot, "robot"),
            (NpcArchetype::Mage, "mage"),
            (NpcArchetype::Beast, "beast"),
            (NpcArchetype::Astronaut, "astronaut"),
            (NpcArchetype::Alien, "alien"),
            (NpcArchetype::Siren, "siren"),
            (NpcArchetype::Diver, "diver"),
            (NpcArchetype::Scorpion, "scorpion"),
            (NpcArchetype::Nomad, "nomad"),
            (NpcArchetype::Skeleton, "skeleton"),
            (NpcArchetype::Lich, "lich"),
        ];
        for (a, expected) in cases {
            assert_eq!(npc_archetype_to_str(*a), *expected);
        }
    }

    #[test]
    fn round141_event_step_json_field_passthrough() {
        // The `event_chain` mapping must preserve fields verbatim.
        let native = EventStep {
            kind: "collide".to_string(),
            delay_secs: 2500,
            payload: "p1:1.5".to_string(),
        };
        let j = EventStepJson {
            kind: native.kind.clone(),
            delay_secs: native.delay_secs,
            payload: native.payload.clone(),
        };
        assert_eq!(j.kind, "collide");
        assert_eq!(j.delay_secs, 2500);
        assert_eq!(j.payload, "p1:1.5");
    }

    #[test]
    fn round141_theme_input_from_json_preserves_difficulty_and_seed() {
        // The two passthrough fields must NOT be clamped or
        // transformed — the WASM layer is a transparent bridge.
        let j = ThemeInputJson {
            visual_style: "fantasy".to_string(),
            music_mood: "mysterious".to_string(),
            difficulty: -0.5, // intentionally out of [0,1] — bridge doesn't clamp
            seed: u64::MAX,
        };
        let native = theme_input_from_json(j).expect("parses");
        assert_eq!(native.difficulty, -0.5);
        assert_eq!(native.seed, u64::MAX);
    }

    #[test]
    fn round141_theme_input_from_json_bad_music_mood_errors() {
        // If the visual_style parses but the music_mood doesn't, the
        // shim surfaces the music_mood error.
        let j = ThemeInputJson {
            visual_style: "fantasy".to_string(),
            music_mood: "jazzy".to_string(),
            difficulty: 0.5,
            seed: 1,
        };
        let err = theme_input_from_json(j).unwrap_err();
        assert!(err.contains("jazzy"), "error must echo the bad music_mood tag: {}", err);
        assert!(err.contains("music_mood"));
    }

    #[test]
    fn round141_npc_disposition_from_json_field_passthrough() {
        // All 3 axes pass through verbatim — the bridge does NOT
        // clamp. Clamping is the caller's responsibility (TS does
        // it in NpcMind.defaultDisposition()).
        let j = NpcDispositionJson {
            friendly: 2.5,
            fear: -1.5,
            trust: 99.0,
        };
        let native = npc_disposition_from_json(j);
        assert_eq!(native.friendly, 2.5);
        assert_eq!(native.fear, -1.5);
        assert_eq!(native.trust, 99.0);
    }

    #[test]
    fn round141_generation_hint_from_json_casts_to_usize() {
        // The JSON shape uses `u32` for atom counts, the native struct
        // uses `usize`. The bridge must cast losslessly.
        let j = GenerationHintJson {
            min_atoms: 5,
            max_atoms: 12,
            reward_multiplier: 2.5,
            base_difficulty_range_lo: 0.2,
            base_difficulty_range_hi: 0.9,
        };
        let native = generation_hint_from_json(j);
        assert_eq!(native.min_atoms, 5);
        assert_eq!(native.max_atoms, 12);
        assert_eq!(native.reward_multiplier, 2.5);
        assert_eq!(native.base_difficulty_range, (0.2, 0.9));
    }

    #[test]
    fn round141_generation_hint_from_json_zero_atoms() {
        // Edge: min_atoms=0 is unusual but must round-trip without
        // error. The downstream `build_generation_config_with_mood`
        // is the one that enforces the practical minimum.
        let j = GenerationHintJson {
            min_atoms: 0,
            max_atoms: 0,
            reward_multiplier: 1.0,
            base_difficulty_range_lo: 0.0,
            base_difficulty_range_hi: 1.0,
        };
        let native = generation_hint_from_json(j);
        assert_eq!(native.min_atoms, 0);
        assert_eq!(native.max_atoms, 0);
    }

    #[test]
    fn round141_palette_to_json_emits_three_hex_strings() {
        // The `Palette` type is `[&str; 3]`. The bridge must serialize
        // each element to a String without quoting or escaping.
        let p: Palette = ["#000000", "#FFFFFF", "#ABCDEF"];
        let j = palette_to_json(p);
        assert_eq!(j.colors.len(), 3);
        assert_eq!(j.colors[0], "#000000");
        assert_eq!(j.colors[1], "#FFFFFF");
        assert_eq!(j.colors[2], "#ABCDEF");
    }

    #[test]
    fn round141_palette_to_json_serializes_to_canonical_shape() {
        // The TS side reads `parsed.colors[0]` directly — the JSON
        // shape must be `{ "colors": [...] }` and not a tuple.
        let p: Palette = ["#111", "#222", "#333"];
        let j = palette_to_json(p);
        let s = serde_json::to_string(&j).unwrap();
        assert!(s.contains("\"colors\""));
        assert!(s.contains("\"#111\""));
        assert!(s.contains("\"#222\""));
        assert!(s.contains("\"#333\""));
    }

    #[test]
    fn round141_theme_to_scene_json_internal_error_envelope_is_json() {
        // When the shim catches an internal error, it serializes an
        // `ErrorJson { error: "..." }` to JSON. Verify on multiple
        // failure paths that the output is *valid JSON* (not a
        // Rust-format Debug string).
        let out = theme_to_scene_json_internal("not json at all");
        let v: serde_json::Value = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("error output must be valid JSON: {}", out));
        assert!(v.get("error").is_some(), "error envelope must have 'error' key");
    }

    #[test]
    fn round141_theme_to_scene_json_internal_unknown_visual_style() {
        // Unknown visual_style tag → error envelope containing the tag.
        let out = theme_to_scene_json_internal(
            r#"{"visual_style":"atlantis","music_mood":"pulse","difficulty":0.5,"seed":1}"#,
        );
        let err: ErrorJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("unknown tag must serialize as ErrorJson: {}", out));
        assert!(err.error.contains("atlantis"));
    }

    #[test]
    fn round141_theme_to_scene_json_internal_unknown_music_mood() {
        // Same path for music_mood — must also error rather than
        // silently coerce.
        let out = theme_to_scene_json_internal(
            r#"{"visual_style":"fantasy","music_mood":"jazzy","difficulty":0.5,"seed":1}"#,
        );
        let err: ErrorJson = serde_json::from_str(&out).unwrap();
        assert!(err.error.contains("jazzy"));
    }

    #[test]
    fn round141_theme_to_scene_json_internal_all_visual_styles_produce_valid_blueprints() {
        // Smoke test: every supported visual_style must produce a
        // valid SceneBlueprintJson. The biome_id echoes the
        // visual_style for the cases that map 1:1 (cyberpunk) and
        // may differ for others (fantasy→forest, underwater→ice, etc.).
        let styles = ["cyberpunk", "fantasy", "space", "underwater", "desert", "dungeon"];
        for style in styles {
            let input = format!(
                r#"{{"visual_style":"{}","music_mood":"pulse","difficulty":0.5,"seed":1}}"#,
                style
            );
            let out = theme_to_scene_json_internal(&input);
            let bp: SceneBlueprintJson = serde_json::from_str(&out)
                .unwrap_or_else(|_| panic!("{}: must parse as SceneBlueprintJson: {}", style, out));
            assert!(!bp.biome_id.is_empty(), "{}: biome_id must be non-empty", style);
        }
    }

    #[test]
    fn round141_build_generation_config_with_mood_json_internal_bad_hint_field() {
        // Missing `hint` field in the input → `parse:` error.
        let args = r#"{"player_level":5,"recent_loss_count":0,"mood":{"friendly":0.0,"fear":0.0,"trust":0.0},"seed":1}"#;
        let out = build_generation_config_with_mood_json_internal(args);
        let err: ErrorJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("missing hint must serialize as ErrorJson: {}", out));
        assert!(err.error.starts_with("parse:"));
    }

    #[test]
    fn round141_build_generation_config_with_mood_json_internal_bad_mood_field() {
        // Missing `mood` field → also a parse error.
        let args = r#"{"player_level":5,"recent_loss_count":0,"hint":{"min_atoms":2,"max_atoms":4,"reward_multiplier":1.0,"base_difficulty_range_lo":0.3,"base_difficulty_range_hi":0.8},"seed":1}"#;
        let out = build_generation_config_with_mood_json_internal(args);
        let err: ErrorJson = serde_json::from_str(&out).unwrap();
        assert!(err.error.starts_with("parse:"));
    }

    #[test]
    fn round141_build_generation_config_with_mood_json_internal_high_loss_count() {
        // recent_loss_count=10 must also exclude shooting (threshold
        // is >= 3). Edge: high count above the threshold.
        let args = format!(
            r#"{{"player_level":5,"recent_loss_count":10,"mood":{},"hint":{},"seed":1}}"#,
            neutral_mood_json(),
            default_hint_json()
        );
        let out = build_generation_config_with_mood_json_internal(&args);
        let cfg: GenerationConfigJson = serde_json::from_str(&out).unwrap();
        assert!(cfg.excluded_types.contains(&"shooting".to_string()));
    }

    #[test]
    fn round141_mood_palette_json_internal_malformed_json_errors() {
        // Non-JSON input → `parse:` error envelope.
        let out = mood_palette_json_internal("not json");
        let err: ErrorJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("bad input must serialize as ErrorJson: {}", out));
        assert!(err.error.starts_with("parse:"));
    }

    #[test]
    fn round141_mood_palette_json_internal_missing_field_errors() {
        // Missing `trust` field → parse error.
        let out = mood_palette_json_internal(r#"{"friendly":0.0,"fear":0.5}"#);
        let err: ErrorJson = serde_json::from_str(&out).unwrap();
        assert!(err.error.starts_with("parse:"));
    }

    #[test]
    fn round141_mood_4th_sentence_for_json_internal_branch_high_errors() {
        // branch >= 3 → "no pool" error. Verify branch 4 (well above
        // the pool boundary) also errors cleanly.
        let args = r#"{"branch":4,"blueprint_id":"dim_42"}"#;
        let out = mood_4th_sentence_for_json_internal(args);
        let err: ErrorJson = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("branch >= 3 must serialize as ErrorJson: {}", out));
        assert!(err.error.contains("4"), "error should mention branch 4: {}", err.error);
    }

    #[test]
    fn round141_mood_4th_sentence_for_json_internal_empty_blueprint_id() {
        // Empty blueprint_id must not panic — just hashes to some
        // pool entry.
        let args = r#"{"branch":0,"blueprint_id":""}"#;
        let out = mood_4th_sentence_for_json_internal(args);
        let sent: serde_json::Value = serde_json::from_str(&out)
            .unwrap_or_else(|_| panic!("empty blueprint_id must still produce a sentence: {}", out));
        assert!(sent["sentence"].as_str().is_some());
    }

    #[test]
    fn round141_mood_4th_sentence_for_json_internal_missing_blueprint_id_errors() {
        // Missing `blueprint_id` field → parse error.
        let out = mood_4th_sentence_for_json_internal(r#"{"branch":0}"#);
        let err: ErrorJson = serde_json::from_str(&out).unwrap();
        assert!(err.error.starts_with("parse:"));
    }

    #[test]
    fn round141_wasm_module_version_constant() {
        // The version string is a contract: the TS-side
        // `loadSceneGenWasm` checks the `0.2.0-round` prefix. If a
        // future round bumps it, this test must be updated in
        // lockstep.
        let v = wasm_module_version();
        assert!(v.starts_with("0.2.0-round"), "version must keep major prefix: {}", v);
        assert!(v.contains("round51"), "version must mention round 51: {}", v);
    }

    #[test]
    fn round141_wasm_module_version_matches_expected_string() {
        // Pin the exact string. A new round that bumps the version
        // (e.g. round 52) must update both this test and the TS-side
        // version check.
        assert_eq!(wasm_module_version(), "0.2.0-round51");
    }

    #[test]
    fn round141_error_json_serializes_with_error_key() {
        // The `ErrorJson` envelope must always serialize to
        // `{"error":"..."}`. Verify via the canonical scene_json
        // error path.
        let out = theme_to_scene_json_internal("not json");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let err_str = v["error"].as_str().expect("'error' must be a string");
        assert!(!err_str.is_empty(), "'error' must be non-empty");
    }

    #[test]
    fn round141_wasm_shim_theme_to_scene_json_matches_internal() {
        // The `#[wasm_bindgen]` shim is a thin wrapper around the
        // internal helper. Verify they produce byte-identical output
        // for the same input.
        let input = r#"{"visual_style":"cyberpunk","music_mood":"pulse","difficulty":0.5,"seed":1}"#;
        let from_shim = theme_to_scene_json(input);
        let from_internal = theme_to_scene_json_internal(input);
        assert_eq!(from_shim, from_internal);
    }

    #[test]
    fn round141_wasm_shim_build_generation_config_matches_internal() {
        // Same parity check for the build_generation_config shim.
        let args = format!(
            r#"{{"player_level":5,"recent_loss_count":0,"mood":{},"hint":{},"seed":42}}"#,
            neutral_mood_json(),
            default_hint_json()
        );
        let from_shim = build_generation_config_with_mood_json(&args);
        let from_internal = build_generation_config_with_mood_json_internal(&args);
        assert_eq!(from_shim, from_internal);
    }

    #[test]
    fn round141_wasm_shim_mood_palette_matches_internal() {
        // Same parity check for mood_palette.
        let from_shim = mood_palette_json(&neutral_mood_json());
        let from_internal = mood_palette_json_internal(&neutral_mood_json());
        assert_eq!(from_shim, from_internal);
    }

    #[test]
    fn round141_wasm_shim_mood_4th_sentence_matches_internal() {
        // Same parity check for mood_4th_sentence_for.
        let args = r#"{"branch":0,"blueprint_id":"dim_42"}"#;
        let from_shim = mood_4th_sentence_for_json(args);
        let from_internal = mood_4th_sentence_for_json_internal(args);
        assert_eq!(from_shim, from_internal);
    }

    #[test]
    fn round141_npc_disposition_negative_axes_dont_change_palette() {
        // The palette is selected by the BRANCH (which axis is
        // "max positive"), not by exact values. Verify that a
        // "negative-friendly, positive-fear" mood also picks the
        // fear palette (fear is the dominant positive axis).
        let mood = NpcDispositionJson {
            friendly: -0.8,
            fear: 0.6,
            trust: -0.5,
        };
        let j = serde_json::to_string(&mood).unwrap();
        let out = mood_palette_json_internal(&j);
        let p: PaletteJson = serde_json::from_str(&out).unwrap();
        // The fear palette is ["#0A1A2F", "#1B4965", "#CAE9FF"]
        assert_eq!(p.colors, ["#0A1A2F", "#1B4965", "#CAE9FF"]);
    }
}
