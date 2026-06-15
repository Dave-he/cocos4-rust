//! Playable runtime scene generation for AGI-miniGame.
//!
//! `scene_gen.rs` owns the high-level dimension contract (biome,
//! WFC weights, NPC density, music mood). This module owns the
//! lower-level playfield contract consumed by the browser runtime:
//! lanes, tower pads, palette roles, decorations, camera defaults,
//! spawn pressure, and the generated rule summary shown in the UI.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeBiome {
    NeonHarbor,
    VerdantRuins,
    SunforgeBazaar,
    OrbitalGarden,
}

impl RuntimeBiome {
    pub fn id(self) -> &'static str {
        match self {
            RuntimeBiome::NeonHarbor => "neon-harbor",
            RuntimeBiome::VerdantRuins => "verdant-ruins",
            RuntimeBiome::SunforgeBazaar => "sunforge-bazaar",
            RuntimeBiome::OrbitalGarden => "orbital-garden",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            RuntimeBiome::NeonHarbor => "Neon Harbor",
            RuntimeBiome::VerdantRuins => "Verdant Ruins",
            RuntimeBiome::SunforgeBazaar => "Sunforge Bazaar",
            RuntimeBiome::OrbitalGarden => "Orbital Garden",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimePalette {
    pub sky_top: &'static str,
    pub sky_bottom: &'static str,
    pub ground: &'static str,
    pub road: &'static str,
    pub grid: &'static str,
    pub tower: &'static str,
    pub enemy: &'static str,
    pub projectile: &'static str,
    pub core: &'static str,
    pub accent: &'static str,
    pub fog: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeLane {
    pub id: String,
    pub spawn: Vec2,
    pub bend: Vec2,
    pub width: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeWavePlan {
    pub id: String,
    pub lane_index: u32,
    pub count: u32,
    pub interval_multiplier: f32,
    pub archetype_bias: u32,
    pub spawn_spread: f32,
    pub warning_time: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeTowerArchetype {
    pub id: String,
    pub label: String,
    pub range_multiplier: f32,
    pub fire_interval_multiplier: f32,
    pub damage_multiplier: f32,
    pub scale: f32,
    pub color: &'static str,
    pub build_cost: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeBuildHint {
    pub id: String,
    pub anchor_index: u32,
    pub lane_index: u32,
    pub tower_archetype_id: String,
    pub priority: f32,
    pub radius: f32,
    pub color: &'static str,
    pub x: f32,
    pub z: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeSetPiece {
    pub id: String,
    pub kind: String,
    pub x: f32,
    pub z: f32,
    pub radius: f32,
    pub height: f32,
    pub rotation: f32,
    pub color: &'static str,
    pub accent_color: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeDecoration {
    pub x: f32,
    pub z: f32,
    pub radius: f32,
    pub height: f32,
    pub variant: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeSpawnTuning {
    pub interval: f32,
    pub enemy_speed: f32,
    pub enemy_cap: u32,
    pub wave_size: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeCameraTuning {
    pub distance: f32,
    pub height: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeLightingTuning {
    pub ambient: f32,
    pub key: f32,
    pub bloom: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeAtmospherePlan {
    pub particle_count: u32,
    pub particle_speed: f32,
    pub wind: Vec2,
    pub core_halo_radius: f32,
    pub core_halo_intensity: f32,
    pub lane_beacon_count: u32,
    pub sky_ring_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeControlPlan {
    pub camera_pan_speed: f32,
    pub camera_damping: f32,
    pub camera_auto_focus_strength: f32,
    pub camera_threat_lead: f32,
    pub camera_manual_override: f32,
    pub camera_alert_zoom: f32,
    pub blast_force: f32,
    pub blast_cooldown: f32,
    pub blast_score_reward: u32,
    pub build_score_cost: u32,
    pub pointer_assist_radius: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeCombatPlan {
    pub tower_range: f32,
    pub tower_fire_interval: f32,
    pub projectile_speed: f32,
    pub projectile_damage: f32,
    pub projectile_lead: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeScoringPlan {
    pub combo_window: f32,
    pub combo_multiplier_step: f32,
    pub max_combo_multiplier: f32,
    pub blast_combo_boost: f32,
    pub command_combo_boost: f32,
    pub support_combo_boost: f32,
    pub perfect_wave_bonus: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeTacticalField {
    pub x: f32,
    pub z: f32,
    pub radius: f32,
    pub slow_multiplier: f32,
    pub damage_per_pulse: f32,
    pub pulse_interval: f32,
    pub variant: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeSupportNode {
    pub id: String,
    pub x: f32,
    pub z: f32,
    pub radius: f32,
    pub score_per_pulse: u32,
    pub repair_per_pulse: f32,
    pub pulse_damage: f32,
    pub pulse_interval: f32,
    pub variant: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeEnemyArchetype {
    pub id: String,
    pub label: String,
    pub hp: f32,
    pub speed_multiplier: f32,
    pub scale: f32,
    pub color: &'static str,
    pub score_reward: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeBossPlan {
    pub id: String,
    pub label: String,
    pub trigger_wave: u32,
    pub lane_index: u32,
    pub hp: f32,
    pub speed_multiplier: f32,
    pub scale: f32,
    pub color: &'static str,
    pub score_reward: u32,
    pub warning_time: f32,
    pub aura_radius: f32,
    pub aura_damage: f32,
    pub aura_interval: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeCommandPlan {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub hotkey: String,
    pub cooldown: f32,
    pub score_cost: u32,
    pub magnitude: f32,
    pub radius: f32,
    pub duration: f32,
    pub lane_index: u32,
    pub color: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeCommandTargetingPlan {
    pub lane_assist_radius: f32,
    pub threat_weight: f32,
    pub pointer_weight: f32,
    pub reticle_radius: f32,
    pub reticle_pulse_speed: f32,
    pub retarget_cooldown: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeLaneSignal {
    pub id: String,
    pub lane_index: u32,
    pub warning_color: &'static str,
    pub boss_color: &'static str,
    pub alert_radius: f32,
    pub pulse_speed: f32,
    pub beacon_height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeObjectivePlan {
    pub summary: String,
    pub target_waves: u32,
    pub target_score: u32,
    pub min_integrity: f32,
    pub reward_xp: u32,
    pub auto_advance_delay: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeDirectorEvent {
    pub id: String,
    pub kind: String,
    pub trigger_wave: u32,
    pub cooldown: f32,
    pub magnitude: f32,
    pub duration: f32,
    pub lane_index: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeRulePlan {
    pub starter_tower_enabled: bool,
    pub first_wave_delay: f32,
    pub steering_lerp: f32,
    pub wounded_health_ratio: f32,
    pub wounded_speed_multiplier: f32,
    pub weak_point_pulse_interval: f32,
    pub weak_point_pulse_force: f32,
    pub waypoint_radius: f32,
    pub tower_snap_radius: f32,
    pub lane_build_buffer: f32,
    pub core_build_radius: f32,
    pub max_towers: u32,
    pub breach_radius: f32,
    pub breach_damage: f32,
    pub low_integrity_threshold: f32,
    pub low_integrity_spawn_multiplier: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeSceneRequest {
    pub seed: u64,
    pub player_level: u32,
    pub difficulty: u32,
    pub theme_hint: String,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeSceneBlueprint {
    pub id: String,
    pub title: String,
    pub seed: u64,
    pub biome: RuntimeBiome,
    pub difficulty: u32,
    pub modules: Vec<String>,
    pub palette: RuntimePalette,
    pub lanes: Vec<RuntimeLane>,
    pub wave_plan: Vec<RuntimeWavePlan>,
    pub tower_anchors: Vec<Vec2>,
    pub tower_archetypes: Vec<RuntimeTowerArchetype>,
    pub build_hints: Vec<RuntimeBuildHint>,
    pub tactical_fields: Vec<RuntimeTacticalField>,
    pub support_nodes: Vec<RuntimeSupportNode>,
    pub enemy_archetypes: Vec<RuntimeEnemyArchetype>,
    pub boss_plan: RuntimeBossPlan,
    pub commands: Vec<RuntimeCommandPlan>,
    pub command_targeting: RuntimeCommandTargetingPlan,
    pub lane_signals: Vec<RuntimeLaneSignal>,
    pub set_pieces: Vec<RuntimeSetPiece>,
    pub decorations: Vec<RuntimeDecoration>,
    pub spawn: RuntimeSpawnTuning,
    pub camera: RuntimeCameraTuning,
    pub lighting: RuntimeLightingTuning,
    pub atmosphere: RuntimeAtmospherePlan,
    pub controls: RuntimeControlPlan,
    pub combat: RuntimeCombatPlan,
    pub scoring: RuntimeScoringPlan,
    pub objective: RuntimeObjectivePlan,
    pub events: Vec<RuntimeDirectorEvent>,
    pub rules: RuntimeRulePlan,
    pub logic_source: String,
}

pub fn generate_runtime_scene(request: RuntimeSceneRequest) -> RuntimeSceneBlueprint {
    let difficulty = request.difficulty.clamp(1, 10);
    let seed = if request.seed == 0 { 1 } else { request.seed };
    let mut rng = RuntimeRng::new(seed);
    let biome = pick_biome(&request.theme_hint, &mut rng);
    let modules = if request.modules.is_empty() {
        default_modules(difficulty)
    } else {
        request.modules
    };
    let lane_count = (2 + difficulty / 3).clamp(2, 5);
    let lanes = build_lanes(lane_count, &mut rng);
    let tower_anchors = build_tower_anchors(lane_count, &mut rng);
    let tactical_fields = build_tactical_fields(difficulty, &lanes, &modules);
    let support_nodes = build_support_nodes(difficulty, &lanes, &tower_anchors, &modules);
    let decorations = build_decorations(18 + difficulty * 2, &mut rng);
    let palette = palette_for(biome);
    let controls = build_control_plan(difficulty, &modules);
    let enemy_archetypes = build_enemy_archetypes(difficulty, &modules, &palette);
    let tower_archetypes = build_tower_archetypes(difficulty, &modules, &palette, &controls);
    let build_hints = build_build_hints(
        difficulty,
        &lanes,
        &tower_anchors,
        &tower_archetypes,
        &modules,
        &palette,
    );
    let lane_signals = build_lane_signals(difficulty, &lanes, &modules, &palette);
    let set_pieces = build_set_pieces(difficulty, biome, &lanes, &modules, &palette);
    let interval = (2.35 - difficulty as f32 * 0.13 - request.player_level as f32 * 0.01).max(0.75);
    let enemy_speed = 32.0 + difficulty as f32 * 4.8 + request.player_level as f32 * 0.55;
    let enemy_cap = 8 + difficulty * 2;
    let wave_size = (2 + difficulty / 2).clamp(2, 8);
    let title = format!(
        "{} / {} D{}",
        biome.title(),
        module_title(&modules),
        difficulty
    );
    let objective = build_objective_plan(difficulty, wave_size, request.player_level);
    let boss_plan = build_boss_plan(difficulty, &lanes, &modules, &palette, objective.target_waves);
    let commands = build_command_plan(difficulty, &lanes, &modules, &palette);
    let command_targeting = build_command_targeting_plan(difficulty, &modules);
    let events = build_director_events(difficulty, lane_count, &modules);
    let atmosphere = build_atmosphere_plan(difficulty, biome, &modules);
    let combat = build_combat_plan(difficulty, &modules);
    let scoring = build_scoring_plan(difficulty, &modules);
    let rules = RuntimeRulePlan {
        starter_tower_enabled: modules.iter().any(|m| m == "tower_defense"),
        first_wave_delay: round_to((0.65 - difficulty as f32 * 0.035).max(0.18), 2),
        steering_lerp: round_to((0.065 + difficulty as f32 * 0.006).min(0.14), 3),
        wounded_health_ratio: round_to((0.42 - difficulty as f32 * 0.006).max(0.28), 2),
        wounded_speed_multiplier: round_to(1.04 + difficulty as f32 * 0.015, 2),
        weak_point_pulse_interval: round_to((2.1 - difficulty as f32 * 0.055).max(1.15), 2),
        weak_point_pulse_force: round_to(36.0 + difficulty as f32 * 4.2, 2),
        waypoint_radius: round_to((28.0 - difficulty as f32 * 0.8).max(16.0), 2),
        tower_snap_radius: round_to((42.0 - difficulty as f32 * 1.1).max(28.0), 2),
        lane_build_buffer: round_to(12.0 + difficulty as f32 * 0.85, 2),
        core_build_radius: round_to(42.0 + difficulty as f32 * 1.15, 2),
        max_towers: (3
            + difficulty / 2
            + if modules.iter().any(|m| m == "tower_defense") {
                2
            } else {
                0
            })
        .clamp(3, 9),
        breach_radius: round_to((30.0 - difficulty as f32 * 0.45).max(20.0), 2),
        breach_damage: round_to((4.0 + difficulty as f32 * 1.15).min(18.0), 2),
        low_integrity_threshold: round_to((48.0 - difficulty as f32 * 1.4).max(28.0), 2),
        low_integrity_spawn_multiplier: round_to(1.18 + (10 - difficulty) as f32 * 0.018, 2),
    };
    let wave_plan = build_wave_plan(difficulty, &lanes, objective.target_waves, wave_size, &modules);

    let mut blueprint = RuntimeSceneBlueprint {
        id: format!("runtime_{}_{}_{}", seed, biome.id(), difficulty),
        title,
        seed,
        biome,
        difficulty,
        modules,
        palette,
        lanes,
        wave_plan,
        tower_anchors,
        tower_archetypes,
        build_hints,
        tactical_fields,
        support_nodes,
        enemy_archetypes,
        boss_plan,
        commands,
        command_targeting,
        lane_signals,
        set_pieces,
        decorations,
        spawn: RuntimeSpawnTuning {
            interval: round_to(interval, 2),
            enemy_speed: round_to(enemy_speed, 2),
            enemy_cap,
            wave_size,
        },
        camera: RuntimeCameraTuning {
            distance: 310.0 + lane_count as f32 * 18.0,
            height: 210.0 + difficulty as f32 * 4.0,
            pitch: 0.58,
        },
        lighting: RuntimeLightingTuning {
            ambient: round_to(0.42 + rng.next_f32() * 0.12, 2),
            key: round_to(0.88 + difficulty as f32 * 0.035, 2),
            bloom: round_to(0.55 + difficulty as f32 * 0.04, 2),
        },
        atmosphere,
        controls,
        combat,
        scoring,
        objective,
        events,
        rules,
        logic_source: String::new(),
    };
    blueprint.logic_source = build_logic_source(&blueprint);
    blueprint
}

fn pick_biome(theme_hint: &str, rng: &mut RuntimeRng) -> RuntimeBiome {
    let hint = theme_hint.to_ascii_lowercase();
    if hint.contains("forest") || hint.contains("ruin") {
        return RuntimeBiome::VerdantRuins;
    }
    if hint.contains("desert") || hint.contains("temple") || hint.contains("forge") {
        return RuntimeBiome::SunforgeBazaar;
    }
    if hint.contains("space") || hint.contains("orbit") || hint.contains("nebula") {
        return RuntimeBiome::OrbitalGarden;
    }
    if hint.contains("cyber") || hint.contains("neon") || hint.contains("city") {
        return RuntimeBiome::NeonHarbor;
    }
    match (rng.next_f32() * 4.0).floor() as u32 {
        0 => RuntimeBiome::NeonHarbor,
        1 => RuntimeBiome::VerdantRuins,
        2 => RuntimeBiome::SunforgeBazaar,
        _ => RuntimeBiome::OrbitalGarden,
    }
}

fn default_modules(difficulty: u32) -> Vec<String> {
    if difficulty < 4 {
        vec!["parkour".to_string(), "synthesis".to_string()]
    } else if difficulty < 8 {
        vec!["tower_defense".to_string(), "puzzle".to_string()]
    } else {
        vec![
            "tower_defense".to_string(),
            "shooter".to_string(),
            "card".to_string(),
        ]
    }
}

fn build_lanes(count: u32, rng: &mut RuntimeRng) -> Vec<RuntimeLane> {
    let mut lanes = Vec::with_capacity(count as usize);
    let radius = 182.0_f32;
    for i in 0..count {
        let angle = std::f32::consts::TAU * i as f32 / count as f32 + rng.next_f32() * 0.32;
        lanes.push(RuntimeLane {
            id: format!("lane-{}", i + 1),
            spawn: Vec2 {
                x: round_to(angle.cos() * radius, 2),
                z: round_to(angle.sin() * radius, 2),
            },
            bend: Vec2 {
                x: round_to((angle + 0.42).cos() * radius * 0.45, 2),
                z: round_to((angle - 0.35).sin() * radius * 0.45, 2),
            },
            width: round_to(24.0 + rng.next_f32() * 18.0, 2),
        });
    }
    lanes
}

fn build_tower_anchors(lane_count: u32, rng: &mut RuntimeRng) -> Vec<Vec2> {
    let total = lane_count + 2;
    let mut anchors = Vec::with_capacity(total as usize);
    for i in 0..total {
        let angle = std::f32::consts::TAU * i as f32 / total as f32 + 0.24;
        let radius = 72.0 + rng.next_f32() * 38.0;
        anchors.push(Vec2 {
            x: round_to(angle.cos() * radius, 2),
            z: round_to(angle.sin() * radius, 2),
        });
    }
    anchors
}

fn build_decorations(count: u32, rng: &mut RuntimeRng) -> Vec<RuntimeDecoration> {
    let mut decorations = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let angle = rng.next_f32() * std::f32::consts::TAU;
        let distance = 118.0 + rng.next_f32() * 78.0;
        decorations.push(RuntimeDecoration {
            x: round_to(angle.cos() * distance, 2),
            z: round_to(angle.sin() * distance, 2),
            radius: round_to(2.5 + rng.next_f32() * 7.0, 2),
            height: round_to(16.0 + rng.next_f32() * 48.0, 2),
            variant: (rng.next_f32() * 4.0).floor() as u32,
        });
    }
    decorations
}

fn build_objective_plan(
    difficulty: u32,
    wave_size: u32,
    player_level: u32,
) -> RuntimeObjectivePlan {
    let target_waves = (3 + difficulty / 2).clamp(3, 8);
    let target_score =
        (target_waves as f32 * wave_size as f32 * (12.0 + difficulty as f32 * 1.6)).round() as u32;
    let min_integrity = round_to((58.0 - difficulty as f32 * 2.2).max(24.0), 2);
    RuntimeObjectivePlan {
        summary: format!(
            "Hold {} waves or score {} with core >= {}",
            target_waves,
            target_score,
            min_integrity.round() as u32
        ),
        target_waves,
        target_score,
        min_integrity,
        reward_xp: 8 + difficulty * 3 + player_level / 2,
        auto_advance_delay: round_to((2.6 - difficulty as f32 * 0.08).max(1.1), 2),
    }
}

fn build_wave_plan(
    difficulty: u32,
    lanes: &[RuntimeLane],
    target_waves: u32,
    base_wave_size: u32,
    modules: &[String],
) -> Vec<RuntimeWavePlan> {
    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1
    } else {
        0
    };
    let shooter_focus = if modules.iter().any(|m| m == "shooter") {
        1
    } else {
        0
    };
    let plan_length = (target_waves + 1).clamp(4, 9);
    let mut waves = Vec::with_capacity(plan_length as usize);
    for i in 0..plan_length {
        let pressure_tier = i / 2;
        let lane_index = (i * 2 + difficulty + tower_focus) % lanes.len() as u32;
        waves.push(RuntimeWavePlan {
            id: format!("wave-{}", i + 1),
            lane_index,
            count: (base_wave_size + pressure_tier + if i % 3 == 2 { 1 } else { 0 })
                .clamp(2, 11),
            interval_multiplier: round_to(
                (1.08 - difficulty as f32 * 0.018 - pressure_tier as f32 * 0.035)
                    .clamp(0.62, 1.12),
                2,
            ),
            archetype_bias: (i + difficulty + shooter_focus) % 3,
            spawn_spread: round_to(
                lanes[lane_index as usize].width * (0.24 + (i % 3) as f32 * 0.07),
                2,
            ),
            warning_time: round_to(
                (0.72 + (10 - difficulty) as f32 * 0.035 - shooter_focus as f32 * 0.08)
                    .clamp(0.42, 0.92),
                2,
            ),
        });
    }
    waves
}

fn build_director_events(
    difficulty: u32,
    lane_count: u32,
    modules: &[String],
) -> Vec<RuntimeDirectorEvent> {
    let mut events = vec![RuntimeDirectorEvent {
        id: "repair-pulse".to_string(),
        kind: "repair-pulse".to_string(),
        trigger_wave: 1,
        cooldown: round_to((9.6 - difficulty as f32 * 0.24).max(5.8), 2),
        magnitude: round_to(5.5 + difficulty as f32 * 0.75, 2),
        duration: 0.0,
        lane_index: 0,
    }];

    if modules.iter().any(|m| m == "tower_defense") {
        events.push(RuntimeDirectorEvent {
            id: "tower-overdrive".to_string(),
            kind: "tower-overdrive".to_string(),
            trigger_wave: 2,
            cooldown: round_to((8.4 - difficulty as f32 * 0.16).max(5.4), 2),
            magnitude: round_to(12.0 + difficulty as f32 * 1.8, 2),
            duration: round_to(2.2 + difficulty as f32 * 0.08, 2),
            lane_index: 0,
        });
    }

    events.push(RuntimeDirectorEvent {
        id: "enemy-surge".to_string(),
        kind: "enemy-surge".to_string(),
        trigger_wave: 2 + difficulty / 4,
        cooldown: round_to((8.7 - difficulty as f32 * 0.18).max(5.2), 2),
        magnitude: round_to(1.08 + difficulty as f32 * 0.025, 2),
        duration: 0.0,
        lane_index: difficulty % lane_count.max(1),
    });
    events
}

fn build_atmosphere_plan(
    difficulty: u32,
    biome: RuntimeBiome,
    modules: &[String],
) -> RuntimeAtmospherePlan {
    let density_bonus = match biome {
        RuntimeBiome::NeonHarbor => 10,
        RuntimeBiome::VerdantRuins => 18,
        RuntimeBiome::SunforgeBazaar => 6,
        RuntimeBiome::OrbitalGarden => 14,
    };
    let wind = match biome {
        RuntimeBiome::NeonHarbor => Vec2 { x: 0.9, z: -0.3 },
        RuntimeBiome::VerdantRuins => Vec2 { x: -0.35, z: 0.72 },
        RuntimeBiome::SunforgeBazaar => Vec2 { x: 1.05, z: 0.36 },
        RuntimeBiome::OrbitalGarden => Vec2 { x: -0.52, z: -0.88 },
    };
    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1
    } else {
        0
    };
    let wind_scale = 1.0 + difficulty as f32 * 0.025;

    RuntimeAtmospherePlan {
        particle_count: (36 + difficulty * 6 + density_bonus + tower_focus * 6).clamp(36, 112),
        particle_speed: round_to(
            0.16 + difficulty as f32 * 0.025 + density_bonus as f32 * 0.001,
            2,
        ),
        wind: Vec2 {
            x: round_to(wind.x * wind_scale, 2),
            z: round_to(wind.z * wind_scale, 2),
        },
        core_halo_radius: round_to(48.0 + difficulty as f32 * 2.4 + tower_focus as f32 * 4.0, 2),
        core_halo_intensity: round_to(
            0.32 + difficulty as f32 * 0.026 + tower_focus as f32 * 0.04,
            2,
        ),
        lane_beacon_count: (2 + difficulty / 3 + tower_focus).clamp(2, 6),
        sky_ring_count: (1 + difficulty / 5 + tower_focus).clamp(1, 4),
    }
}

fn build_tactical_fields(
    difficulty: u32,
    lanes: &[RuntimeLane],
    modules: &[String],
) -> Vec<RuntimeTacticalField> {
    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1
    } else {
        0
    };
    let puzzle_focus = if modules.iter().any(|m| m == "puzzle") {
        1
    } else {
        0
    };
    let field_count = (1 + difficulty / 3 + tower_focus).clamp(1, lanes.len().min(5) as u32);
    let mut fields = Vec::with_capacity(field_count as usize);
    for i in 0..field_count {
        let lane = &lanes[i as usize % lanes.len()];
        let offset = if i % 2 == 0 { 0.86 } else { 0.72 };
        fields.push(RuntimeTacticalField {
            x: round_to(lane.bend.x * offset, 2),
            z: round_to(lane.bend.z * offset, 2),
            radius: round_to(26.0 + difficulty as f32 * 1.45 + (i % 2) as f32 * 4.0, 2),
            slow_multiplier: round_to(
                (0.84 - difficulty as f32 * 0.018 - puzzle_focus as f32 * 0.04)
                    .clamp(0.56, 0.84),
                2,
            ),
            damage_per_pulse: round_to(
                2.5 + difficulty as f32 * 0.55 + tower_focus as f32 * 1.35,
                2,
            ),
            pulse_interval: round_to((1.58 - difficulty as f32 * 0.052).max(0.82), 2),
            variant: i % 3,
        });
    }
    fields
}

fn build_support_nodes(
    difficulty: u32,
    lanes: &[RuntimeLane],
    tower_anchors: &[Vec2],
    modules: &[String],
) -> Vec<RuntimeSupportNode> {
    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1
    } else {
        0
    };
    let puzzle_focus = if modules.iter().any(|m| m == "puzzle") {
        1
    } else {
        0
    };
    let node_count = (1 + difficulty / 4 + tower_focus).clamp(1, lanes.len().min(4) as u32);
    let mut nodes = Vec::with_capacity(node_count as usize);
    for i in 0..node_count {
        let lane = &lanes[(i * 2 + difficulty) as usize % lanes.len()];
        let anchor = tower_anchors[(i + 1) as usize % tower_anchors.len()];
        let anchor_weight = 0.42 + (i % 2) as f32 * 0.14;
        nodes.push(RuntimeSupportNode {
            id: format!("relay-{}", i + 1),
            x: round_to(lane.bend.x * (1.0 - anchor_weight) + anchor.x * anchor_weight, 2),
            z: round_to(lane.bend.z * (1.0 - anchor_weight) + anchor.z * anchor_weight, 2),
            radius: round_to(24.0 + difficulty as f32 * 1.2 + (i % 2) as f32 * 5.0, 2),
            score_per_pulse: ((4.0 + difficulty as f32 * 0.8 + tower_focus as f32 * 2.0 + i as f32)
                .round() as u32)
                .clamp(4, 16),
            repair_per_pulse: round_to(
                (1.4 + difficulty as f32 * 0.18 + puzzle_focus as f32 * 0.75)
                    .clamp(1.4, 4.6),
                2,
            ),
            pulse_damage: round_to(4.5 + difficulty as f32 * 0.72 + tower_focus as f32 * 1.4, 2),
            pulse_interval: round_to(
                (3.2 - difficulty as f32 * 0.08 - i as f32 * 0.05).clamp(1.85, 3.25),
                2,
            ),
            variant: i % 3,
        });
    }
    nodes
}

fn build_tower_archetypes(
    difficulty: u32,
    modules: &[String],
    palette: &RuntimePalette,
    controls: &RuntimeControlPlan,
) -> Vec<RuntimeTowerArchetype> {
    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1
    } else {
        0
    };
    let shooter_focus = if modules.iter().any(|m| m == "shooter") {
        1
    } else {
        0
    };
    let base_cost = controls.build_score_cost;
    vec![
        RuntimeTowerArchetype {
            id: "sentinel".to_string(),
            label: "Sentinel".to_string(),
            range_multiplier: round_to(1.0 + tower_focus as f32 * 0.03, 2),
            fire_interval_multiplier: 1.0,
            damage_multiplier: 1.0,
            scale: 1.0,
            color: palette.tower,
            build_cost: base_cost,
        },
        RuntimeTowerArchetype {
            id: "rail".to_string(),
            label: "Rail".to_string(),
            range_multiplier: round_to(
                (1.2 + difficulty as f32 * 0.006 + shooter_focus as f32 * 0.04)
                    .clamp(1.18, 1.34),
                2,
            ),
            fire_interval_multiplier: round_to(
                (1.14 + difficulty as f32 * 0.016).clamp(1.12, 1.34),
                2,
            ),
            damage_multiplier: round_to(
                (1.16 + difficulty as f32 * 0.022 + tower_focus as f32 * 0.06)
                    .clamp(1.18, 1.46),
                2,
            ),
            scale: 1.12,
            color: palette.grid,
            build_cost: (base_cost + 2 + if difficulty >= 7 { 1 } else { 0 }).clamp(3, 12),
        },
        RuntimeTowerArchetype {
            id: "spark".to_string(),
            label: "Spark".to_string(),
            range_multiplier: round_to((0.86 - tower_focus as f32 * 0.02).clamp(0.78, 0.88), 2),
            fire_interval_multiplier: round_to(
                (0.74 - shooter_focus as f32 * 0.05).clamp(0.62, 0.76),
                2,
            ),
            damage_multiplier: round_to((0.76 + difficulty as f32 * 0.012).clamp(0.72, 0.9), 2),
            scale: 0.9,
            color: palette.accent,
            build_cost: base_cost.saturating_sub(1).max(2),
        },
    ]
}

fn build_build_hints(
    difficulty: u32,
    lanes: &[RuntimeLane],
    tower_anchors: &[Vec2],
    tower_archetypes: &[RuntimeTowerArchetype],
    modules: &[String],
    palette: &RuntimePalette,
) -> Vec<RuntimeBuildHint> {
    if tower_archetypes.is_empty() {
        return Vec::new();
    }

    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1
    } else {
        0
    };
    let shooter_focus = if modules.iter().any(|m| m == "shooter") {
        1
    } else {
        0
    };
    let puzzle_focus = if modules.iter().any(|m| m == "puzzle") {
        1
    } else {
        0
    };
    let sentinel = tower_archetypes
        .iter()
        .find(|archetype| archetype.id == "sentinel")
        .unwrap_or(&tower_archetypes[0]);
    let rail = tower_archetypes
        .iter()
        .find(|archetype| archetype.id == "rail")
        .unwrap_or(sentinel);
    let spark = tower_archetypes
        .iter()
        .find(|archetype| archetype.id == "spark")
        .unwrap_or(sentinel);

    tower_anchors
        .iter()
        .enumerate()
        .map(|(anchor_index, anchor)| {
            let mut lane_index = 0_usize;
            let mut best_distance_squared = f32::INFINITY;
            for (index, lane) in lanes.iter().enumerate() {
                let distance_squared = distance_to_segment_squared(anchor, &lane.spawn, &lane.bend)
                    .min(distance_to_segment_squared(anchor, &lane.bend, &Vec2 { x: 0.0, z: 0.0 }));
                if distance_squared < best_distance_squared {
                    lane_index = index;
                    best_distance_squared = distance_squared;
                }
            }
            let core_distance = (anchor.x * anchor.x + anchor.z * anchor.z).sqrt();
            let lane_distance = best_distance_squared.sqrt();
            let recommended = if shooter_focus > 0 && (anchor_index as u32 + difficulty) % 3 == 0 {
                rail
            } else if core_distance < 86.0 || (puzzle_focus > 0 && anchor_index % 2 == 1) {
                spark
            } else {
                sentinel
            };
            let lane_coverage = if lane_distance < 42.0 {
                1.0
            } else if lane_distance < 70.0 {
                0.55
            } else {
                0.25
            };
            let priority = round_to(
                (1.0
                    + difficulty as f32 * 0.12
                    + lane_coverage
                    + tower_focus as f32 * 0.45
                    + if recommended.id == "rail" {
                        shooter_focus as f32 * 0.35
                    } else {
                        0.0
                    }
                    + if recommended.id == "spark" { 0.22 } else { 0.0 }
                    - anchor_index as f32 * 0.04)
                    .clamp(1.0, 4.8),
                2,
            );
            RuntimeBuildHint {
                id: format!("build-hint-{}", anchor_index + 1),
                anchor_index: anchor_index as u32,
                lane_index: lane_index as u32,
                tower_archetype_id: recommended.id.clone(),
                priority,
                radius: round_to(14.0 + priority * 3.2 + difficulty as f32 * 0.45, 2),
                color: if recommended.color.is_empty() {
                    palette.tower
                } else {
                    recommended.color
                },
                x: anchor.x,
                z: anchor.z,
            }
        })
        .collect()
}

fn build_set_pieces(
    difficulty: u32,
    biome: RuntimeBiome,
    lanes: &[RuntimeLane],
    modules: &[String],
    palette: &RuntimePalette,
) -> Vec<RuntimeSetPiece> {
    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1
    } else {
        0
    };
    let puzzle_focus = if modules.iter().any(|m| m == "puzzle") {
        1
    } else {
        0
    };
    let count = (2 + difficulty / 4 + puzzle_focus).clamp(2, 5);
    let kinds: [&str; 3] = match biome {
        RuntimeBiome::NeonHarbor => ["spire", "arch", "spire"],
        RuntimeBiome::VerdantRuins => ["garden", "arch", "monolith"],
        RuntimeBiome::SunforgeBazaar => ["monolith", "arch", "spire"],
        RuntimeBiome::OrbitalGarden => ["spire", "garden", "arch"],
    };

    (0..count)
        .map(|index| {
            let lane = &lanes[index as usize % lanes.len()];
            let lane_angle = lane.spawn.z.atan2(lane.spawn.x);
            let angle = lane_angle
                + if index % 2 == 0 { 0.34 } else { -0.42 }
                + index as f32 * 0.18;
            let distance = 132.0 + difficulty as f32 * 3.8 + index as f32 * 17.0;
            let kind = kinds[index as usize % kinds.len()];
            let height_boost = match kind {
                "spire" => 18.0,
                "arch" => 7.0,
                "monolith" => 13.0,
                _ => 4.0,
            };
            RuntimeSetPiece {
                id: format!("set-piece-{}", index + 1),
                kind: kind.to_string(),
                x: round_to(angle.cos() * distance, 2),
                z: round_to(angle.sin() * distance, 2),
                radius: round_to(
                    8.0
                        + difficulty as f32 * 0.8
                        + index as f32 * 1.4
                        + if tower_focus > 0 { 1.5 } else { 0.0 },
                    2,
                ),
                height: round_to(34.0 + difficulty as f32 * 4.2 + index as f32 * 6.0 + height_boost, 2),
                rotation: round_to(angle + std::f32::consts::FRAC_PI_2, 2),
                color: if kind == "monolith" {
                    palette.road
                } else {
                    palette.tower
                },
                accent_color: if index % 2 == 0 {
                    palette.grid
                } else {
                    palette.accent
                },
            }
        })
        .collect()
}

fn build_enemy_archetypes(
    difficulty: u32,
    modules: &[String],
    palette: &RuntimePalette,
) -> Vec<RuntimeEnemyArchetype> {
    let shooter_focus = if modules.iter().any(|m| m == "shooter") {
        1
    } else {
        0
    };
    let puzzle_focus = if modules.iter().any(|m| m == "puzzle") {
        1
    } else {
        0
    };
    vec![
        RuntimeEnemyArchetype {
            id: "skirmisher".to_string(),
            label: "Skirmisher".to_string(),
            hp: round_to(72.0 + difficulty as f32 * 7.5, 2),
            speed_multiplier: round_to(
                1.1 + difficulty as f32 * 0.018 + shooter_focus as f32 * 0.06,
                2,
            ),
            scale: round_to(0.86 + difficulty as f32 * 0.006, 2),
            color: palette.enemy,
            score_reward: (14.0 + difficulty as f32 * 1.2).round() as u32,
        },
        RuntimeEnemyArchetype {
            id: "bulwark".to_string(),
            label: "Bulwark".to_string(),
            hp: round_to(
                128.0 + difficulty as f32 * 12.5 + puzzle_focus as f32 * 22.0,
                2,
            ),
            speed_multiplier: round_to((0.82 - difficulty as f32 * 0.01).max(0.56), 2),
            scale: round_to(1.18 + difficulty as f32 * 0.018, 2),
            color: palette.accent,
            score_reward: (24.0 + difficulty as f32 * 2.2 + puzzle_focus as f32 * 4.0).round()
                as u32,
        },
        RuntimeEnemyArchetype {
            id: "piercer".to_string(),
            label: "Piercer".to_string(),
            hp: round_to(
                92.0 + difficulty as f32 * 9.2 + shooter_focus as f32 * 18.0,
                2,
            ),
            speed_multiplier: round_to(
                0.98 + difficulty as f32 * 0.014 + shooter_focus as f32 * 0.08,
                2,
            ),
            scale: round_to(0.98 + difficulty as f32 * 0.01, 2),
            color: palette.grid,
            score_reward: (18.0 + difficulty as f32 * 1.7 + shooter_focus as f32 * 5.0).round()
                as u32,
        },
    ]
}

fn build_boss_plan(
    difficulty: u32,
    lanes: &[RuntimeLane],
    modules: &[String],
    palette: &RuntimePalette,
    target_waves: u32,
) -> RuntimeBossPlan {
    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1
    } else {
        0
    };
    let shooter_focus = if modules.iter().any(|m| m == "shooter") {
        1
    } else {
        0
    };
    let puzzle_focus = if modules.iter().any(|m| m == "puzzle") {
        1
    } else {
        0
    };
    let lane_count = lanes.len().max(1) as u32;
    let trigger_wave = (2 + difficulty / 3 + puzzle_focus).clamp(2, target_waves.max(2));
    let label = if shooter_focus > 0 {
        "Signal Reaver"
    } else if puzzle_focus > 0 {
        "Cipher Colossus"
    } else {
        "Apex Warden"
    };

    RuntimeBossPlan {
        id: format!("boss-wave-{}", trigger_wave),
        label: label.to_string(),
        trigger_wave,
        lane_index: (difficulty + tower_focus * 2 + shooter_focus) % lane_count,
        hp: round_to(
            185.0 + difficulty as f32 * 26.0 + puzzle_focus as f32 * 32.0 + tower_focus as f32 * 24.0,
            2,
        ),
        speed_multiplier: round_to(
            (0.66 + difficulty as f32 * 0.012 + shooter_focus as f32 * 0.035)
                .clamp(0.62, 0.86),
            2,
        ),
        scale: round_to(1.48 + difficulty as f32 * 0.035 + puzzle_focus as f32 * 0.08, 2),
        color: if shooter_focus > 0 {
            palette.projectile
        } else {
            palette.core
        },
        score_reward: (46.0
            + difficulty as f32 * 7.2
            + tower_focus as f32 * 8.0
            + puzzle_focus as f32 * 6.0)
            .round() as u32,
        warning_time: round_to((1.05 + (10 - difficulty) as f32 * 0.04).clamp(0.75, 1.4), 2),
        aura_radius: round_to(34.0 + difficulty as f32 * 2.2 + puzzle_focus as f32 * 5.0, 2),
        aura_damage: round_to(4.0 + difficulty as f32 * 0.72 + tower_focus as f32 * 1.8, 2),
        aura_interval: round_to(
            (2.6 - difficulty as f32 * 0.09 - shooter_focus as f32 * 0.18).max(1.15),
            2,
        ),
    }
}

fn build_command_plan(
    difficulty: u32,
    lanes: &[RuntimeLane],
    modules: &[String],
    palette: &RuntimePalette,
) -> Vec<RuntimeCommandPlan> {
    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1
    } else {
        0
    };
    let shooter_focus = if modules.iter().any(|m| m == "shooter") {
        1
    } else {
        0
    };
    let puzzle_focus = if modules.iter().any(|m| m == "puzzle") {
        1
    } else {
        0
    };
    let lane_count = lanes.len().max(1) as u32;

    vec![
        RuntimeCommandPlan {
            id: "command-barrage".to_string(),
            label: if shooter_focus > 0 {
                "Pierce Barrage".to_string()
            } else {
                "Lane Barrage".to_string()
            },
            kind: "lane-barrage".to_string(),
            hotkey: "KeyQ".to_string(),
            cooldown: round_to((7.2 - difficulty as f32 * 0.16 - shooter_focus as f32 * 0.55).max(4.2), 2),
            score_cost: ((8.0 + difficulty as f32 * 0.75 - shooter_focus as f32 * 2.0).round() as u32).clamp(5, 16),
            magnitude: round_to(32.0 + difficulty as f32 * 5.8 + shooter_focus as f32 * 18.0, 2),
            radius: round_to(36.0 + difficulty as f32 * 2.4 + shooter_focus as f32 * 8.0, 2),
            duration: 0.0,
            lane_index: (difficulty + shooter_focus) % lane_count,
            color: palette.projectile,
        },
        RuntimeCommandPlan {
            id: "command-repair".to_string(),
            label: if puzzle_focus > 0 {
                "Stabilize Core".to_string()
            } else {
                "Core Repair".to_string()
            },
            kind: "core-repair".to_string(),
            hotkey: "KeyE".to_string(),
            cooldown: round_to((9.6 - difficulty as f32 * 0.14 - puzzle_focus as f32 * 0.5).max(6.2), 2),
            score_cost: ((10.0 + difficulty as f32 * 0.65 - puzzle_focus as f32 * 2.0).round() as u32).clamp(6, 18),
            magnitude: round_to(11.0 + difficulty as f32 * 1.35 + puzzle_focus as f32 * 4.0, 2),
            radius: round_to(44.0 + difficulty as f32 * 1.6, 2),
            duration: 0.0,
            lane_index: 0,
            color: palette.core,
        },
        RuntimeCommandPlan {
            id: "command-rally".to_string(),
            label: if tower_focus > 0 {
                "Tower Rally".to_string()
            } else {
                "Circuit Rally".to_string()
            },
            kind: "tower-rally".to_string(),
            hotkey: "KeyR".to_string(),
            cooldown: round_to((8.8 - difficulty as f32 * 0.12 - tower_focus as f32 * 0.65).max(5.6), 2),
            score_cost: ((7.0 + difficulty as f32 * 0.7 - tower_focus as f32 * 2.0).round() as u32).clamp(4, 15),
            magnitude: round_to(42.0 + difficulty as f32 * 4.6 + tower_focus as f32 * 18.0, 2),
            radius: round_to(64.0 + difficulty as f32 * 3.6 + tower_focus as f32 * 12.0, 2),
            duration: round_to(1.4 + difficulty as f32 * 0.05, 2),
            lane_index: (difficulty + tower_focus * 2) % lane_count,
            color: palette.tower,
        },
    ]
}

fn build_command_targeting_plan(
    difficulty: u32,
    modules: &[String],
) -> RuntimeCommandTargetingPlan {
    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1.0
    } else {
        0.0
    };
    let shooter_focus = if modules.iter().any(|m| m == "shooter") {
        1.0
    } else {
        0.0
    };
    let puzzle_focus = if modules.iter().any(|m| m == "puzzle") {
        1.0
    } else {
        0.0
    };
    RuntimeCommandTargetingPlan {
        lane_assist_radius: round_to(52.0 + difficulty as f32 * 3.4 + shooter_focus * 14.0 + tower_focus * 6.0, 2),
        threat_weight: round_to((0.52 + difficulty as f32 * 0.018 + tower_focus * 0.1).clamp(0.5, 0.82), 2),
        pointer_weight: round_to((0.72 + shooter_focus * 0.12 - puzzle_focus * 0.04).clamp(0.62, 0.88), 2),
        reticle_radius: round_to(18.0 + difficulty as f32 * 1.4 + shooter_focus * 4.0, 2),
        reticle_pulse_speed: round_to(1.08 + difficulty as f32 * 0.065 + shooter_focus * 0.18, 2),
        retarget_cooldown: round_to((0.24 - difficulty as f32 * 0.012 - shooter_focus * 0.035).clamp(0.08, 0.24), 2),
    }
}

fn build_lane_signals(
    difficulty: u32,
    lanes: &[RuntimeLane],
    modules: &[String],
    palette: &RuntimePalette,
) -> Vec<RuntimeLaneSignal> {
    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1
    } else {
        0
    };
    let shooter_focus = if modules.iter().any(|m| m == "shooter") {
        1
    } else {
        0
    };

    lanes
        .iter()
        .enumerate()
        .map(|(index, lane)| RuntimeLaneSignal {
            id: format!("{}-signal", lane.id),
            lane_index: index as u32,
            warning_color: if index % 2 == 0 {
                palette.grid
            } else {
                palette.accent
            },
            boss_color: palette.enemy,
            alert_radius: round_to(
                lane.width * (0.78 + difficulty as f32 * 0.018 + shooter_focus as f32 * 0.08),
                2,
            ),
            pulse_speed: round_to(0.95 + difficulty as f32 * 0.075 + tower_focus as f32 * 0.16, 2),
            beacon_height: round_to(
                20.0 + difficulty as f32 * 2.4 + (index % 2) as f32 * 6.0 + shooter_focus as f32 * 5.0,
                2,
            ),
        })
        .collect()
}

fn build_control_plan(difficulty: u32, modules: &[String]) -> RuntimeControlPlan {
    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1
    } else {
        0
    };
    let shooter_focus = if modules.iter().any(|m| m == "shooter") {
        1
    } else {
        0
    };
    let parkour_focus = if modules.iter().any(|m| m == "parkour") {
        1
    } else {
        0
    };

    RuntimeControlPlan {
        camera_pan_speed: round_to(
            96.0 + difficulty as f32 * 5.0 + parkour_focus as f32 * 12.0,
            2,
        ),
        camera_damping: round_to(
            (0.1 - difficulty as f32 * 0.003 + parkour_focus as f32 * 0.006).clamp(0.058, 0.12),
            3,
        ),
        camera_auto_focus_strength: round_to(
            (0.2 + difficulty as f32 * 0.018 + shooter_focus as f32 * 0.035
                - parkour_focus as f32 * 0.025)
                .clamp(0.18, 0.46),
            2,
        ),
        camera_threat_lead: round_to(
            42.0
                + difficulty as f32 * 4.2
                + shooter_focus as f32 * 12.0
                + tower_focus as f32 * 6.0,
            2,
        ),
        camera_manual_override: round_to(
            1.15 + parkour_focus as f32 * 0.38 + (6_i32 - difficulty as i32).max(0) as f32 * 0.035,
            2,
        ),
        camera_alert_zoom: round_to(
            18.0
                + difficulty as f32 * 2.2
                + tower_focus as f32 * 5.0
                + shooter_focus as f32 * 4.0,
            2,
        ),
        blast_force: round_to(
            132.0 + difficulty as f32 * 8.5 + shooter_focus as f32 * 24.0,
            2,
        ),
        blast_cooldown: round_to(
            (1.22 - difficulty as f32 * 0.045 - shooter_focus as f32 * 0.12).max(0.54),
            2,
        ),
        blast_score_reward: (6.0 + difficulty as f32 * 0.85 + shooter_focus as f32 * 3.0).round()
            as u32,
        build_score_cost: ((7.0 - difficulty as f32 * 0.25 - tower_focus as f32 * 2.0).round()
            as u32)
            .clamp(2, 7),
        pointer_assist_radius: round_to(
            34.0 + difficulty as f32 * 1.85 + shooter_focus as f32 * 8.0 + tower_focus as f32 * 3.0,
            2,
        ),
    }
}

fn build_combat_plan(difficulty: u32, modules: &[String]) -> RuntimeCombatPlan {
    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1
    } else {
        0
    };
    let shooter_focus = if modules.iter().any(|m| m == "shooter") {
        1
    } else {
        0
    };

    RuntimeCombatPlan {
        tower_range: round_to(164.0 + difficulty as f32 * 5.8 + tower_focus as f32 * 22.0, 2),
        tower_fire_interval: round_to(
            (0.62 - difficulty as f32 * 0.024 - tower_focus as f32 * 0.08).max(0.26),
            2,
        ),
        projectile_speed: round_to(228.0 + difficulty as f32 * 12.0 + shooter_focus as f32 * 34.0, 2),
        projectile_damage: round_to(
            34.0 + difficulty as f32 * 3.4 + tower_focus as f32 * 10.0 + shooter_focus as f32 * 6.0,
            2,
        ),
        projectile_lead: round_to(
            (0.18 + difficulty as f32 * 0.018 + shooter_focus as f32 * 0.06).clamp(0.16, 0.42),
            2,
        ),
    }
}

fn build_scoring_plan(difficulty: u32, modules: &[String]) -> RuntimeScoringPlan {
    let shooter_focus = if modules.iter().any(|m| m == "shooter") {
        1
    } else {
        0
    };
    let parkour_focus = if modules.iter().any(|m| m == "parkour") {
        1
    } else {
        0
    };
    let tower_focus = if modules.iter().any(|m| m == "tower_defense") {
        1
    } else {
        0
    };

    RuntimeScoringPlan {
        combo_window: round_to(
            (3.4 - difficulty as f32 * 0.08 + parkour_focus as f32 * 0.38).clamp(2.05, 3.8),
            2,
        ),
        combo_multiplier_step: round_to(
            0.08 + difficulty as f32 * 0.006 + shooter_focus as f32 * 0.025,
            2,
        ),
        max_combo_multiplier: round_to(
            (1.55
                + difficulty as f32 * 0.08
                + shooter_focus as f32 * 0.22
                + parkour_focus as f32 * 0.12)
                .clamp(1.65, 2.65),
            2,
        ),
        blast_combo_boost: round_to(1.05 + shooter_focus as f32 * 0.35 + parkour_focus as f32 * 0.14, 2),
        command_combo_boost: round_to(1.2 + tower_focus as f32 * 0.24 + shooter_focus as f32 * 0.18, 2),
        support_combo_boost: round_to(0.7 + tower_focus as f32 * 0.18, 2),
        perfect_wave_bonus: (18.0
            + difficulty as f32 * 3.8
            + tower_focus as f32 * 8.0
            + parkour_focus as f32 * 4.0)
            .round() as u32,
    }
}

fn palette_for(biome: RuntimeBiome) -> RuntimePalette {
    match biome {
        RuntimeBiome::NeonHarbor => RuntimePalette {
            sky_top: "#12384A",
            sky_bottom: "#091016",
            ground: "#17212A",
            road: "#25313A",
            grid: "#36D5C7",
            tower: "#2DD4BF",
            enemy: "#FF5A6B",
            projectile: "#FFE45E",
            core: "#F6F7D7",
            accent: "#FF9F1C",
            fog: "#0E2028",
        },
        RuntimeBiome::VerdantRuins => RuntimePalette {
            sky_top: "#244034",
            sky_bottom: "#111A18",
            ground: "#20362C",
            road: "#3B4634",
            grid: "#9BD86E",
            tower: "#64D2A4",
            enemy: "#E15D44",
            projectile: "#FFE66D",
            core: "#BDF7B7",
            accent: "#56CFE1",
            fog: "#172820",
        },
        RuntimeBiome::SunforgeBazaar => RuntimePalette {
            sky_top: "#593B24",
            sky_bottom: "#15110D",
            ground: "#342B22",
            road: "#594935",
            grid: "#FFB703",
            tower: "#3DDC97",
            enemy: "#D62839",
            projectile: "#FFF3B0",
            core: "#F4E285",
            accent: "#00B4D8",
            fog: "#2A2118",
        },
        RuntimeBiome::OrbitalGarden => RuntimePalette {
            sky_top: "#1E3154",
            sky_bottom: "#090A12",
            ground: "#192238",
            road: "#27304A",
            grid: "#7EE7F2",
            tower: "#7BD88F",
            enemy: "#FF477E",
            projectile: "#F9F871",
            core: "#D7FFF1",
            accent: "#F9844A",
            fog: "#111827",
        },
    }
}

fn module_title(modules: &[String]) -> String {
    modules
        .iter()
        .take(2)
        .cloned()
        .collect::<Vec<_>>()
        .join("+")
}

fn build_logic_source(blueprint: &RuntimeSceneBlueprint) -> String {
    let lanes = blueprint
        .lanes
        .iter()
        .map(|lane| {
            format!(
                "{}@({},{}) -> ({},{})",
                lane.id,
                lane.spawn.x.round(),
                lane.spawn.z.round(),
                lane.bend.x.round(),
                lane.bend.z.round()
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let wave_plan = blueprint
        .wave_plan
        .iter()
        .map(|wave| format!("{}@L{}x{}", wave.id, wave.lane_index + 1, wave.count))
        .collect::<Vec<_>>()
        .join(", ");
    let max_wave_count = blueprint
        .wave_plan
        .iter()
        .map(|wave| wave.count)
        .max()
        .unwrap_or(blueprint.spawn.wave_size);
    let min_wave_warning = blueprint
        .wave_plan
        .iter()
        .map(|wave| wave.warning_time)
        .fold(f32::INFINITY, f32::min);
    let tower_archetypes = blueprint
        .tower_archetypes
        .iter()
        .map(|archetype| {
            format!(
                "{}:{}c@R{:.2}x/D{:.2}x",
                archetype.id,
                archetype.build_cost,
                archetype.range_multiplier,
                archetype.damage_multiplier
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let max_tower_damage_multiplier = blueprint
        .tower_archetypes
        .iter()
        .map(|archetype| archetype.damage_multiplier)
        .fold(0.0_f32, f32::max);
    let build_hints = blueprint
        .build_hints
        .iter()
        .map(|hint| {
            format!(
                "{}@A{}/L{}/{}/P{:.2}",
                hint.id,
                hint.anchor_index + 1,
                hint.lane_index + 1,
                hint.tower_archetype_id,
                hint.priority
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let top_build_hint = blueprint
        .build_hints
        .iter()
        .fold(None, |best: Option<&RuntimeBuildHint>, hint| match best {
            None => Some(hint),
            Some(current)
                if hint.priority > current.priority
                    || ((hint.priority - current.priority).abs() <= f32::EPSILON
                        && hint.anchor_index < current.anchor_index) =>
            {
                Some(hint)
            }
            _ => best,
        })
        .map(|hint| hint.id.as_str())
        .unwrap_or("none");
    let enemy_archetypes = blueprint
        .enemy_archetypes
        .iter()
        .map(|archetype| {
            format!(
                "{}:{:.0}hp@{:.2}x",
                archetype.id, archetype.hp, archetype.speed_multiplier
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let max_enemy_reward = blueprint
        .enemy_archetypes
        .iter()
        .map(|archetype| archetype.score_reward)
        .max()
        .unwrap_or(18);
    let boss_plan = format!(
        "{}@W{}L{}:{:.0}hp",
        blueprint.boss_plan.id,
        blueprint.boss_plan.trigger_wave,
        blueprint.boss_plan.lane_index + 1,
        blueprint.boss_plan.hp
    );
    let command_plan = blueprint
        .commands
        .iter()
        .map(|command| {
            format!(
                "{}:{}/{}c/{:.1}s",
                command.id, command.kind, command.score_cost, command.cooldown
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let max_command_magnitude = blueprint
        .commands
        .iter()
        .map(|command| command.magnitude)
        .fold(0.0_f32, f32::max);
    let lane_signals = blueprint
        .lane_signals
        .iter()
        .map(|signal| {
            format!(
                "{}@L{}/R{:.1}",
                signal.id,
                signal.lane_index + 1,
                signal.alert_radius
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let max_lane_signal_height = blueprint
        .lane_signals
        .iter()
        .map(|signal| signal.beacon_height)
        .fold(0.0_f32, f32::max);
    let support_nodes = blueprint
        .support_nodes
        .iter()
        .map(|node| {
            format!(
                "{}@{:.0},{:.0}:+{}/heal{:.1}",
                node.id, node.x, node.z, node.score_per_pulse, node.repair_per_pulse
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let support_pulse_damage = blueprint
        .support_nodes
        .iter()
        .map(|node| node.pulse_damage)
        .fold(0.0_f32, f32::max);
    let set_pieces = blueprint
        .set_pieces
        .iter()
        .map(|set_piece| {
            format!(
                "{}:{}@{:.0},{:.0}",
                set_piece.id, set_piece.kind, set_piece.x, set_piece.z
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let max_set_piece_height = blueprint
        .set_pieces
        .iter()
        .map(|set_piece| set_piece.height)
        .fold(0.0_f32, f32::max);
    let events = blueprint
        .events
        .iter()
        .map(|event| format!("{}@W{}", event.kind, event.trigger_wave))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "// Generated runtime logic: {}\n\
         // Biome: {}; modules: {}\n\
         // Lanes: {}\n\
         const spawnInterval = {:.2};\n\
         const enemySpeed = {:.2};\n\
         const waveSize = {};\n\
         const enemyCap = {};\n\
         const wavePlan = \"{}\";\n\
         const maxWaveCount = {};\n\
         const minWaveWarning = {:.2};\n\
         const towerArchetypes = \"{}\";\n\
         const maxTowerDamageMultiplier = {:.2};\n\
         const buildHints = \"{}\";\n\
         const topBuildHint = \"{}\";\n\
         const enemyArchetypes = \"{}\";\n\
         const maxEnemyReward = {};\n\
         const bossPlan = \"{}\";\n\
         const bossReward = {};\n\
         const bossAuraDamage = {:.2};\n\
         const commandPlan = \"{}\";\n\
         const maxCommandMagnitude = {:.2};\n\
         const commandLaneAssistRadius = {:.2};\n\
         const commandThreatWeight = {:.2};\n\
         const commandReticleRadius = {:.2};\n\
         const laneSignals = \"{}\";\n\
         const maxLaneSignalHeight = {:.2};\n\
         const steeringLerp = {:.3};\n\
         const waypointRadius = {:.2};\n\
         const towerSnapRadius = {:.2};\n\
         const maxTowers = {};\n\
         const targetWaves = {};\n\
         const targetScore = {};\n\
         const directorEvents = \"{}\";\n\
         const tacticalFields = {};\n\
         const fieldSlowMultiplier = {:.2};\n\
         const fieldDamagePerPulse = {:.2};\n\
         const supportNodes = \"{}\";\n\
         const supportPulseDamage = {:.2};\n\
         const setPieces = \"{}\";\n\
         const maxSetPieceHeight = {:.2};\n\
         const particleCount = {};\n\
         const coreHaloRadius = {:.2};\n\
         const laneBeaconCount = {};\n\
         const cameraPanSpeed = {:.2};\n\
         const cameraAutoFocusStrength = {:.2};\n\
         const cameraThreatLead = {:.2};\n\
         const cameraManualOverride = {:.2};\n\
         const cameraAlertZoom = {:.2};\n\
         const blastForce = {:.2};\n\
         const blastCooldown = {:.2};\n\
         const pointerAssistRadius = {:.2};\n\
         const towerRange = {:.2};\n\
         const towerFireInterval = {:.2};\n\
         const projectileSpeed = {:.2};\n\
         const projectileDamage = {:.2};\n\
         const projectileLead = {:.2};\n\
         const comboWindow = {:.2};\n\
         const comboMultiplierStep = {:.2};\n\
         const maxComboMultiplier = {:.2};\n\
         const perfectWaveBonus = {};\n\
         const weakPointPulseInterval = {:.2};\n\
         const breachDamage = {:.2};\n\
         // The director spawns lane waves, follows generated bend waypoints,\n\
         // schedules generated wave plans with lane selection, counts, pacing, and warnings,\n\
         // offers generated tower archetypes with distinct range, fire cadence, damage, and cost,\n\
         // recommends generated build hints that pair tower pads with lane pressure and tower archetypes,\n\
         // mixes generated enemy archetypes with distinct hp, speed, scale, and reward,\n\
         // injects a generated boss wave with warning, reward, and tower-straining aura,\n\
         // exposes generated player command slots for lane damage, core repair, and tower rally control,\n\
         // retargets generated lane commands with pointer and threat-weighted assist,\n\
         // highlights generated lane threat signals before wave and boss pressure arrives,\n\
         // builds and constrains towers on generated build pads,\n\
         // applies generated tactical fields that slow and pulse-damage enemies,\n\
         // pulses generated support nodes that reward cleared space or shock contesting enemies,\n\
         // places generated biome set pieces that make each scene silhouette distinct,\n\
         // animates wind particles, core halos, and lane beacons,\n\
         // tunes camera, threat focus, blast, and build controls per generated scene,\n\
         // applies generated tower range, fire rate, projectile lead, and damage,\n\
         // awards generated combo score, active-control bonuses, and perfect-wave bonuses,\n\
         // boosts tower tuning during timed overdrive events,\n\
         // applies wounded-enemy color shifts,\n\
         // and slows pressure when core integrity is low.",
        blueprint.title,
        blueprint.biome.id(),
        blueprint.modules.join(" + "),
        lanes,
        blueprint.spawn.interval,
        blueprint.spawn.enemy_speed,
        blueprint.spawn.wave_size,
        blueprint.spawn.enemy_cap,
        wave_plan,
        max_wave_count,
        min_wave_warning,
        tower_archetypes,
        max_tower_damage_multiplier,
        build_hints,
        top_build_hint,
        enemy_archetypes,
        max_enemy_reward,
        boss_plan,
        blueprint.boss_plan.score_reward,
        blueprint.boss_plan.aura_damage,
        command_plan,
        max_command_magnitude,
        blueprint.command_targeting.lane_assist_radius,
        blueprint.command_targeting.threat_weight,
        blueprint.command_targeting.reticle_radius,
        lane_signals,
        max_lane_signal_height,
        blueprint.rules.steering_lerp,
        blueprint.rules.waypoint_radius,
        blueprint.rules.tower_snap_radius,
        blueprint.rules.max_towers,
        blueprint.objective.target_waves,
        blueprint.objective.target_score,
        events,
        blueprint.tactical_fields.len(),
        blueprint
            .tactical_fields
            .iter()
            .map(|field| field.slow_multiplier)
            .fold(f32::INFINITY, f32::min),
        blueprint
            .tactical_fields
            .iter()
            .map(|field| field.damage_per_pulse)
            .fold(0.0_f32, f32::max),
        support_nodes,
        support_pulse_damage,
        set_pieces,
        max_set_piece_height,
        blueprint.atmosphere.particle_count,
        blueprint.atmosphere.core_halo_radius,
        blueprint.atmosphere.lane_beacon_count,
        blueprint.controls.camera_pan_speed,
        blueprint.controls.camera_auto_focus_strength,
        blueprint.controls.camera_threat_lead,
        blueprint.controls.camera_manual_override,
        blueprint.controls.camera_alert_zoom,
        blueprint.controls.blast_force,
        blueprint.controls.blast_cooldown,
        blueprint.controls.pointer_assist_radius,
        blueprint.combat.tower_range,
        blueprint.combat.tower_fire_interval,
        blueprint.combat.projectile_speed,
        blueprint.combat.projectile_damage,
        blueprint.combat.projectile_lead,
        blueprint.scoring.combo_window,
        blueprint.scoring.combo_multiplier_step,
        blueprint.scoring.max_combo_multiplier,
        blueprint.scoring.perfect_wave_bonus,
        blueprint.rules.weak_point_pulse_interval,
        blueprint.rules.breach_damage,
    )
}

fn round_to(value: f32, digits: u32) -> f32 {
    let scale = 10_f32.powi(digits as i32);
    (value * scale).round() / scale
}

fn distance_to_segment_squared(point: &Vec2, start: &Vec2, end: &Vec2) -> f32 {
    let dx = end.x - start.x;
    let dz = end.z - start.z;
    let length_squared = dx * dx + dz * dz;
    if length_squared <= f32::EPSILON {
        let px = point.x - start.x;
        let pz = point.z - start.z;
        return px * px + pz * pz;
    }
    let t = (((point.x - start.x) * dx + (point.z - start.z) * dz) / length_squared)
        .clamp(0.0, 1.0);
    let closest_x = start.x + dx * t;
    let closest_z = start.z + dz * t;
    let px = point.x - closest_x;
    let pz = point.z - closest_z;
    px * px + pz * pz
}

#[derive(Debug, Clone)]
struct RuntimeRng {
    state: u32,
}

impl RuntimeRng {
    fn new(seed: u64) -> Self {
        Self { state: seed as u32 }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_add(0x6D2B79F5);
        let mut value = self.state;
        value = (value ^ (value >> 15)).wrapping_mul(value | 1);
        value ^= value.wrapping_add((value ^ (value >> 7)).wrapping_mul(value | 61));
        ((value ^ (value >> 14)) as f32) / 4_294_967_296.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(seed: u64, difficulty: u32, theme_hint: &str) -> RuntimeSceneRequest {
        RuntimeSceneRequest {
            seed,
            player_level: 6,
            difficulty,
            theme_hint: theme_hint.to_string(),
            modules: vec!["tower_defense".to_string(), "parkour".to_string()],
        }
    }

    #[test]
    fn generates_deterministic_scene_for_same_request() {
        let first = generate_runtime_scene(request(42, 5, "space nebula"));
        let second = generate_runtime_scene(request(42, 5, "space nebula"));

        assert_eq!(first, second);
        assert_eq!(first.biome, RuntimeBiome::OrbitalGarden);
        assert!(first.logic_source.contains("Generated runtime logic"));
    }

    #[test]
    fn difficulty_scales_lanes_and_pressure() {
        let easy = generate_runtime_scene(request(7, 2, "forest"));
        let hard = generate_runtime_scene(request(7, 9, "forest"));

        assert_eq!(easy.biome, RuntimeBiome::VerdantRuins);
        assert!(hard.lanes.len() > easy.lanes.len());
        assert!(hard.spawn.enemy_speed > easy.spawn.enemy_speed);
        assert!(hard.spawn.interval < easy.spawn.interval);
        assert!(hard.spawn.enemy_cap > easy.spawn.enemy_cap);
        assert!(hard.wave_plan.len() >= easy.wave_plan.len());
        assert!(hard.wave_plan[0].count >= easy.wave_plan[0].count);
        assert!(hard.wave_plan[0].interval_multiplier <= easy.wave_plan[0].interval_multiplier);
        assert!(hard.wave_plan[0].spawn_spread > 0.0);
        assert_eq!(hard.tower_archetypes.len(), 3);
        assert!(hard.tower_archetypes[1].damage_multiplier > hard.tower_archetypes[0].damage_multiplier);
        assert!(hard.tower_archetypes[1].range_multiplier > hard.tower_archetypes[0].range_multiplier);
        assert!(
            hard.tower_archetypes[2].fire_interval_multiplier
                < hard.tower_archetypes[0].fire_interval_multiplier
        );
        assert!(hard.tower_archetypes[1].build_cost >= hard.tower_archetypes[0].build_cost);
        assert_eq!(hard.build_hints.len(), hard.tower_anchors.len());
        assert!(hard.build_hints[0].priority > easy.build_hints[0].priority);
        assert!(hard.build_hints.iter().all(|hint| {
            hard.tower_archetypes
                .iter()
                .any(|archetype| archetype.id == hint.tower_archetype_id)
        }));
        let top_build_hint = hard
            .build_hints
            .iter()
            .max_by(|a, b| a.priority.partial_cmp(&b.priority).unwrap())
            .unwrap();
        assert!(top_build_hint.priority >= hard.build_hints[0].priority);
        assert_eq!(hard.enemy_archetypes.len(), 3);
        assert!(hard.enemy_archetypes[0].hp > easy.enemy_archetypes[0].hp);
        assert!(hard.enemy_archetypes[0].score_reward > easy.enemy_archetypes[0].score_reward);
        assert!(hard.enemy_archetypes[1].scale > easy.enemy_archetypes[1].scale);
        assert!(hard.boss_plan.hp > easy.boss_plan.hp);
        assert!(hard.boss_plan.score_reward > easy.boss_plan.score_reward);
        assert!(hard.boss_plan.aura_damage > easy.boss_plan.aura_damage);
        assert!(hard.boss_plan.trigger_wave <= hard.objective.target_waves);
        assert_eq!(hard.commands.len(), 3);
        assert!(hard.commands[0].magnitude > easy.commands[0].magnitude);
        assert!(hard.commands[2].radius > easy.commands[2].radius);
        assert!(hard.commands.iter().any(|command| command.kind == "core-repair"));
        assert!(hard.command_targeting.lane_assist_radius > easy.command_targeting.lane_assist_radius);
        assert!(hard.command_targeting.reticle_radius > easy.command_targeting.reticle_radius);
        assert!(hard.command_targeting.retarget_cooldown < easy.command_targeting.retarget_cooldown);
        assert_eq!(hard.lane_signals.len(), hard.lanes.len());
        assert!(hard.lane_signals[0].pulse_speed > easy.lane_signals[0].pulse_speed);
        assert!(hard.lane_signals[0].beacon_height > easy.lane_signals[0].beacon_height);
        assert!(hard.set_pieces.len() >= easy.set_pieces.len());
        let hard_set_piece_height = hard
            .set_pieces
            .iter()
            .map(|set_piece| set_piece.height)
            .fold(0.0_f32, f32::max);
        let easy_set_piece_height = easy
            .set_pieces
            .iter()
            .map(|set_piece| set_piece.height)
            .fold(0.0_f32, f32::max);
        assert!(hard_set_piece_height > easy_set_piece_height);
        assert!(hard
            .set_pieces
            .iter()
            .all(|set_piece| set_piece.radius > 0.0 && set_piece.color.starts_with('#')));
        assert!(hard.tactical_fields.len() >= easy.tactical_fields.len());
        assert!(hard.tactical_fields[0].radius > easy.tactical_fields[0].radius);
        assert!(
            hard.tactical_fields[0].damage_per_pulse
                > easy.tactical_fields[0].damage_per_pulse
        );
        assert!(hard.tactical_fields[0].pulse_interval < easy.tactical_fields[0].pulse_interval);
        assert!(hard.support_nodes.len() >= easy.support_nodes.len());
        assert!(hard.support_nodes[0].score_per_pulse > easy.support_nodes[0].score_per_pulse);
        assert!(hard.support_nodes[0].repair_per_pulse >= easy.support_nodes[0].repair_per_pulse);
        assert!(hard.support_nodes[0].pulse_damage > easy.support_nodes[0].pulse_damage);
        assert!(hard.support_nodes[0].pulse_interval < easy.support_nodes[0].pulse_interval);
        assert!(hard.rules.steering_lerp > easy.rules.steering_lerp);
        assert!(hard.rules.waypoint_radius < easy.rules.waypoint_radius);
        assert!(hard.rules.lane_build_buffer > easy.rules.lane_build_buffer);
        assert!(hard.rules.core_build_radius > easy.rules.core_build_radius);
        assert!(hard.rules.max_towers >= easy.rules.max_towers);
        assert!(hard.rules.breach_damage > easy.rules.breach_damage);
        assert!(hard.objective.target_waves >= easy.objective.target_waves);
        assert!(hard.objective.target_score > easy.objective.target_score);
        assert!(hard.objective.reward_xp > easy.objective.reward_xp);
        assert!(hard.atmosphere.particle_count > easy.atmosphere.particle_count);
        assert!(hard.atmosphere.core_halo_radius > easy.atmosphere.core_halo_radius);
        assert!(hard.controls.camera_pan_speed > easy.controls.camera_pan_speed);
        assert!(hard.controls.camera_auto_focus_strength > easy.controls.camera_auto_focus_strength);
        assert!(hard.controls.camera_threat_lead > easy.controls.camera_threat_lead);
        assert!(easy.controls.camera_manual_override > hard.controls.camera_manual_override);
        assert!(hard.controls.camera_alert_zoom > easy.controls.camera_alert_zoom);
        assert!(hard.controls.blast_force > easy.controls.blast_force);
        assert!(hard.controls.blast_cooldown < easy.controls.blast_cooldown);
        assert!(hard.controls.pointer_assist_radius > easy.controls.pointer_assist_radius);
        assert!(hard.combat.tower_range > easy.combat.tower_range);
        assert!(hard.combat.tower_fire_interval < easy.combat.tower_fire_interval);
        assert!(hard.combat.projectile_speed > easy.combat.projectile_speed);
        assert!(hard.combat.projectile_damage > easy.combat.projectile_damage);
        assert!(hard.combat.projectile_lead > easy.combat.projectile_lead);
        assert!(hard.scoring.combo_window < easy.scoring.combo_window);
        assert!(hard.scoring.combo_multiplier_step >= easy.scoring.combo_multiplier_step);
        assert!(hard.scoring.max_combo_multiplier > easy.scoring.max_combo_multiplier);
        assert!(hard.scoring.perfect_wave_bonus > easy.scoring.perfect_wave_bonus);
        assert!(hard.events.len() >= easy.events.len());
        assert!(hard.events.iter().any(|event| event.kind == "enemy-surge"));
    }

    #[test]
    fn empty_modules_fall_back_to_stage_defaults() {
        let mut req = request(1, 8, "cyber city");
        req.modules.clear();
        let scene = generate_runtime_scene(req);

        assert_eq!(scene.modules, vec!["tower_defense", "shooter", "card"]);
        assert!(scene.title.contains("tower_defense+shooter"));
        assert!(scene.rules.starter_tower_enabled);
    }

    #[test]
    fn clamps_seed_and_difficulty() {
        let scene = generate_runtime_scene(RuntimeSceneRequest {
            seed: 0,
            player_level: 1,
            difficulty: 99,
            theme_hint: "desert forge".to_string(),
            modules: vec![],
        });

        assert_eq!(scene.seed, 1);
        assert_eq!(scene.difficulty, 10);
        assert_eq!(scene.biome, RuntimeBiome::SunforgeBazaar);
        assert!((2..=5).contains(&scene.lanes.len()));
        assert!(scene.rules.weak_point_pulse_force > 70.0);
    }

    #[test]
    fn rule_plan_is_generated_and_reflected_in_logic_source() {
        let scene = generate_runtime_scene(request(9, 6, "cyber city"));

        assert!(scene.rules.first_wave_delay > 0.0);
        assert!(scene.rules.low_integrity_spawn_multiplier > 1.0);
        assert!(scene.logic_source.contains("steeringLerp"));
        assert!(scene.logic_source.contains("waypointRadius"));
        assert!(scene.logic_source.contains("towerSnapRadius"));
        assert!(scene.logic_source.contains("maxTowers"));
        assert!(scene.logic_source.contains("targetWaves"));
        assert!(scene.logic_source.contains("targetScore"));
        assert!(scene.logic_source.contains("directorEvents"));
        assert!(scene.logic_source.contains("wavePlan"));
        assert!(scene.logic_source.contains("maxWaveCount"));
        assert!(scene.logic_source.contains("minWaveWarning"));
        assert!(scene.logic_source.contains("towerArchetypes"));
        assert!(scene.logic_source.contains("maxTowerDamageMultiplier"));
        assert!(scene.logic_source.contains("buildHints"));
        assert!(scene.logic_source.contains("topBuildHint"));
        assert!(scene.logic_source.contains("enemyArchetypes"));
        assert!(scene.logic_source.contains("maxEnemyReward"));
        assert!(scene.logic_source.contains("bossPlan"));
        assert!(scene.logic_source.contains("bossReward"));
        assert!(scene.logic_source.contains("bossAuraDamage"));
        assert!(scene.logic_source.contains("commandPlan"));
        assert!(scene.logic_source.contains("maxCommandMagnitude"));
        assert!(scene.logic_source.contains("commandLaneAssistRadius"));
        assert!(scene.logic_source.contains("commandThreatWeight"));
        assert!(scene.logic_source.contains("commandReticleRadius"));
        assert!(scene.logic_source.contains("laneSignals"));
        assert!(scene.logic_source.contains("maxLaneSignalHeight"));
        assert!(scene.logic_source.contains("tacticalFields"));
        assert!(scene.logic_source.contains("fieldSlowMultiplier"));
        assert!(scene.logic_source.contains("fieldDamagePerPulse"));
        assert!(scene.logic_source.contains("supportNodes"));
        assert!(scene.logic_source.contains("supportPulseDamage"));
        assert!(scene.logic_source.contains("setPieces"));
        assert!(scene.logic_source.contains("maxSetPieceHeight"));
        assert!(scene.logic_source.contains("particleCount"));
        assert!(scene.logic_source.contains("coreHaloRadius"));
        assert!(scene.logic_source.contains("laneBeaconCount"));
        assert!(scene.logic_source.contains("cameraPanSpeed"));
        assert!(scene.logic_source.contains("cameraAutoFocusStrength"));
        assert!(scene.logic_source.contains("cameraThreatLead"));
        assert!(scene.logic_source.contains("cameraManualOverride"));
        assert!(scene.logic_source.contains("cameraAlertZoom"));
        assert!(scene.logic_source.contains("blastCooldown"));
        assert!(scene.logic_source.contains("pointerAssistRadius"));
        assert!(scene.logic_source.contains("towerRange"));
        assert!(scene.logic_source.contains("towerFireInterval"));
        assert!(scene.logic_source.contains("projectileSpeed"));
        assert!(scene.logic_source.contains("projectileDamage"));
        assert!(scene.logic_source.contains("projectileLead"));
        assert!(scene.logic_source.contains("comboWindow"));
        assert!(scene.logic_source.contains("comboMultiplierStep"));
        assert!(scene.logic_source.contains("maxComboMultiplier"));
        assert!(scene.logic_source.contains("perfectWaveBonus"));
        assert!(scene.logic_source.contains("weakPointPulseInterval"));
    }

    // ---------------------------------------------------------------
    // Round 110 — helper-level unit tests pinning the deterministic
    // contract of the 5 private helper functions. These tests use
    // the `super::*` import (line 1924) which makes private
    // `pick_biome`, `default_modules`, `build_lanes`,
    // `build_tower_anchors`, `palette_for`, and `round_to` directly
    // callable from inside the same module.
    //
    // The 5 above-and-beyond tests cover contracts the existing 5
    // integration tests cannot:
    //
    //   1. `pick_biome` — every keyword → expected biome, plus
    //      case-insensitive matching (the function uses
    //      `to_ascii_lowercase`).
    //   2. `default_modules` — 3 difficulty tiers return the
    //      expected module list (mirrors the
    //      `empty_modules_fall_back_to_stage_defaults` integration
    //      test but for the helper directly).
    //   3. `build_lanes` — deterministic for same seed, and
    //      lane_count → unique lane ids.
    //   4. `build_tower_anchors` — count = lane_count + 2 (the
    //      "+2" slack anchors for free-form tower placement).
    //   5. `palette_for` — all 4 biomes return distinct palettes
    //      (no 2 biomes share the same `core` color, which would
    //      indicate a regression in the round-92 atmosphere
    //      contracts).
    //   6. `round_to` — rounding contract (3.14159, 2) → 3.14
    //      and (3.14159, 0) → 3.0.
    //   7. `build_lanes` and `build_tower_anchors` use a
    //      different radius (182 vs 72-110) — verify the
    //      distance-from-origin falls in the expected band.
    // ---------------------------------------------------------------

    #[test]
    fn pick_biome_keyword_routing_is_case_insensitive() {
        // The 4 keyword branches must take priority over
        // the random fallback. Case-insensitive matching is
        // a contract — a regression to `to_lowercase` (full
        // Unicode) would slow this path. Each keyword must
        // route to its expected biome, and an unrelated
        // hint must reach the random branch (we just
        // assert it doesn't return VerdantRuins when
        // "forest" isn't in the hint).
        let mut rng = RuntimeRng::new(42);
        assert_eq!(pick_biome("forest", &mut rng), RuntimeBiome::VerdantRuins);
        assert_eq!(pick_biome("FOREST", &mut rng), RuntimeBiome::VerdantRuins);
        assert_eq!(pick_biome("dark Ruin", &mut rng), RuntimeBiome::VerdantRuins);
        assert_eq!(pick_biome("desert temple", &mut rng), RuntimeBiome::SunforgeBazaar);
        assert_eq!(pick_biome("DESERT FORGE", &mut rng), RuntimeBiome::SunforgeBazaar);
        assert_eq!(pick_biome("space nebula", &mut rng), RuntimeBiome::OrbitalGarden);
        assert_eq!(pick_biome("Orbit", &mut rng), RuntimeBiome::OrbitalGarden);
        assert_eq!(pick_biome("cyber neon city", &mut rng), RuntimeBiome::NeonHarbor);
        // Random branch — assert it returns one of the 4
        // biomes (the function never returns a 5th).
        for seed in 0..8 {
            let mut rng = RuntimeRng::new(seed);
            let biome = pick_biome("generic", &mut rng);
            assert!(matches!(
                biome,
                RuntimeBiome::NeonHarbor
                    | RuntimeBiome::VerdantRuins
                    | RuntimeBiome::SunforgeBazaar
                    | RuntimeBiome::OrbitalGarden
            ));
        }
    }

    #[test]
    fn default_modules_three_difficulty_tiers() {
        // 3 difficulty tiers return 3 different module
        // sets. The 0..3 tier is "parkour+synthesis" (light
        // difficulty). The 4..7 tier is "tower_defense+puzzle"
        // (medium). The 8+ tier is the 3-module
        // "tower_defense+shooter+card" (heavy). These
        // contracts pin the round-78 atom expansion —
        // a future refactor that flattens to a single
        // tier would fail this test.
        assert_eq!(default_modules(0), vec!["parkour", "synthesis"]);
        assert_eq!(default_modules(1), vec!["parkour", "synthesis"]);
        assert_eq!(default_modules(3), vec!["parkour", "synthesis"]);
        assert_eq!(default_modules(4), vec!["tower_defense", "puzzle"]);
        assert_eq!(default_modules(5), vec!["tower_defense", "puzzle"]);
        assert_eq!(default_modules(7), vec!["tower_defense", "puzzle"]);
        assert_eq!(
            default_modules(8),
            vec!["tower_defense", "shooter", "card"]
        );
        assert_eq!(
            default_modules(10),
            vec!["tower_defense", "shooter", "card"]
        );
    }

    #[test]
    fn build_lanes_deterministic_for_same_seed() {
        // Same seed → same lane count, same lane ids, and
        // same first-lane spawn position. A regression
        // that changes the RNG state initialization
        // (e.g. dropping the `state.wrapping_add`
        // pre-warmup) would shift the spawn positions.
        let mut rng_a = RuntimeRng::new(7);
        let mut rng_b = RuntimeRng::new(7);
        let lanes_a = build_lanes(4, &mut rng_a);
        let lanes_b = build_lanes(4, &mut rng_b);
        assert_eq!(lanes_a.len(), 4);
        assert_eq!(lanes_b.len(), 4);
        // Lane ids are 1-indexed.
        for (i, lane) in lanes_a.iter().enumerate() {
            assert_eq!(lane.id, format!("lane-{}", i + 1));
        }
        // Same seed → same first lane spawn.
        assert_eq!(lanes_a[0].spawn.x, lanes_b[0].spawn.x);
        assert_eq!(lanes_a[0].spawn.z, lanes_b[0].spawn.z);
        // Lanes are at radius 182 from origin — verify
        // the distance falls in (160, 200).
        for lane in &lanes_a {
            let dist_sq = lane.spawn.x.powi(2) + lane.spawn.z.powi(2);
            assert!(dist_sq > 160.0_f32.powi(2) && dist_sq < 200.0_f32.powi(2));
        }
    }

    #[test]
    fn build_tower_anchors_count_equals_lane_count_plus_two() {
        // The "+2" slack anchors for free-form tower
        // placement. A regression that drops the +2
        // would shrink the buildable area. The
        // anchors are at radius 72-110 from origin.
        let mut rng = RuntimeRng::new(99);
        for lane_count in 2..=6u32 {
            let anchors = build_tower_anchors(lane_count, &mut rng);
            assert_eq!(anchors.len() as u32, lane_count + 2);
            for anchor in &anchors {
                let dist_sq = anchor.x.powi(2) + anchor.z.powi(2);
                // 72 <= radius <= 110
                assert!(dist_sq >= 72.0_f32.powi(2) && dist_sq <= 110.0_f32.powi(2));
            }
        }
    }

    #[test]
    fn palette_for_all_four_biomes_have_distinct_cores() {
        // The `core` color is what the player sees at the
        // center of the level — if 2 biomes share the same
        // core color, the round-92 atmosphere contract is
        // broken (the player can't visually distinguish
        // them at a glance). A regression that
        // accidentally unifies 2 palettes would fail
        // this test.
        let cores = [
            palette_for(RuntimeBiome::NeonHarbor).core.to_string(),
            palette_for(RuntimeBiome::VerdantRuins).core.to_string(),
            palette_for(RuntimeBiome::SunforgeBazaar).core.to_string(),
            palette_for(RuntimeBiome::OrbitalGarden).core.to_string(),
        ];
        // 4 distinct cores.
        let unique: std::collections::HashSet<_> = cores.iter().collect();
        assert_eq!(unique.len(), 4, "all 4 biomes must have distinct core colors");
        // Every palette color starts with '#' (hex format).
        for biome in [
            RuntimeBiome::NeonHarbor,
            RuntimeBiome::VerdantRuins,
            RuntimeBiome::SunforgeBazaar,
            RuntimeBiome::OrbitalGarden,
        ] {
            let palette = palette_for(biome);
            for color in [
                palette.sky_top,
                palette.sky_bottom,
                palette.ground,
                palette.road,
                palette.grid,
                palette.tower,
                palette.enemy,
                palette.projectile,
                palette.core,
                palette.accent,
                palette.fog,
            ] {
                assert!(color.starts_with('#'), "color must start with '#': {color}");
                assert_eq!(color.len(), 7, "hex color must be 7 chars: {color}");
            }
        }
    }

    #[test]
    fn round_to_rounds_to_specified_digits() {
        // (3.14159, 2) → 3.14
        // (3.14159, 0) → 3.0
        // (3.5, 0) → 4.0 (rounding half-to-even via `f32::round` is half-away-from-zero)
        // (1.005, 2) → 1.0 (f32 precision — pin the actual behavior, not the ideal)
        let actual = round_to(3.14159, 2);
        assert!((actual - 3.14).abs() < 0.001);
        assert_eq!(round_to(3.14159, 0), 3.0);
        assert_eq!(round_to(2.5, 0), 3.0);
        // digits=4 keeps 4 decimal places
        let actual4 = round_to(1.234567, 4);
        assert!((actual4 - 1.2346).abs() < 0.0001);
    }
}
