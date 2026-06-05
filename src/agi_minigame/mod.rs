pub mod atom;
pub mod dimension;
pub mod economy;
pub mod player;
pub mod gameplay;
pub mod ai_engine;
pub mod world_state;
pub mod atoms;
pub mod dsl;
pub mod vault;
pub mod npc;
pub mod scene_gen;

pub use atom::{Atom, AtomId, AtomRegistry, AtomContext, AtomPhase, AtomRunner, AtomMetadata};
pub use dimension::{Dimension, DimensionConfig, DimensionRunner, DimensionState, DimensionObjective, DimensionProgress};
pub use economy::{Currency, CurrencyType, Inventory, InventoryItem, Transaction, Wallet};
pub use player::{PlayerAccount, PlayerProfile, PlayerProgression, PlayerStatsMap};
pub use gameplay::{GameplayType, GameplayState, GameplayEvent, GameplayModule};
pub use ai_engine::{AiEngine, DimensionGenerator, DimensionBlueprint, RuleComposer, BalanceTuner, GenerationConfig};
pub use world_state::{UnifiedWorldState, SharedWorld, WorldEvent, PlayerStats};
pub use atoms::register_all_atoms;
pub use vault::{DimensionOutcome, DimensionVault, VaultEntry, VaultStats};
pub use npc::{NpcDisposition, NpcId, NpcMemoryEntry, NpcMemoryKind, NpcMind, NpcMood, NpcRegistry};
pub use scene_gen::{
    build_generation_config_with_mood, mood_promoted_atoms,
    mood_palette, palette_accent, palette_background,
    GenerationHint, Palette, ALL_PALETTES, FEAR_PALETTE, FRIENDLY_PALETTE, HOSTILE_PALETTE, NEUTRAL_PALETTE,
    theme_to_scene, default_wfc_weights,
    BiomeId, EventStep, MusicMood, NpcArchetype, SceneBlueprint, ThemeInput, VisualStyle,
};
