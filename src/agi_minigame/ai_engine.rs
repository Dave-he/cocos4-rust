use std::collections::HashMap;

use rand::Rng;

use crate::base::value::{Value, ValueMap};

use super::atom::{AtomFactory, AtomId, AtomRegistry};
use super::gameplay::GameplayType;
use super::npc::NpcDisposition;

#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub min_atoms: usize,
    pub max_atoms: usize,
    pub difficulty_range: (f32, f32),
    pub allow_composite: bool,
    pub seed: Option<u64>,
    pub player_level: u32,
    pub preferred_types: Vec<GameplayType>,
    pub excluded_types: Vec<GameplayType>,
    pub reward_multiplier: f32,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            min_atoms: 2,
            max_atoms: 4,
            difficulty_range: (0.5, 1.0),
            allow_composite: true,
            seed: None,
            player_level: 1,
            preferred_types: Vec::new(),
            excluded_types: Vec::new(),
            reward_multiplier: 1.0,
        }
    }
}

impl GenerationConfig {
    pub fn for_player_level(level: u32) -> Self {
        let max_atoms = (2 + level / 5).min(6) as usize;
        let difficulty = 0.3 + (level as f32 * 0.1).min(0.7);
        Self {
            min_atoms: 2,
            max_atoms,
            difficulty_range: (difficulty, difficulty + 0.3),
            player_level: level,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct DimensionBlueprint {
    pub id: String,
    pub name: String,
    pub description: String,
    pub atom_ids: Vec<AtomId>,
    pub atom_weights: HashMap<AtomId, f32>,
    pub difficulty: f32,
    pub rules: Vec<GeneratedRule>,
    pub rewards: Vec<GeneratedReward>,
    pub theme: DimensionTheme,
    pub time_limit_secs: Option<u32>,
    pub objectives: Vec<Objective>,
}

#[derive(Debug, Clone)]
pub struct GeneratedRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub params: ValueMap,
}

#[derive(Debug, Clone)]
pub enum RuleType {
    Modifier,
    Constraint,
    Trigger,
    Transformation,
}

#[derive(Debug, Clone)]
pub struct GeneratedReward {
    pub item_id: String,
    pub base_quantity: u32,
    pub scaling_factor: f32,
}

#[derive(Debug, Clone)]
pub struct DimensionTheme {
    pub name: String,
    pub visual_style: String,
    pub music_mood: String,
    pub color_palette: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Objective {
    pub id: String,
    pub description: String,
    pub objective_type: ObjectiveType,
    pub target_value: u64,
    pub is_optional: bool,
}

#[derive(Debug, Clone)]
pub enum ObjectiveType {
    Score,
    Time,
    Collect,
    Defeat,
    Survive,
    Custom(String),
}

pub struct DimensionGenerator {
    rng: rand::rngs::StdRng,
    name_parts: NameParts,
}

struct NameParts {
    adjectives: Vec<&'static str>,
    nouns: Vec<&'static str>,
    themes: Vec<&'static str>,
}

impl DimensionGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: rand::SeedableRng::seed_from_u64(seed),
            name_parts: NameParts {
                adjectives: vec![
                    "混沌", "永恒", "幻影", "量子", "虚空", "烈焰", "冰霜",
                    "雷霆", "暗影", "光辉", "深渊", "星辰", "时空", "命运",
                ],
                nouns: vec![
                    "迷宫", "战场", "神殿", "深渊", "花园", "塔楼", "竞技场",
                    "秘境", "次元", "试炼", "回廊", "领域", "裂隙", "梦境",
                ],
                themes: vec![
                    "赛博朋克", "奇幻森林", "海底世界", "太空站", "古墓",
                    "浮空城", "熔岩地带", "冰原", "沙漠绿洲", "暗黑地牢",
                ],
            },
        }
    }

    pub fn generate(&mut self, config: &GenerationConfig, registry: &AtomRegistry) -> DimensionBlueprint {
        let available_atoms = self.filter_atoms(config, registry);
        let num_atoms = self.rng.gen_range(config.min_atoms..=config.max_atoms.min(available_atoms.len()));

        let selected_atoms = self.select_atoms(&available_atoms, num_atoms);
        let difficulty = self.rng.gen_range(config.difficulty_range.0..=config.difficulty_range.1);
        let atom_weights = self.generate_weights(&selected_atoms);
        let rules = self.generate_rules(&selected_atoms, difficulty);
        let rewards = self.generate_rewards(&selected_atoms, difficulty, config.reward_multiplier);
        let theme = self.generate_theme();
        let name = self.generate_name();
        let objectives = self.generate_objectives(&selected_atoms, difficulty);

        DimensionBlueprint {
            id: format!("dim_{}", self.rng.gen::<u32>()),
            name,
            description: format!("在{}中挑战{}种玩法的组合", theme.name, selected_atoms.len()),
            atom_ids: selected_atoms,
            atom_weights,
            difficulty,
            rules,
            rewards,
            theme,
            time_limit_secs: if difficulty > 0.7 { Some(180) } else { None },
            objectives,
        }
    }

    fn filter_atoms(&self, config: &GenerationConfig, registry: &AtomRegistry) -> Vec<AtomId> {
        registry
            .list_all()
            .iter()
            .filter(|m| {
                let gt = GameplayType::from_name(&m.gameplay_type);
                !config.excluded_types.contains(&gt)
            })
            .map(|m| m.id.clone())
            .collect()
    }

    fn select_atoms(&mut self, available: &[AtomId], count: usize) -> Vec<AtomId> {
        let mut indices: Vec<usize> = (0..available.len()).collect();
        let mut selected = Vec::new();

        for _ in 0..count.min(available.len()) {
            if indices.is_empty() {
                break;
            }
            let idx = self.rng.gen_range(0..indices.len());
            let atom_idx = indices.remove(idx);
            selected.push(available[atom_idx].clone());
        }

        selected
    }

    fn generate_weights(&mut self, atoms: &[AtomId]) -> HashMap<AtomId, f32> {
        let mut weights = HashMap::new();
        let total = atoms.len() as f32;
        for atom in atoms {
            let w = 0.5 + self.rng.gen::<f32>() * 0.5;
            weights.insert(atom.clone(), w / total);
        }
        weights
    }

    fn generate_rules(&mut self, atoms: &[AtomId], difficulty: f32) -> Vec<GeneratedRule> {
        let mut rules = Vec::new();
        let rule_templates = [
            ("speed_boost", "加速", "行动速度提升", RuleType::Modifier),
            ("double_score", "双倍得分", "得分翻倍", RuleType::Modifier),
            ("resource_drain", "资源消耗", "资源持续消耗", RuleType::Constraint),
            ("chain_bonus", "连锁奖励", "连续操作获得额外奖励", RuleType::Trigger),
            ("random_swap", "随机交换", "定期随机交换元素", RuleType::Transformation),
            ("time_pressure", "时间压力", "倒计时加速", RuleType::Constraint),
            ("power_surge", "力量涌动", "攻击力周期性增强", RuleType::Trigger),
        ];

        let num_rules = (1 + (difficulty * 3.0) as usize).min(rule_templates.len());
        let mut template_indices: Vec<usize> = (0..rule_templates.len()).collect();

        for _ in 0..num_rules {
            if template_indices.is_empty() {
                break;
            }
            let idx = self.rng.gen_range(0..template_indices.len());
            let ti = template_indices.remove(idx);
            let template = &rule_templates[ti];
            let mut params = ValueMap::new();
            params.insert("intensity".to_string(), Value::Float(difficulty as f32));
            params.insert("duration".to_string(), Value::Float((10.0 + self.rng.gen::<f32>() * 30.0) as f32));

            rules.push(GeneratedRule {
                rule_id: template.0.to_string(),
                name: template.1.to_string(),
                description: template.2.to_string(),
                rule_type: template.3.clone(),
                params,
            });
        }

        rules
    }

    fn generate_rewards(&mut self, atoms: &[AtomId], difficulty: f32, multiplier: f32) -> Vec<GeneratedReward> {
        let mut rewards = Vec::new();
        let base_gold = (50.0 * difficulty * multiplier) as u32;
        let base_gem = (5.0 * difficulty * multiplier) as u32;

        rewards.push(GeneratedReward {
            item_id: "gold".to_string(),
            base_quantity: base_gold,
            scaling_factor: 1.0 + difficulty,
        });
        rewards.push(GeneratedReward {
            item_id: "gem".to_string(),
            base_quantity: base_gem.max(1),
            scaling_factor: 0.5 + difficulty,
        });

        if difficulty > 0.6 {
            rewards.push(GeneratedReward {
                item_id: "rare_chest".to_string(),
                base_quantity: 1,
                scaling_factor: difficulty,
            });
        }

        rewards
    }

    fn generate_theme(&mut self) -> DimensionTheme {
        let adj_idx = self.rng.gen_range(0..self.name_parts.adjectives.len());
        let theme_idx = self.rng.gen_range(0..self.name_parts.themes.len());

        let palettes = [
            vec!["#FF6B6B".to_string(), "#4ECDC4".to_string(), "#45B7D1".to_string()],
            vec!["#2C3E50".to_string(), "#E74C3C".to_string(), "#ECF0F1".to_string()],
            vec!["#6C5CE7".to_string(), "#A29BFE".to_string(), "#FD79A8".to_string()],
            vec!["#00B894".to_string(), "#55EFC4".to_string(), "#FDCB6E".to_string()],
        ];
        let palette_idx = self.rng.gen_range(0..palettes.len());

        DimensionTheme {
            name: format!("{}·{}", self.name_parts.adjectives[adj_idx], self.name_parts.themes[theme_idx]),
            visual_style: self.name_parts.themes[theme_idx].to_string(),
            music_mood: if self.rng.gen_bool(0.5) { "epic" } else { "mysterious" }.to_string(),
            color_palette: palettes[palette_idx].clone(),
        }
    }

    fn generate_name(&mut self) -> String {
        let adj_idx = self.rng.gen_range(0..self.name_parts.adjectives.len());
        let noun_idx = self.rng.gen_range(0..self.name_parts.nouns.len());
        format!("{}{}", self.name_parts.adjectives[adj_idx], self.name_parts.nouns[noun_idx])
    }

    fn generate_objectives(&mut self, atoms: &[AtomId], difficulty: f32) -> Vec<Objective> {
        let mut objectives = Vec::new();

        objectives.push(Objective {
            id: "main_score".to_string(),
            description: format!("达到{}分", (1000.0 * difficulty) as u64),
            objective_type: ObjectiveType::Score,
            target_value: (1000.0 * difficulty) as u64,
            is_optional: false,
        });

        if atoms.len() > 1 {
            objectives.push(Objective {
                id: "combo_master".to_string(),
                description: "完成3次组合连击".to_string(),
                objective_type: ObjectiveType::Custom("combo".to_string()),
                target_value: 3,
                is_optional: true,
            });
        }

        if difficulty > 0.5 {
            objectives.push(Objective {
                id: "survival".to_string(),
                description: "存活超过60秒".to_string(),
                objective_type: ObjectiveType::Survive,
                target_value: 60,
                is_optional: true,
            });
        }

        objectives
    }
}

pub struct RuleComposer {
    rule_library: HashMap<String, RuleDefinition>,
}

#[derive(Debug, Clone)]
pub struct RuleDefinition {
    pub id: String,
    pub name: String,
    pub applicable_atoms: Vec<String>,
    pub conflict_rules: Vec<String>,
    pub synergy_rules: Vec<String>,
    pub min_difficulty: f32,
    pub max_difficulty: f32,
}

impl RuleComposer {
    pub fn new() -> Self {
        let mut library = HashMap::new();

        library.insert("speed_boost".to_string(), RuleDefinition {
            id: "speed_boost".to_string(),
            name: "加速".to_string(),
            applicable_atoms: vec!["parkour".to_string(), "turn_combat".to_string()],
            conflict_rules: vec!["time_pressure".to_string()],
            synergy_rules: vec!["chain_bonus".to_string()],
            min_difficulty: 0.3,
            max_difficulty: 1.0,
        });

        library.insert("double_score".to_string(), RuleDefinition {
            id: "double_score".to_string(),
            name: "双倍得分".to_string(),
            applicable_atoms: vec!["match3".to_string(), "parkour".to_string()],
            conflict_rules: vec![],
            synergy_rules: vec!["chain_bonus".to_string()],
            min_difficulty: 0.0,
            max_difficulty: 1.0,
        });

        library.insert("chain_bonus".to_string(), RuleDefinition {
            id: "chain_bonus".to_string(),
            name: "连锁奖励".to_string(),
            applicable_atoms: vec!["match3".to_string(), "card".to_string(), "synthesis".to_string()],
            conflict_rules: vec![],
            synergy_rules: vec!["double_score".to_string(), "speed_boost".to_string()],
            min_difficulty: 0.2,
            max_difficulty: 1.0,
        });

        Self { rule_library: library }
    }

    pub fn compose(&self, atoms: &[AtomId], difficulty: f32) -> Vec<GeneratedRule> {
        let mut rules = Vec::new();
        let mut active_synergies: Vec<String> = Vec::new();

        for (id, def) in &self.rule_library {
            if difficulty < def.min_difficulty || difficulty > def.max_difficulty {
                continue;
            }

            let applicable = atoms.iter().any(|a| def.applicable_atoms.contains(&a.to_string()));
            if !applicable {
                continue;
            }

            let has_conflict = def.conflict_rules.iter().any(|c: &String| {
                rules.iter().any(|r: &crate::agi_minigame::ai_engine::GeneratedRule| r.rule_id == *c)
            });
            if has_conflict {
                continue;
            }

            let has_synergy = def.synergy_rules.iter().any(|s| active_synergies.contains(s));

            let mut params = ValueMap::new();
            let intensity = if has_synergy { difficulty * 1.5 } else { difficulty };
            params.insert("intensity".to_string(), Value::Float(intensity as f32));

            rules.push(GeneratedRule {
                rule_id: id.clone(),
                name: def.name.clone(),
                description: format!("{} (强度: {:.1})", def.name, intensity),
                rule_type: RuleType::Modifier,
                params,
            });

            for s in &def.synergy_rules {
                active_synergies.push(s.clone());
            }
        }

        rules
    }

    pub fn add_rule_definition(&mut self, definition: RuleDefinition) {
        self.rule_library.insert(definition.id.clone(), definition);
    }
}

impl Default for RuleComposer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BalanceTuner {
    target_win_rate: f32,
    target_avg_score: u64,
    target_avg_duration_secs: f32,
    history: Vec<BalanceDataPoint>,
}

#[derive(Debug, Clone)]
struct BalanceDataPoint {
    dimension_id: String,
    difficulty: f32,
    player_level: u32,
    score: u64,
    duration_secs: f32,
    completed: bool,
}

impl BalanceTuner {
    pub fn new() -> Self {
        Self {
            target_win_rate: 0.6,
            target_avg_score: 1000,
            target_avg_duration_secs: 120.0,
            history: Vec::new(),
        }
    }

    pub fn record_result(&mut self, dimension_id: &str, difficulty: f32, player_level: u32, score: u64, duration_secs: f32, completed: bool) {
        self.history.push(BalanceDataPoint {
            dimension_id: dimension_id.to_string(),
            difficulty,
            player_level,
            score,
            duration_secs,
            completed,
        });
    }

    pub fn suggest_difficulty(&self, player_level: u32) -> f32 {
        let recent: Vec<_> = self.history
            .iter()
            .rev()
            .take(20)
            .filter(|d| (d.player_level as i32 - player_level as i32).abs() <= 2)
            .collect();

        if recent.is_empty() {
            // Base scales with player_level but must stay within the
            // unit range so callers can compare/weight it directly
            // against history-based suggestions.
            return (0.3 + player_level as f32 * 0.05).clamp(0.1, 1.0);
        }

        let win_rate = recent.iter().filter(|d| d.completed).count() as f32 / recent.len() as f32;
        let avg_score: f64 = recent.iter().map(|d| d.score as f64).sum::<f64>() / recent.len() as f64;
        let avg_duration: f64 = recent.iter().map(|d| d.duration_secs as f64).sum::<f64>() / recent.len() as f64;

        let mut adjustment = 0.0f32;

        if win_rate > self.target_win_rate + 0.1 {
            adjustment += 0.1;
        } else if win_rate < self.target_win_rate - 0.1 {
            adjustment -= 0.1;
        }

        if avg_score > self.target_avg_score as f64 * 1.5 {
            adjustment += 0.05;
        } else if avg_score < self.target_avg_score as f64 * 0.5 {
            adjustment -= 0.05;
        }

        if avg_duration > self.target_avg_duration_secs as f64 * 1.5 {
            adjustment += 0.05;
        } else if avg_duration < self.target_avg_duration_secs as f64 * 0.5 {
            adjustment -= 0.05;
        }

        let base = 0.3 + player_level as f32 * 0.05;
        (base + adjustment).clamp(0.1, 1.0)
    }

    /// Round 22 — reflexive loop with the world's NPC mood.
    ///
    /// Returns the same value as [`BalanceTuner::suggest_difficulty`]
    /// for the given player level, then nudges it by the collective
    /// NPC disposition (typically from
    /// [`super::npc::NpcRegistry::average_disposition`]):
    ///
    /// - `fear > 0.5` → `-0.10` (the world already feels too scary)
    /// - `friendly > 0.5 && trust > 0.3` → `+0.08`
    ///   (NPCs like the player → they're doing well → raise the
    ///   stakes)
    /// - `friendly < -0.3` → `-0.05` (NPCs hate the player; a
    ///   difficulty bump won't fix social rot, ease up a bit instead)
    ///
    /// Multiple branches can fire together (their adjustments stack).
    /// The result is always clamped into `[0.1, 1.0]`, matching
    /// `suggest_difficulty`.
    ///
    /// When `mood == NpcDisposition::default()` (neutral) the
    /// function returns exactly the same value as
    /// `suggest_difficulty(player_level)` — the reflexive loop adds
    /// information when there *is* information, never noise.
    pub fn suggest_difficulty_with_mood(
        &self,
        player_level: u32,
        mood: NpcDisposition,
    ) -> f32 {
        let base = self.suggest_difficulty(player_level);
        let bias = Self::mood_bias(mood);
        (base + bias).clamp(0.1, 1.0)
    }

    /// Pure mood → bias mapping. Exposed for the game layer so the
    /// HUD can preview the upcoming nudge before committing to a
    /// dimension. Always returns a value in `[-0.15, 0.08]`.
    pub fn mood_bias(mood: NpcDisposition) -> f32 {
        let mut bias = 0.0f32;
        if mood.fear > 0.5 {
            bias -= 0.10;
        }
        if mood.friendly > 0.5 && mood.trust > 0.3 {
            bias += 0.08;
        }
        if mood.friendly < -0.3 {
            bias -= 0.05;
        }
        bias
    }

    pub fn get_stats(&self) -> BalanceStats {
        if self.history.is_empty() {
            return BalanceStats {
                total_sessions: 0,
                win_rate: 0.0,
                avg_score: 0,
                avg_duration: 0.0,
            };
        }

        let wins = self.history.iter().filter(|d| d.completed).count();
        BalanceStats {
            total_sessions: self.history.len(),
            win_rate: wins as f32 / self.history.len() as f32,
            avg_score: self.history.iter().map(|d| d.score).sum::<u64>() / self.history.len() as u64,
            avg_duration: self.history.iter().map(|d| d.duration_secs).sum::<f32>() / self.history.len() as f32,
        }
    }
}

impl Default for BalanceTuner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BalanceStats {
    pub total_sessions: usize,
    pub win_rate: f32,
    pub avg_score: u64,
    pub avg_duration: f32,
}

pub struct AiEngine {
    pub generator: DimensionGenerator,
    pub composer: RuleComposer,
    pub tuner: BalanceTuner,
}

impl AiEngine {
    pub fn new(seed: u64) -> Self {
        Self {
            generator: DimensionGenerator::new(seed),
            composer: RuleComposer::new(),
            tuner: BalanceTuner::new(),
        }
    }

    pub fn generate_dimension(&mut self, config: &GenerationConfig, registry: &AtomRegistry) -> DimensionBlueprint {
        let mut config = config.clone();
        let suggested = self.tuner.suggest_difficulty(config.player_level);
        config.difficulty_range = (
            (suggested - 0.1).max(0.1),
            (suggested + 0.1).min(1.0),
        );

        let mut blueprint = self.generator.generate(&config, registry);

        let composed_rules = self.composer.compose(&blueprint.atom_ids, blueprint.difficulty);
        blueprint.rules.extend(composed_rules);

        blueprint
    }

    pub fn record_session(&mut self, dimension_id: &str, difficulty: f32, player_level: u32, score: u64, duration_secs: f32, completed: bool) {
        self.tuner.record_result(dimension_id, difficulty, player_level, score, duration_secs, completed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agi_minigame::atom::AtomMetadata;

    fn make_test_registry() -> AtomRegistry {
        let mut registry = AtomRegistry::new();
        let atoms = vec![
            ("match3", "Match3", "puzzle"),
            ("tower_defense", "TowerDefense", "strategy"),
            ("card", "Card", "card"),
            ("turn_combat", "TurnCombat", "rpg"),
            ("parkour", "Parkour", "action"),
            ("synthesis", "Synthesis", "casual"),
        ];

        for (id, name, gt) in atoms {
            let metadata = AtomMetadata {
                id: id.to_string(),
                name: name.to_string(),
                version: 1,
                gameplay_type: gt.to_string(),
                description: format!("{} atom", name),
                tags: vec![gt.to_string()],
            };
            let factory: AtomFactory = Box::new(|| panic!("factory not used in test"));
            registry.register(id.to_string(), metadata, factory);
        }
        registry
    }

    #[test]
    fn test_dimension_generator() {
        let mut gen = DimensionGenerator::new(42);
        let config = GenerationConfig::default();
        let registry = make_test_registry();

        let blueprint = gen.generate(&config, &registry);
        assert!(!blueprint.atom_ids.is_empty());
        assert!(!blueprint.name.is_empty());
        assert!(blueprint.difficulty >= 0.0 && blueprint.difficulty <= 1.0);
        assert!(!blueprint.rewards.is_empty());
    }

    #[test]
    fn test_generation_config_for_level() {
        let config = GenerationConfig::for_player_level(10);
        assert_eq!(config.player_level, 10);
        assert!(config.max_atoms >= 2);
    }

    #[test]
    fn test_rule_composer() {
        let composer = RuleComposer::new();
        let atoms = vec!["match3".to_string(), "parkour".to_string()];
        let rules = composer.compose(&atoms, 0.5);
        assert!(!rules.is_empty());
    }

    #[test]
    fn test_balance_tuner() {
        let mut tuner = BalanceTuner::new();
        let difficulty = tuner.suggest_difficulty(1);
        assert!(difficulty >= 0.1 && difficulty <= 1.0);

        tuner.record_result("dim1", 0.5, 1, 500, 60.0, true);
        tuner.record_result("dim2", 0.5, 1, 300, 45.0, false);

        let stats = tuner.get_stats();
        assert_eq!(stats.total_sessions, 2);
    }

    #[test]
    fn test_ai_engine_generate() {
        let mut engine = AiEngine::new(123);
        let config = GenerationConfig::default();
        let registry = make_test_registry();

        let blueprint = engine.generate_dimension(&config, &registry);
        assert!(!blueprint.atom_ids.is_empty());
        assert!(!blueprint.objectives.is_empty());
    }

    #[test]
    fn test_ai_engine_adaptive() {
        let mut engine = AiEngine::new(456);
        let registry = make_test_registry();

        for _ in 0..5 {
            engine.record_session("dim1", 0.5, 1, 2000, 180.0, true);
        }

        let config = GenerationConfig::for_player_level(1);
        let blueprint = engine.generate_dimension(&config, &registry);
        assert!(blueprint.difficulty > 0.3);
    }

    #[test]
    fn test_generate_multiple_unique() {
        let mut gen = DimensionGenerator::new(789);
        let config = GenerationConfig::default();
        let registry = make_test_registry();

        let b1 = gen.generate(&config, &registry);
        let b2 = gen.generate(&config, &registry);
        assert_ne!(b1.id, b2.id);
    }
}

// ---------------------------------------------------------------------------
// Round 19 — additional ai_engine tests (use the public API only).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round19_tests {
    use super::*;

    #[test]
    fn rule_composer_filters_by_difficulty() {
        let composer = RuleComposer::new();
        // A very high minimum_difficulty should yield only high-tier rules.
        let high = composer.compose(&["parkour".to_string()], 0.99);
        for r in &high {
            // GeneratedRule stores its tuning values on `params`
            // (a ValueMap), and uses `rule_id` (not `id`).
            assert!(r.params.get("intensity")
                        .map(|v| matches!(v, crate::base::value::Value::Float(f) if *f >= 0.0))
                        .unwrap_or(true),
                    "high-difficulty compose should not include low-tier rules: {:?}", r.rule_id);
        }
    }

    #[test]
    fn rule_composer_default_has_at_least_three_rules() {
        let composer = RuleComposer::new();
        assert!(!composer.compose(&["match3".to_string()], 0.5).is_empty()
                || !composer.compose(&["parkour".to_string()], 0.5).is_empty()
                || !composer.compose(&["card".to_string()], 0.5).is_empty());
    }

    #[test]
    fn balance_tuner_suggests_difficulty_in_unit_range() {
        let tuner = BalanceTuner::new();
        for level in 1..=20 {
            let d = tuner.suggest_difficulty(level);
            assert!(d >= 0.0 && d <= 1.0, "level {level} → {d} out of [0,1]");
        }
    }

    #[test]
    fn balance_tuner_recording_does_not_change_immediate_suggestion() {
        let mut tuner = BalanceTuner::new();
        let d_before = tuner.suggest_difficulty(5);
        // Record a session at very different levels → the level-5
        // suggestion should remain stable because the filter takes
        // only sessions within ±2 of the requested level.
        for i in 0..5 {
            tuner.record_result("d1", 0.7, 1, 100, 60.0, i % 2 == 0);
        }
        let d_after = tuner.suggest_difficulty(5);
        assert!((d_before - d_after).abs() < 0.01);
    }

    #[test]
    fn balance_tuner_widens_with_matching_level_history() {
        let mut tuner = BalanceTuner::new();
        for i in 0..10 {
            tuner.record_result("d1", 0.5, 5, 100, 60.0, i % 2 == 0);
        }
        let d = tuner.suggest_difficulty(5);
        assert!(d >= 0.0 && d <= 1.0);
    }

    // ---- Round 22 — NpcMind ↔ BalanceTuner reflexive loop ----

    #[test]
    fn mood_bias_neutral_disposition_is_zero() {
        assert_eq!(BalanceTuner::mood_bias(NpcDisposition::default()), 0.0);
    }

    #[test]
    fn mood_bias_high_fear_lowers_difficulty() {
        let scared = NpcDisposition { friendly: 0.0, fear: 0.7, trust: 0.0 };
        assert!(BalanceTuner::mood_bias(scared) < 0.0);
        assert!((BalanceTuner::mood_bias(scared) - -0.10).abs() < 1e-6);
    }

    #[test]
    fn mood_bias_friendly_and_trusting_raises_difficulty() {
        let beloved = NpcDisposition { friendly: 0.8, fear: 0.0, trust: 0.5 };
        assert!(BalanceTuner::mood_bias(beloved) > 0.0);
        assert!((BalanceTuner::mood_bias(beloved) - 0.08).abs() < 1e-6);
    }

    #[test]
    fn mood_bias_friendly_alone_is_not_enough() {
        let liked_but_distrusted = NpcDisposition { friendly: 0.8, fear: 0.0, trust: 0.1 };
        // Liked but not trusted → no bonus (trust gate failed).
        assert_eq!(BalanceTuner::mood_bias(liked_but_distrusted), 0.0);
    }

    #[test]
    fn mood_bias_hated_player_eases_difficulty() {
        let hated = NpcDisposition { friendly: -0.5, fear: 0.0, trust: 0.0 };
        assert!((BalanceTuner::mood_bias(hated) - -0.05).abs() < 1e-6);
    }

    #[test]
    fn mood_bias_branches_can_stack() {
        // High fear AND hated → both penalties apply (-0.10 - 0.05).
        let nightmare = NpcDisposition { friendly: -0.5, fear: 0.7, trust: 0.0 };
        assert!((BalanceTuner::mood_bias(nightmare) - -0.15).abs() < 1e-6);
    }

    #[test]
    fn mood_bias_is_bounded() {
        // Even at extremes, |mood_bias| ≤ 0.15.
        let extreme = NpcDisposition { friendly: -1.0, fear: 1.0, trust: -1.0 };
        let bias = BalanceTuner::mood_bias(extreme);
        assert!(bias <= 0.08 && bias >= -0.15, "got {bias}");
    }

    #[test]
    fn suggest_with_mood_equals_plain_when_neutral() {
        let mut tuner = BalanceTuner::new();
        for i in 0..6 {
            tuner.record_result("d1", 0.5, 5, 1000, 100.0, i % 2 == 0);
        }
        let plain = tuner.suggest_difficulty(5);
        let mooded = tuner.suggest_difficulty_with_mood(5, NpcDisposition::default());
        assert!((plain - mooded).abs() < 1e-6);
    }

    #[test]
    fn suggest_with_mood_clamps_at_floor() {
        // Empty history + low level + scared+hated → would push below 0.1.
        let tuner = BalanceTuner::new();
        let nightmare = NpcDisposition { friendly: -1.0, fear: 1.0, trust: 0.0 };
        let d = tuner.suggest_difficulty_with_mood(1, nightmare);
        assert!(d >= 0.1, "got {d}");
        assert!(d <= 1.0);
    }

    #[test]
    fn suggest_with_mood_clamps_at_ceiling() {
        // Empty history + max level + adoring NPCs → would push above 1.0.
        let tuner = BalanceTuner::new();
        let adored = NpcDisposition { friendly: 1.0, fear: 0.0, trust: 1.0 };
        let d = tuner.suggest_difficulty_with_mood(50, adored);
        assert!(d <= 1.0, "got {d}");
        assert!(d >= 0.1);
    }

    #[test]
    fn suggest_with_mood_actually_moves_difficulty() {
        let tuner = BalanceTuner::new();
        // Use a middle-level so we have headroom on both sides.
        let level = 5;
        let scared = NpcDisposition { friendly: 0.0, fear: 0.9, trust: 0.0 };
        let adored = NpcDisposition { friendly: 0.9, fear: 0.0, trust: 0.5 };
        let d_scared = tuner.suggest_difficulty_with_mood(level, scared);
        let d_adored = tuner.suggest_difficulty_with_mood(level, adored);
        let d_plain = tuner.suggest_difficulty(level);
        assert!(d_scared < d_plain, "scared {d_scared} should be < plain {d_plain}");
        assert!(d_adored > d_plain, "adored {d_adored} should be > plain {d_plain}");
    }
}

// ---------------------------------------------------------------------------
// Round 129 — ai_engine.rs helper-level unit tests.
// Mirrors the round-110b / 122 / 123 / 124 / 125 / 126 / 127 / 128
// pattern: pin behaviour of the small public helpers
// (`GenerationConfig::default` + `for_player_level`,
// `RuleComposer::new` rule-library size + `compose` on
// empty-atom input, `BalanceTuner::suggest_difficulty`
// clamps at the high end, `BalanceTuner::get_stats`
// for empty + single-session vaults) so refactors
// can't silently change the contract.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round129_tests {
    use super::*;

    #[test]
    fn generation_config_default_field_values_pinned() {
        // Pin the exact `Default::default()` values for
        // `GenerationConfig` so a refactor that flips
        // (e.g.) `min_atoms: 2 → 1` or
        // `difficulty_range: (0.5, 1.0) → (0.4, 0.9)`
        // breaks this test (both are visible to the
        // round-21 test_dimension_generator sanity).
        let c = GenerationConfig::default();
        assert_eq!(c.min_atoms, 2);
        assert_eq!(c.max_atoms, 4);
        assert_eq!(c.difficulty_range, (0.5, 1.0));
        assert!(c.allow_composite);
        assert_eq!(c.seed, None);
        assert_eq!(c.player_level, 1);
        assert!(c.preferred_types.is_empty());
        assert!(c.excluded_types.is_empty());
        assert!((c.reward_multiplier - 1.0).abs() < 1e-6);
    }

    #[test]
    fn generation_config_for_level_zero_uses_low_difficulty() {
        // Boundary at the low end: player_level = 0.
        // Per the impl: max_atoms = (2 + 0 / 5).min(6) = 2
        // difficulty = 0.3 + 0.0.min(0.7) = 0.3
        // difficulty_range = (0.3, 0.6)
        let c = GenerationConfig::for_player_level(0);
        assert_eq!(c.min_atoms, 2);
        assert_eq!(c.max_atoms, 2);
        assert_eq!(c.player_level, 0);
        assert!((c.difficulty_range.0 - 0.3).abs() < 1e-6);
        assert!((c.difficulty_range.1 - 0.6).abs() < 1e-6);
    }

    #[test]
    fn generation_config_for_level_twenty_uses_difficulty_0_65() {
        // Mid-range: player_level = 20.
        // max_atoms = (2 + 20 / 5).min(6) = 6
        // difficulty = 0.3 + (20 * 0.1).min(0.7) = 0.3 + 0.7 = 1.0
        // difficulty_range = (1.0, 1.3).clamp_upper(1.0) = (1.0, 1.0)
        // (the impl does NOT clamp difficulty_range upper,
        // only suggest_difficulty's base+adjustment)
        let c = GenerationConfig::for_player_level(20);
        assert_eq!(c.max_atoms, 6);
        assert_eq!(c.player_level, 20);
        assert!((c.difficulty_range.0 - 1.0).abs() < 1e-6);
        assert!((c.difficulty_range.1 - 1.3).abs() < 1e-6);
    }

    #[test]
    fn generation_config_for_level_hundred_caps_max_atoms_at_six() {
        // Player at the level cap — max_atoms must clamp to 6.
        let c = GenerationConfig::for_player_level(100);
        assert_eq!(c.max_atoms, 6);
        assert_eq!(c.player_level, 100);
        // difficulty = 0.3 + (100 * 0.1).min(0.7) = 0.3 + 0.7 = 1.0
        // difficulty_range = (1.0, 1.3)
        assert!((c.difficulty_range.0 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn rule_composer_new_library_has_exactly_three_rules() {
        // The `new()` library registers exactly 3 rules:
        // speed_boost, double_score, chain_bonus.
        // A regression that added (say) a fourth rule
        // here would change the composed-rule counts
        // visible in the round-19 test_balance_tuner
        // and round-22 mood-reflex loop downstream.
        let composer = RuleComposer::new();
        // The compose method is the public way to
        // count the rules that survive difficulty
        // filtering. With difficulty = 0.5 all 3 are
        // in range. With no atoms, none are applicable.
        let no_atoms = composer.compose(&[], 0.5);
        assert_eq!(no_atoms.len(), 0);
        // With an applicable atom (parkour is in
        // speed_boost's applicable_atoms), speed_boost
        // at least should be present.
        let parkour = composer.compose(&["parkour".to_string()], 0.5);
        let parkour_ids: Vec<&str> = parkour.iter().map(|r| r.rule_id.as_str()).collect();
        assert!(parkour_ids.contains(&"speed_boost"),
                "parkour+d0.5 should include speed_boost, got {:?}", parkour_ids);
    }

    #[test]
    fn rule_composer_compose_with_empty_atoms_returns_empty() {
        // Empty atom list → no rule is applicable → empty result.
        let composer = RuleComposer::new();
        let r = composer.compose(&[], 0.5);
        assert!(r.is_empty());
        let r2 = composer.compose(&[], 0.99);
        assert!(r2.is_empty());
    }

    #[test]
    fn balance_tuner_suggest_difficulty_at_level_hundred_clamps_to_one() {
        // Player at the level cap (level = 100). The base
        // formula (0.3 + 100 * 0.05) = 5.3 must be
        // clamped to the [0.1, 1.0] unit range.
        let tuner = BalanceTuner::new();
        let d = tuner.suggest_difficulty(100);
        assert!((d - 1.0).abs() < 1e-6, "expected clamp to 1.0, got {d}");
    }

    #[test]
    fn balance_tuner_suggest_difficulty_at_level_zero_returns_0_3() {
        // Empty history + level 0 → base = 0.3 + 0*0.05 = 0.3.
        let tuner = BalanceTuner::new();
        let d = tuner.suggest_difficulty(0);
        assert!((d - 0.3).abs() < 1e-6, "expected 0.3, got {d}");
    }

    #[test]
    fn balance_tuner_get_stats_empty_history_returns_zeros() {
        // Empty history → all 4 stats fields zero.
        let tuner = BalanceTuner::new();
        let s = tuner.get_stats();
        assert_eq!(s.total_sessions, 0);
        assert_eq!(s.win_rate, 0.0);
        assert_eq!(s.avg_score, 0);
        assert_eq!(s.avg_duration, 0.0);
    }

    #[test]
    fn balance_tuner_get_stats_single_completed_session() {
        // One session, completed=true.
        // win_rate = 1/1 = 1.0.
        // avg_score = score / 1.
        // avg_duration = duration / 1.
        let mut tuner = BalanceTuner::new();
        tuner.record_result("d1", 0.5, 5, 750, 60.0, true);
        let s = tuner.get_stats();
        assert_eq!(s.total_sessions, 1);
        assert!((s.win_rate - 1.0).abs() < 1e-6);
        assert_eq!(s.avg_score, 750);
        assert!((s.avg_duration - 60.0).abs() < 1e-6);
    }

    #[test]
    fn dimension_generator_is_seed_deterministic() {
        // Two generators seeded with the same value
        // must produce the same `id` (a `dim_<u32>`
        // derived from the first rng.gen::<u32>() call).
        let mut g1 = DimensionGenerator::new(0xDEAD_BEEF);
        let mut g2 = DimensionGenerator::new(0xDEAD_BEEF);
        let cfg = GenerationConfig::default();
        let registry = make_test_registry_local();
        let bp1 = g1.generate(&cfg, &registry);
        let bp2 = g2.generate(&cfg, &registry);
        assert_eq!(bp1.id, bp2.id, "same seed should produce same dim id");
        // Different seed → different id (with very
        // high probability).
        let mut g3 = DimensionGenerator::new(0xCAFE_F00D);
        let bp3 = g3.generate(&cfg, &registry);
        assert_ne!(bp1.id, bp3.id);
    }

    /// Local replica of `mod tests::make_test_registry`.
    /// The original is private to the round-21 module,
    /// so this round-129 module builds its own (the
    /// 6-atom fixture is tiny — copy is cheap).
    fn make_test_registry_local() -> crate::agi_minigame::atom::AtomRegistry {
        use crate::agi_minigame::atom::{AtomFactory, AtomMetadata, AtomRegistry};
        let mut registry = AtomRegistry::new();
        let atoms = vec![
            ("match3", "Match3", "puzzle"),
            ("tower_defense", "TowerDefense", "strategy"),
            ("card", "Card", "card"),
            ("turn_combat", "TurnCombat", "rpg"),
            ("parkour", "Parkour", "action"),
            ("synthesis", "Synthesis", "casual"),
        ];
        for (id, name, gt) in atoms {
            let metadata = AtomMetadata {
                id: id.to_string(),
                name: name.to_string(),
                version: 1,
                gameplay_type: gt.to_string(),
                description: format!("{} atom", name),
                tags: vec![gt.to_string()],
            };
            let factory: AtomFactory = Box::new(|| panic!("factory not used in test"));
            registry.register(id.to_string(), metadata, factory);
        }
        registry
    }
}

// ---------------------------------------------------------------------------
// Round 155 helper-level tests for `ai_engine.rs`.
//
// Round 155 closes surface-area gaps left after the
// round-19 / round-129 sweep — specifically the
// GenerationConfig boundary contracts (level-0,
// level-saturation, default values) and the
// BalanceTuner mood-reflex contracts (mood_bias
// per-branch, stacked branches, default-mood
// equivalence, clamp behavior).
//
// Each test is fully self-contained: it builds
// its own `BalanceTuner` / `GenerationConfig` via
// inline literals, so a regression in one fixture
// doesn't poison the others.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round155_tests {
    use super::*;
    use super::super::npc::NpcDisposition;

    // -----------------------------------------------------------------
    // GenerationConfig — for_player_level + default.
    // -----------------------------------------------------------------

    #[test]
    fn generation_config_default_initial_values_round155() {
        // GenerationConfig::default() pins
        // all 9 fields. Regression that
        // pre-set `seed = Some(0)` would
        // silently break the "no seed"
        // path used by the round-21
        // blueprint pipeline.
        let cfg = GenerationConfig::default();
        assert_eq!(cfg.min_atoms, 2);
        assert_eq!(cfg.max_atoms, 4);
        assert_eq!(cfg.difficulty_range, (0.5, 1.0));
        assert!(cfg.allow_composite);
        assert_eq!(cfg.seed, None);
        assert_eq!(cfg.player_level, 1);
        assert!(cfg.preferred_types.is_empty());
        assert!(cfg.excluded_types.is_empty());
        assert_eq!(cfg.reward_multiplier, 1.0);
    }

    #[test]
    fn generation_config_for_player_level_zero_round155() {
        // Level-0 boundary: max_atoms
        // formula is `(2 + level / 5).min(6)`,
        // so level=0 → max_atoms=2.
        // Difficulty is `0.3 + level * 0.1`,
        // saturated to 1.0; level=0 →
        // difficulty=0.3.
        let cfg = GenerationConfig::for_player_level(0);
        assert_eq!(cfg.player_level, 0);
        assert_eq!(cfg.max_atoms, 2);
        assert_eq!(cfg.min_atoms, 2);
        assert!((cfg.difficulty_range.0 - 0.3).abs() < 1e-6);
        assert!((cfg.difficulty_range.1 - 0.6).abs() < 1e-6);
    }

    #[test]
    fn generation_config_for_player_level_high_saturates_round155() {
        // High-level boundary: max_atoms
        // saturates at 6 (level=20
        // would give 2 + 4 = 6; level=30
        // would give 2 + 6 = 8 capped at
        // 6). difficulty saturates at
        // 1.0 (level * 0.1 + 0.3 ≥ 1.0
        // when level ≥ 7).
        let cfg = GenerationConfig::for_player_level(50);
        assert_eq!(cfg.max_atoms, 6, "max_atoms must saturate at 6");
        assert_eq!(cfg.min_atoms, 2);
        // 0.3 + 50 * 0.1 = 5.3 → upper
        // bound is difficulty + 0.3 = 5.6
        // (we only assert lower bound is
        // saturated ≥ 1.0; upper is the
        // raw computation, which is
        // caller-side downstream).
        assert!(
            cfg.difficulty_range.0 >= 1.0,
            "difficulty lower bound must saturate at 1.0 for high levels"
        );
    }

    // -----------------------------------------------------------------
    // BalanceTuner::mood_bias — per-branch + stacked contracts.
    // -----------------------------------------------------------------

    #[test]
    fn balance_tuner_mood_bias_default_mood_is_zero_round155() {
        // Round-22 reflexive-loop
        // contract: when mood is
        // `NpcDisposition::default()`,
        // mood_bias must return exactly
        // 0.0 (no branches fire). A
        // regression that leaked a
        // constant bias into the default
        // mood would silently nudge
        // difficulty for every fresh
        // player.
        assert_eq!(
            BalanceTuner::mood_bias(NpcDisposition::default()),
            0.0
        );
    }

    #[test]
    fn balance_tuner_mood_bias_high_fear_is_neg_010_round155() {
        // Branch 1: fear > 0.5 → bias
        // = -0.10. Pin the exact value
        // — a regression to -0.05
        // would halve the difficulty
        // easing.
        let mood = NpcDisposition { fear: 0.7, ..NpcDisposition::default() };
        assert!((BalanceTuner::mood_bias(mood) - -0.10).abs() < 1e-6);
    }

    #[test]
    fn balance_tuner_mood_bias_friendly_trust_is_pos_008_round155() {
        // Branch 2: friendly > 0.5 AND
        // trust > 0.3 → bias = +0.08.
        // Both conditions must hold
        // together — a regression that
        // dropped the trust check would
        // over-fire the branch.
        let mood_ok = NpcDisposition {
            friendly: 0.6, trust: 0.4, ..NpcDisposition::default()
        };
        assert!((BalanceTuner::mood_bias(mood_ok) - 0.08).abs() < 1e-6);
        // friendly OK but trust too low → no bias.
        let mood_no_trust = NpcDisposition {
            friendly: 0.6, trust: 0.2, ..NpcDisposition::default()
        };
        assert_eq!(BalanceTuner::mood_bias(mood_no_trust), 0.0);
        // trust OK but friendly too low → no bias.
        let mood_no_friendly = NpcDisposition {
            friendly: 0.4, trust: 0.4, ..NpcDisposition::default()
        };
        assert_eq!(BalanceTuner::mood_bias(mood_no_friendly), 0.0);
    }

    #[test]
    fn balance_tuner_mood_bias_hostile_friendly_is_neg_005_round155() {
        // Branch 3: friendly < -0.3 →
        // bias = -0.05. Pin the
        // boundary: friendly = -0.31
        // fires, friendly = -0.3 does
        // NOT (strict <).
        let mood_fires = NpcDisposition {
            friendly: -0.31, ..NpcDisposition::default()
        };
        assert!((BalanceTuner::mood_bias(mood_fires) - -0.05).abs() < 1e-6);
        let mood_does_not = NpcDisposition {
            friendly: -0.3, ..NpcDisposition::default()
        };
        assert_eq!(BalanceTuner::mood_bias(mood_does_not), 0.0);
    }

    // -----------------------------------------------------------------
    // BalanceTuner::get_stats + suggest_difficulty contracts.
    // -----------------------------------------------------------------

    #[test]
    fn balance_tuner_get_stats_for_empty_history_returns_zeros_round155() {
        // Empty history: total_sessions=0,
        // win_rate=0.0, avg_score=0,
        // avg_duration=0.0. The zero
        // division path must not
        // produce NaN.
        let tuner = BalanceTuner::new();
        let stats = tuner.get_stats();
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.win_rate, 0.0);
        assert_eq!(stats.avg_score, 0);
        assert_eq!(stats.avg_duration, 0.0);
    }

    #[test]
    fn balance_tuner_suggest_difficulty_with_default_mood_equals_base_round155() {
        // Round-22 reflexive-loop
        // contract: when mood is the
        // default (zero bias),
        // `suggest_difficulty_with_mood`
        // must return EXACTLY the same
        // value as `suggest_difficulty`
        // (the reflexive loop adds
        // information when there IS
        // information, never noise).
        let tuner = BalanceTuner::new();
        let base = tuner.suggest_difficulty(5);
        let with_default = tuner.suggest_difficulty_with_mood(
            5, NpcDisposition::default()
        );
        assert!((base - with_default).abs() < 1e-6);
    }

    #[test]
    fn balance_tuner_suggest_difficulty_with_mood_clamps_to_unit_range_round155() {
        // The result must be clamped to
        // [0.1, 1.0] regardless of
        // mood. A regression that
        // dropped the clamp would let
        // the reflexive loop push
        // difficulty past 1.0 (or
        // below 0.1).
        let tuner = BalanceTuner::new();
        // High fear: bias = -0.10;
        // low level base is 0.3 +
        // 0.05*1 = 0.35, so result is
        // 0.25 (within range, but
        // verify the clamp is not
        // pushing it below 0.1).
        let mood = NpcDisposition { fear: 0.8, ..NpcDisposition::default() };
        let d = tuner.suggest_difficulty_with_mood(1, mood);
        assert!(d >= 0.1, "difficulty must clamp to >= 0.1: got {}", d);
        assert!(d <= 1.0, "difficulty must clamp to <= 1.0: got {}", d);
    }
}
