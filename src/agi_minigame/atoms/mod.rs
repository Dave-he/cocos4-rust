pub mod match3;
pub mod tower_defense;
pub mod card;
pub mod turn_combat;
pub mod parkour;
pub mod synthesis;
pub mod integration_tests;

pub use match3::Match3Atom;
pub use tower_defense::TowerDefenseAtom;
pub use card::CardAtom;
pub use turn_combat::TurnCombatAtom;
pub use parkour::ParkourAtom;
pub use synthesis::SynthesisAtom;

use crate::agi_minigame::atom::{AtomRegistry, AtomMetadata, AtomFactory};

fn factory<A, F>(f: F) -> AtomFactory
where
    A: crate::agi_minigame::atom::Atom + 'static,
    F: Fn() -> A + 'static + Send + Sync,
{
    Box::new(move || Box::new(f()))
}

pub fn register_all_atoms(registry: &mut AtomRegistry) {
    registry.register(
        "match3".to_string(),
        AtomMetadata {
            id: "match3".to_string(),
            name: "三消".to_string(),
            version: 1,
            gameplay_type: "puzzle".to_string(),
            description: "交换、匹配、消除、连锁、得分、道具".to_string(),
            tags: vec!["puzzle".to_string(), "casual".to_string(), "match3".to_string()],
        },
        factory(|| Match3Atom::new(8, 8, 30)),
    );

    registry.register(
        "tower_defense".to_string(),
        AtomMetadata {
            id: "tower_defense".to_string(),
            name: "塔防".to_string(),
            version: 1,
            gameplay_type: "strategy".to_string(),
            description: "放置、路径、怪物波次、攻击、升级、防御".to_string(),
            tags: vec!["strategy".to_string(), "tower_defense".to_string()],
        },
        factory(|| TowerDefenseAtom::new(10, 10, 100.0, 200)),
    );

    registry.register(
        "card".to_string(),
        AtomMetadata {
            id: "card".to_string(),
            name: "卡牌".to_string(),
            version: 1,
            gameplay_type: "card".to_string(),
            description: "抽卡、出牌、费用、效果、结算、卡组".to_string(),
            tags: vec!["card".to_string(), "strategy".to_string()],
        },
        factory(|| CardAtom::new(10, 10)),
    );

    registry.register(
        "turn_combat".to_string(),
        AtomMetadata {
            id: "turn_combat".to_string(),
            name: "回合战斗".to_string(),
            version: 1,
            gameplay_type: "rpg".to_string(),
            description: "行动条、普攻、技能、Buff、属性、站位".to_string(),
            tags: vec!["rpg".to_string(), "combat".to_string()],
        },
        factory(|| TurnCombatAtom::new()),
    );

    registry.register(
        "parkour".to_string(),
        AtomMetadata {
            id: "parkour".to_string(),
            name: "跑酷".to_string(),
            version: 1,
            gameplay_type: "action".to_string(),
            description: "前进、跳跃、滑行、障碍物、收集、冲刺".to_string(),
            tags: vec!["action".to_string(), "runner".to_string()],
        },
        factory(|| ParkourAtom::new(3, 5.0, 3)),
    );

    registry.register(
        "synthesis".to_string(),
        AtomMetadata {
            id: "synthesis".to_string(),
            name: "合成".to_string(),
            version: 1,
            gameplay_type: "casual".to_string(),
            description: "合并、升级、产出、配方、解锁".to_string(),
            tags: vec!["casual".to_string(), "crafting".to_string()],
        },
        factory(|| SynthesisAtom::new()),
    );
}
