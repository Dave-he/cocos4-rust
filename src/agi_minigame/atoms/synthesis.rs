use std::any::Any;
use std::collections::HashMap;

use crate::base::value::{Value, ValueMap};

use super::super::atom::{Atom, AtomContext, AtomId, AtomPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    Resource,
    Equipment,
    Consumable,
    Material,
}

#[derive(Debug, Clone)]
pub struct SynthItem {
    pub id: String,
    pub name: String,
    pub item_type: ItemType,
    pub tier: u32,
    pub value: u32,
}

impl SynthItem {
    pub fn new(id: &str, name: &str, item_type: ItemType, tier: u32, value: u32) -> Self {
        Self { id: id.to_string(), name: name.to_string(), item_type, tier, value }
    }
}

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub inputs: Vec<(String, u32)>,
    pub output: (String, u32),
    pub craft_time: f32,
    pub is_unlocked: bool,
}

impl Recipe {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            inputs: Vec::new(),
            output: (String::new(), 0),
            craft_time: 1.0,
            is_unlocked: false,
        }
    }

    pub fn with_input(mut self, item_id: &str, count: u32) -> Self {
        self.inputs.push((item_id.to_string(), count));
        self
    }

    pub fn with_output(mut self, item_id: &str, count: u32) -> Self {
        self.output = (item_id.to_string(), count);
        self
    }

    pub fn with_craft_time(mut self, time: f32) -> Self {
        self.craft_time = time;
        self
    }

    pub fn unlocked(mut self) -> Self {
        self.is_unlocked = true;
        self
    }

    pub fn can_craft(&self, inventory: &HashMap<String, u32>) -> bool {
        if !self.is_unlocked {
            return false;
        }
        for (item_id, count) in &self.inputs {
            if inventory.get(item_id).copied().unwrap_or(0) < *count {
                return false;
            }
        }
        true
    }
}

pub struct SynthesisAtom {
    phase: AtomPhase,
    inventory: HashMap<String, u32>,
    recipes: Vec<Recipe>,
    item_definitions: HashMap<String, SynthItem>,
    crafting_queue: Vec<CraftJob>,
    score: u64,
    items_crafted: u32,
    highest_tier: u32,
    discoveries: u32,
}

#[derive(Debug, Clone)]
struct CraftJob {
    recipe_id: String,
    progress: f32,
    output_item: String,
    output_count: u32,
}

impl SynthesisAtom {
    pub fn new() -> Self {
        Self {
            phase: AtomPhase::Uninitialized,
            inventory: HashMap::new(),
            recipes: Vec::new(),
            item_definitions: HashMap::new(),
            crafting_queue: Vec::new(),
            score: 0,
            items_crafted: 0,
            highest_tier: 0,
            discoveries: 0,
        }
    }

    pub fn add_item_definition(&mut self, item: SynthItem) {
        self.item_definitions.insert(item.id.clone(), item);
    }

    pub fn add_recipe(&mut self, recipe: Recipe) {
        self.recipes.push(recipe);
    }

    pub fn add_to_inventory(&mut self, item_id: &str, count: u32) {
        *self.inventory.entry(item_id.to_string()).or_insert(0) += count;
    }

    pub fn remove_from_inventory(&mut self, item_id: &str, count: u32) -> bool {
        let current = self.inventory.get(item_id).copied().unwrap_or(0);
        if current < count {
            return false;
        }
        self.inventory.insert(item_id.to_string(), current - count);
        if current - count == 0 {
            self.inventory.remove(item_id);
        }
        true
    }

    pub fn get_inventory_count(&self, item_id: &str) -> u32 {
        self.inventory.get(item_id).copied().unwrap_or(0)
    }

    pub fn craft(&mut self, recipe_id: &str) -> bool {
        let recipe = match self.recipes.iter().find(|r| r.id == recipe_id) {
            Some(r) => r.clone(),
            None => return false,
        };

        if !recipe.can_craft(&self.inventory) {
            return false;
        }

        for (item_id, count) in &recipe.inputs {
            self.remove_from_inventory(item_id, *count);
        }

        self.crafting_queue.push(CraftJob {
            recipe_id: recipe.id.clone(),
            progress: 0.0,
            output_item: recipe.output.0.clone(),
            output_count: recipe.output.1,
        });

        true
    }

    pub fn instant_craft(&mut self, recipe_id: &str) -> bool {
        let recipe = match self.recipes.iter().find(|r| r.id == recipe_id) {
            Some(r) => r.clone(),
            None => return false,
        };

        if !recipe.can_craft(&self.inventory) {
            return false;
        }

        for (item_id, count) in &recipe.inputs {
            self.remove_from_inventory(item_id, *count);
        }

        self.add_to_inventory(&recipe.output.0, recipe.output.1);
        self.items_crafted += 1;
        self.score += recipe.output.1 as u64 * 10;

        if let Some(item_def) = self.item_definitions.get(&recipe.output.0) {
            if item_def.tier > self.highest_tier {
                self.highest_tier = item_def.tier;
            }
        }

        self.check_discoveries();
        true
    }

    fn update_crafting(&mut self, dt: f32) {
        let mut completed = Vec::new();
        for job in &mut self.crafting_queue {
            job.progress += dt;
            let recipe = self.recipes.iter().find(|r| r.id == job.recipe_id);
            let craft_time = recipe.map(|r| r.craft_time).unwrap_or(1.0);
            if job.progress >= craft_time {
                completed.push((job.output_item.clone(), job.output_count));
            }
        }

        self.crafting_queue.retain(|j| {
            let recipe = self.recipes.iter().find(|r| r.id == j.recipe_id);
            let craft_time = recipe.map(|r| r.craft_time).unwrap_or(1.0);
            j.progress < craft_time
        });

        for (item_id, count) in completed {
            self.add_to_inventory(&item_id, count);
            self.items_crafted += 1;
            self.score += count as u64 * 10;
        }
    }

    fn check_discoveries(&mut self) {
        for recipe in &mut self.recipes {
            if recipe.is_unlocked {
                continue;
            }
            let all_inputs_available = recipe.inputs.iter().all(|(id, _)| {
                self.inventory.contains_key(id)
            });
            if all_inputs_available {
                recipe.is_unlocked = true;
                self.discoveries += 1;
                self.score += 100;
            }
        }
    }

    pub fn generate_base_items(&mut self) {
        let base_items = vec![
            SynthItem::new("wood", "木材", ItemType::Resource, 1, 5),
            SynthItem::new("stone", "石材", ItemType::Resource, 1, 5),
            SynthItem::new("herb", "草药", ItemType::Resource, 1, 5),
            SynthItem::new("iron", "铁矿石", ItemType::Resource, 1, 8),
            SynthItem::new("crystal", "水晶", ItemType::Resource, 1, 10),
            SynthItem::new("plank", "木板", ItemType::Material, 2, 15),
            SynthItem::new("brick", "砖块", ItemType::Material, 2, 15),
            SynthItem::new("potion", "药水", ItemType::Consumable, 2, 20),
            SynthItem::new("ingot", "铁锭", ItemType::Material, 2, 25),
            SynthItem::new("sword", "铁剑", ItemType::Equipment, 3, 50),
            SynthItem::new("shield", "铁盾", ItemType::Equipment, 3, 50),
            SynthItem::new("magic_staff", "法杖", ItemType::Equipment, 4, 100),
        ];

        for item in base_items {
            self.add_item_definition(item);
        }

        let recipes = vec![
            Recipe::new("r_plank", "制作木板").with_input("wood", 2).with_output("plank", 1).with_craft_time(2.0).unlocked(),
            Recipe::new("r_brick", "制作砖块").with_input("stone", 2).with_output("brick", 1).with_craft_time(2.0).unlocked(),
            Recipe::new("r_potion", "制作药水").with_input("herb", 3).with_output("potion", 1).with_craft_time(3.0).unlocked(),
            Recipe::new("r_ingot", "冶炼铁锭").with_input("iron", 3).with_output("ingot", 1).with_craft_time(4.0),
            Recipe::new("r_sword", "锻造铁剑").with_input("ingot", 2).with_input("wood", 1).with_output("sword", 1).with_craft_time(5.0),
            Recipe::new("r_shield", "锻造铁盾").with_input("ingot", 2).with_input("plank", 1).with_output("shield", 1).with_craft_time(5.0),
            Recipe::new("r_staff", "制作法杖").with_input("crystal", 3).with_input("wood", 2).with_output("magic_staff", 1).with_craft_time(8.0),
        ];

        for recipe in recipes {
            self.add_recipe(recipe);
        }
    }

    pub fn give_starter_resources(&mut self) {
        self.add_to_inventory("wood", 10);
        self.add_to_inventory("stone", 10);
        self.add_to_inventory("herb", 10);
        self.add_to_inventory("iron", 5);
        self.add_to_inventory("crystal", 2);
    }

    pub fn get_score(&self) -> u64 { self.score }
    pub fn get_items_crafted(&self) -> u32 { self.items_crafted }
    pub fn get_highest_tier(&self) -> u32 { self.highest_tier }
    pub fn get_discoveries(&self) -> u32 { self.discoveries }
    pub fn get_crafting_queue_size(&self) -> usize { self.crafting_queue.len() }
    pub fn get_unlocked_recipes(&self) -> Vec<&Recipe> {
        self.recipes.iter().filter(|r| r.is_unlocked).collect()
    }
}

impl Default for SynthesisAtom {
    fn default() -> Self {
        Self::new()
    }
}

impl Atom for SynthesisAtom {
    fn atom_id(&self) -> AtomId { "synthesis".to_string() }
    fn atom_name(&self) -> &str { "合成" }

    fn on_init(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Initialized; }

    fn on_enter(&mut self, _ctx: &mut AtomContext) {
        self.inventory.clear();
        self.crafting_queue.clear();
        self.score = 0;
        self.items_crafted = 0;
        self.highest_tier = 0;
        self.discoveries = 0;
        if self.item_definitions.is_empty() {
            self.generate_base_items();
        }
        self.give_starter_resources();
        self.check_discoveries();
        self.phase = AtomPhase::Running;
    }

    fn on_update(&mut self, ctx: &mut AtomContext) {
        self.update_crafting(ctx.delta_time);
        self.check_discoveries();
    }

    fn on_pause(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Paused; }
    fn on_resume(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Running; }
    fn on_exit(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Completed; }
    fn on_destroy(&mut self) { self.phase = AtomPhase::Uninitialized; }

    fn save_state(&self) -> ValueMap {
        let mut map = ValueMap::new();
        map.insert("score".to_string(), Value::Integer(self.score as i32));
        map.insert("items_crafted".to_string(), Value::Integer(self.items_crafted as i32));
        map.insert("highest_tier".to_string(), Value::Integer(self.highest_tier as i32));
        map.insert("discoveries".to_string(), Value::Integer(self.discoveries as i32));
        map
    }

    fn load_state(&mut self, state: &ValueMap) {
        if let Some(Value::Integer(n)) = state.get("score") { self.score = *n as u64; }
        if let Some(Value::Integer(n)) = state.get("items_crafted") { self.items_crafted = *n as u32; }
        if let Some(Value::Integer(n)) = state.get("highest_tier") { self.highest_tier = *n as u32; }
        if let Some(Value::Integer(n)) = state.get("discoveries") { self.discoveries = *n as u32; }
    }

    fn handle_event(&mut self, event: &str, data: &ValueMap, _ctx: &mut AtomContext) {
        match event {
            "craft" => {
                if let Some(Value::String(id)) = data.get("recipe_id") {
                    self.craft(id);
                }
            }
            "instant_craft" => {
                if let Some(Value::String(id)) = data.get("recipe_id") {
                    self.instant_craft(id);
                }
            }
            "add_item" => {
                if let Some(Value::String(id)) = data.get("item_id") {
                    let count = data.get("count").and_then(|v| if let Value::Integer(n) = v { Some(*n as u32) } else { None }).unwrap_or(1);
                    self.add_to_inventory(id, count);
                }
            }
            _ => {}
        }
    }

    fn current_phase(&self) -> AtomPhase { self.phase }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::agi_minigame::world_state::UnifiedWorldState;
    use crate::agi_minigame::player::PlayerProfile;

    fn make_ctx() -> AtomContext {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))));
        AtomContext::new(ws).with_delta_time(0.016)
    }

    #[test]
    fn test_synthesis_init() {
        let mut atom = SynthesisAtom::new();
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        assert!(atom.get_inventory_count("wood") > 0);
        assert!(atom.get_unlocked_recipes().len() > 0);
    }

    #[test]
    fn test_craft_item() {
        let mut atom = SynthesisAtom::new();
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        let result = atom.instant_craft("r_plank");
        assert!(result);
        assert!(atom.get_inventory_count("plank") > 0);
        assert!(atom.get_score() > 0);
    }

    #[test]
    fn test_craft_insufficient_materials() {
        let mut atom = SynthesisAtom::new();
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        atom.inventory.clear();
        let result = atom.instant_craft("r_plank");
        assert!(!result);
    }

    #[test]
    fn test_recipe_discovery() {
        let mut atom = SynthesisAtom::new();
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        atom.add_to_inventory("iron", 10);
        atom.check_discoveries();

        let unlocked = atom.get_unlocked_recipes();
        assert!(unlocked.len() > 3);
    }

    #[test]
    fn test_crafting_queue() {
        let mut atom = SynthesisAtom::new();
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        atom.craft("r_plank");
        assert_eq!(atom.get_crafting_queue_size(), 1);

        for _ in 0..200 {
            atom.on_update(&mut ctx);
        }
        assert_eq!(atom.get_crafting_queue_size(), 0);
    }

    #[test]
    fn test_inventory_management() {
        let mut atom = SynthesisAtom::new();
        atom.add_to_inventory("wood", 5);
        assert_eq!(atom.get_inventory_count("wood"), 5);
        assert!(atom.remove_from_inventory("wood", 3));
        assert_eq!(atom.get_inventory_count("wood"), 2);
        assert!(!atom.remove_from_inventory("wood", 5));
    }

    #[test]
    fn test_recipe_can_craft() {
        let mut inv = HashMap::new();
        inv.insert("wood".to_string(), 2);
        let recipe = Recipe::new("r1", "Test").with_input("wood", 2).with_output("plank", 1).unlocked();
        assert!(recipe.can_craft(&inv));

        let recipe_locked = Recipe::new("r2", "Locked").with_input("wood", 1).with_output("plank", 1);
        assert!(!recipe_locked.can_craft(&inv));
    }
}
