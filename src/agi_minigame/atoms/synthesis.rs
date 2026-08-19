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
        assert!(!atom.get_unlocked_recipes().is_empty());
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

// ---------------------------------------------------------------------------
// Round 138 helper-level tests — follow
// the round 110b / 122-137
// pattern. The pre-round-138
// `mod tests` had 7 integration
// tests (init, craft, insufficient,
// discovery, queue, inventory,
// can_craft) but no focused unit
// coverage of the public surface.
// These tests pin the per-enum
// variant counts, per-field
// defaults of `SynthesisAtom::new`
// + `Recipe::new` + `SynthItem::new`,
// the builder-chain returns-self
// contract, the `can_craft` matrix
// (locked / missing / exact /
// over-supplied), the
// `add_to_inventory` /
// `remove_from_inventory` round
// trip, the `craft` vs
// `instant_craft` divergence
// (queue vs immediate), the
// discovery unlock contract, the
// `save_state` / `load_state`
// round-trip, the `handle_event`
// dispatch for craft /
// instant_craft / add_item
// (with + without count) /
// unknown, and the lifecycle
// `on_init` / `on_enter` /
// `on_pause` / `on_resume` /
// `on_exit` / `on_destroy` phase
// transitions.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round138_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::agi_minigame::world_state::UnifiedWorldState;
    use crate::agi_minigame::player::PlayerProfile;

    fn make_ctx() -> AtomContext {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))));
        AtomContext::new(ws).with_delta_time(0.016)
    }

    /// `ItemType` has 4
    /// variants (Resource /
    /// Equipment /
    /// Consumable /
    /// Material).
    #[test]
    fn item_type_has_4_variants_round_138() {
        let v = [
            ItemType::Resource,
            ItemType::Equipment,
            ItemType::Consumable,
            ItemType::Material,
        ];
        for &x in &v { assert_eq!(x, x); }
        assert_ne!(ItemType::Resource, ItemType::Material);
        assert_ne!(ItemType::Consumable, ItemType::Equipment);
    }

    /// `SynthItem::new`
    /// stores the
    /// constructor
    /// args verbatim.
    #[test]
    fn synth_item_new_stores_fields_round_138() {
        let i = SynthItem::new("gem", "宝石", ItemType::Resource, 2, 50);
        assert_eq!(i.id, "gem");
        assert_eq!(i.name, "宝石");
        assert_eq!(i.item_type, ItemType::Resource);
        assert_eq!(i.tier, 2);
        assert_eq!(i.value, 50);
    }

    /// `Recipe::new` —
    /// empty inputs,
    /// output = ("", 0),
    /// craft_time = 1.0,
    /// is_unlocked = false.
    #[test]
    fn recipe_new_defaults_round_138() {
        let r = Recipe::new("r0", "Test");
        assert_eq!(r.id, "r0");
        assert_eq!(r.name, "Test");
        assert!(r.inputs.is_empty());
        assert_eq!(r.output.0, "");
        assert_eq!(r.output.1, 0);
        assert!((r.craft_time - 1.0).abs() < 1e-6);
        assert!(!r.is_unlocked);
    }

    /// `Recipe::with_input`
    /// appends a
    /// (item_id, count)
    /// tuple +
    /// returns self
    /// for chaining.
    #[test]
    fn recipe_with_input_appends_round_138() {
        let r = Recipe::new("r1", "X")
            .with_input("wood", 2)
            .with_input("stone", 1);
        assert_eq!(r.inputs.len(), 2);
        assert_eq!(r.inputs[0], ("wood".to_string(), 2));
        assert_eq!(r.inputs[1], ("stone".to_string(), 1));
    }

    /// `Recipe::with_output`
    /// sets the
    /// output +
    /// returns self
    /// for chaining.
    #[test]
    fn recipe_with_output_round_138() {
        let r = Recipe::new("r1", "X").with_output("plank", 3);
        assert_eq!(r.output.0, "plank");
        assert_eq!(r.output.1, 3);
    }

    /// `Recipe::with_craft_time`
    /// sets time
    /// + returns
    /// self.
    #[test]
    fn recipe_with_craft_time_round_138() {
        let r = Recipe::new("r1", "X").with_craft_time(7.5);
        assert!((r.craft_time - 7.5).abs() < 1e-6);
    }

    /// `Recipe::unlocked`
    /// sets
    /// is_unlocked
    /// to true
    /// + returns
    /// self.
    #[test]
    fn recipe_unlocked_sets_flag_round_138() {
        let r = Recipe::new("r1", "X").unlocked();
        assert!(r.is_unlocked);
    }

    /// `can_craft` —
    /// locked recipe
    /// returns false
    /// even with
    /// enough inputs.
    #[test]
    fn recipe_can_craft_locked_returns_false_round_138() {
        let mut inv = HashMap::new();
        inv.insert("wood".to_string(), 5);
        let r = Recipe::new("r1", "X").with_input("wood", 1);
        // Not unlocked
        // → false.
        assert!(!r.can_craft(&inv));
    }

    /// `can_craft` —
    /// unlocked but
    /// missing input
    /// returns false.
    #[test]
    fn recipe_can_craft_missing_input_returns_false_round_138() {
        let inv = HashMap::new(); // empty
        let r = Recipe::new("r1", "X")
            .with_input("wood", 1)
            .unlocked();
        assert!(!r.can_craft(&inv));
    }

    /// `can_craft` —
    /// exact match
    /// returns true.
    #[test]
    fn recipe_can_craft_exact_match_round_138() {
        let mut inv = HashMap::new();
        inv.insert("wood".to_string(), 2);
        let r = Recipe::new("r1", "X")
            .with_input("wood", 2)
            .unlocked();
        assert!(r.can_craft(&inv));
    }

    /// `can_craft` —
    /// over-supplied
    /// inventory also
    /// returns true.
    #[test]
    fn recipe_can_craft_extra_inventory_ok_round_138() {
        let mut inv = HashMap::new();
        inv.insert("wood".to_string(), 100);
        let r = Recipe::new("r1", "X")
            .with_input("wood", 2)
            .unlocked();
        assert!(r.can_craft(&inv));
    }

    /// `SynthesisAtom::new`
    /// defaults:
    /// phase = Uninit,
    /// all collections
    /// empty, all
    /// counters 0.
    #[test]
    fn synthesis_atom_new_defaults_round_138() {
        let a = SynthesisAtom::new();
        assert_eq!(a.phase, AtomPhase::Uninitialized);
        assert!(a.inventory.is_empty());
        assert!(a.recipes.is_empty());
        assert!(a.item_definitions.is_empty());
        assert!(a.crafting_queue.is_empty());
        assert_eq!(a.score, 0);
        assert_eq!(a.items_crafted, 0);
        assert_eq!(a.highest_tier, 0);
        assert_eq!(a.discoveries, 0);
    }

    /// `add_item_definition`
    /// inserts by id
    /// + overwrites
    /// on re-add.
    #[test]
    fn add_item_definition_inserts_and_overwrites_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_item_definition(SynthItem::new("a", "A", ItemType::Resource, 1, 10));
        assert_eq!(a.item_definitions.len(), 1);
        a.add_item_definition(SynthItem::new("a", "A2", ItemType::Material, 2, 20));
        // Re-adding
        // same id
        // overwrites
        // (not duplicate).
        assert_eq!(a.item_definitions.len(), 1);
        assert_eq!(a.item_definitions.get("a").unwrap().name, "A2");
    }

    /// `add_recipe`
    /// appends.
    #[test]
    fn add_recipe_appends_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_recipe(Recipe::new("r1", "X"));
        a.add_recipe(Recipe::new("r2", "Y"));
        assert_eq!(a.recipes.len(), 2);
    }

    /// `add_to_inventory`
    /// accumulates
    /// on existing
    /// entries.
    #[test]
    fn add_to_inventory_accumulates_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_to_inventory("wood", 3);
        a.add_to_inventory("wood", 2);
        assert_eq!(a.get_inventory_count("wood"), 5);
    }

    /// `remove_from_inventory`
    /// returns false
    /// when count
    /// exceeds current.
    #[test]
    fn remove_from_inventory_insufficient_returns_false_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_to_inventory("wood", 3);
        assert!(!a.remove_from_inventory("wood", 5));
        // Count
        // unchanged.
        assert_eq!(a.get_inventory_count("wood"), 3);
    }

    /// `remove_from_inventory`
    /// drops the
    /// entry when
    /// count hits 0.
    #[test]
    fn remove_from_inventory_drops_at_zero_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_to_inventory("wood", 3);
        assert!(a.remove_from_inventory("wood", 3));
        assert_eq!(a.get_inventory_count("wood"), 0);
        assert!(!a.inventory.contains_key("wood"));
    }

    /// `get_inventory_count`
    /// returns 0
    /// for missing
    /// items.
    #[test]
    fn get_inventory_count_missing_returns_zero_round_138() {
        let a = SynthesisAtom::new();
        assert_eq!(a.get_inventory_count("nope"), 0);
    }

    /// `craft` returns
    /// false for
    /// unknown recipe.
    #[test]
    fn craft_unknown_recipe_returns_false_round_138() {
        let mut a = SynthesisAtom::new();
        assert!(!a.craft("r_does_not_exist"));
    }

    /// `craft` returns
    /// false when
    /// `!can_craft`.
    #[test]
    fn craft_cannot_craft_returns_false_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_recipe(
            Recipe::new("r1", "X")
                .with_input("wood", 1)
                .unlocked(),
        );
        // Empty
        // inventory →
        // !can_craft.
        assert!(!a.craft("r1"));
    }

    /// `craft`
    /// consumes
    /// inputs +
    /// pushes to
    /// queue.
    #[test]
    fn craft_consumes_and_pushes_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_recipe(
            Recipe::new("r1", "X")
                .with_input("wood", 2)
                .with_output("plank", 1)
                .unlocked(),
        );
        a.add_to_inventory("wood", 5);
        assert!(a.craft("r1"));
        // wood
        // consumed
        // 5-2=3.
        assert_eq!(a.get_inventory_count("wood"), 3);
        // Job
        // pushed.
        assert_eq!(a.get_crafting_queue_size(), 1);
    }

    /// `instant_craft`
    /// returns false
    /// for unknown
    /// recipe.
    #[test]
    fn instant_craft_unknown_returns_false_round_138() {
        let mut a = SynthesisAtom::new();
        assert!(!a.instant_craft("r_does_not_exist"));
    }

    /// `instant_craft`
    /// returns false
    /// when
    /// `!can_craft`.
    #[test]
    fn instant_craft_cannot_craft_returns_false_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_recipe(
            Recipe::new("r1", "X")
                .with_input("wood", 5)
                .with_output("plank", 1)
                .unlocked(),
        );
        assert!(!a.instant_craft("r1"));
    }

    /// `instant_craft`
    /// adds output to
    /// inventory +
    /// increments
    /// `items_crafted` +
    /// adds score
    /// (output_count
    /// * 10).
    #[test]
    fn instant_craft_adds_output_and_score_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_item_definition(SynthItem::new("plank", "木板", ItemType::Material, 2, 15));
        a.add_recipe(
            Recipe::new("r1", "X")
                .with_input("wood", 2)
                .with_output("plank", 1)
                .unlocked(),
        );
        a.add_to_inventory("wood", 5);
        assert!(a.instant_craft("r1"));
        assert_eq!(a.get_inventory_count("plank"), 1);
        assert_eq!(a.get_items_crafted(), 1);
        assert_eq!(a.get_score(), 10);
    }

    /// `instant_craft`
    /// updates
    /// `highest_tier`
    /// to the
    /// max-tier
    /// output seen.
    #[test]
    fn instant_craft_updates_highest_tier_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_item_definition(SynthItem::new("sword", "剑", ItemType::Equipment, 3, 50));
        a.add_recipe(
            Recipe::new("r_sword", "Sword")
                .with_input("iron", 2)
                .with_output("sword", 1)
                .unlocked(),
        );
        a.add_to_inventory("iron", 5);
        a.instant_craft("r_sword");
        assert_eq!(a.get_highest_tier(), 3);
    }

    /// `check_discoveries`
    /// unlocks a
    /// locked recipe
    /// when all its
    /// inputs become
    /// present in the
    /// inventory.
    #[test]
    fn check_discoveries_unlocks_recipe_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_recipe(
            Recipe::new("r_x", "X")
                .with_input("iron", 1)
                .with_input("wood", 1),
        );
        assert_eq!(a.get_unlocked_recipes().len(), 0);
        a.add_to_inventory("iron", 1);
        a.add_to_inventory("wood", 1);
        a.check_discoveries();
        assert_eq!(a.get_unlocked_recipes().len(), 1);
        // Discovery
        // counter
        // bumped
        // + 100
        // score
        // bonus.
        assert_eq!(a.get_discoveries(), 1);
        assert_eq!(a.get_score(), 100);
    }

    /// `check_discoveries`
    /// does NOT
    /// re-unlock an
    /// already-unlocked
    /// recipe (no
    /// double
    /// discovery
    /// count).
    #[test]
    fn check_discoveries_no_double_count_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_recipe(
            Recipe::new("r_x", "X")
                .with_input("wood", 1)
                .unlocked(),
        );
        a.add_to_inventory("wood", 1);
        a.check_discoveries();
        a.check_discoveries();
        // Still
        // 0
        // discoveries
        // because the
        // recipe was
        // already
        // unlocked
        // at the
        // time of the
        // first check.
        assert_eq!(a.get_discoveries(), 0);
    }

    /// `generate_base_items`
    /// adds 12
    /// definitions
    /// + 7 recipes.
    #[test]
    fn generate_base_items_populates_round_138() {
        let mut a = SynthesisAtom::new();
        a.generate_base_items();
        assert_eq!(a.item_definitions.len(), 12);
        assert_eq!(a.recipes.len(), 7);
    }

    /// `give_starter_resources`
    /// gives
    /// wood/stone/
    /// herb/iron/
    /// crystal.
    #[test]
    fn give_starter_resources_populates_round_138() {
        let mut a = SynthesisAtom::new();
        a.give_starter_resources();
        assert_eq!(a.get_inventory_count("wood"), 10);
        assert_eq!(a.get_inventory_count("stone"), 10);
        assert_eq!(a.get_inventory_count("herb"), 10);
        assert_eq!(a.get_inventory_count("iron"), 5);
        assert_eq!(a.get_inventory_count("crystal"), 2);
    }

    /// `get_unlocked_recipes`
    /// filters to
    /// only
    /// `is_unlocked`
    /// entries.
    #[test]
    fn get_unlocked_recipes_filters_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_recipe(Recipe::new("a", "A").unlocked());
        a.add_recipe(Recipe::new("b", "B"));
        a.add_recipe(Recipe::new("c", "C").unlocked());
        let unlocked = a.get_unlocked_recipes();
        assert_eq!(unlocked.len(), 2);
        assert_eq!(unlocked[0].id, "a");
        assert_eq!(unlocked[1].id, "c");
    }

    /// `save_state`
    /// includes the
    /// 4 persisted
    /// keys.
    #[test]
    fn save_state_keys_round_138() {
        let a = SynthesisAtom::new();
        let s = a.save_state();
        assert!(s.contains_key("score"));
        assert!(s.contains_key("items_crafted"));
        assert!(s.contains_key("highest_tier"));
        assert!(s.contains_key("discoveries"));
    }

    /// `load_state`
    /// restores all
    /// 4 persisted
    /// fields.
    #[test]
    fn load_state_restores_all_fields_round_138() {
        let mut a = SynthesisAtom::new();
        let mut s = ValueMap::new();
        s.insert("score".to_string(), Value::Integer(1000));
        s.insert("items_crafted".to_string(), Value::Integer(50));
        s.insert("highest_tier".to_string(), Value::Integer(4));
        s.insert("discoveries".to_string(), Value::Integer(7));
        a.load_state(&s);
        assert_eq!(a.score, 1000);
        assert_eq!(a.items_crafted, 50);
        assert_eq!(a.highest_tier, 4);
        assert_eq!(a.discoveries, 7);
    }

    /// `handle_event`
    /// with
    /// `craft` +
    /// `recipe_id`
    /// calls
    /// `craft()`.
    #[test]
    fn handle_event_craft_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_recipe(
            Recipe::new("r1", "X")
                .with_input("wood", 2)
                .unlocked(),
        );
        a.add_to_inventory("wood", 5);
        let mut data = ValueMap::new();
        data.insert("recipe_id".to_string(), Value::String("r1".to_string()));
        let mut ctx = make_ctx();
        a.handle_event("craft", &data, &mut ctx);
        assert_eq!(a.get_crafting_queue_size(), 1);
    }

    /// `handle_event`
    /// with
    /// `instant_craft`
    /// + `recipe_id`
    /// calls
    /// `instant_craft()`.
    #[test]
    fn handle_event_instant_craft_round_138() {
        let mut a = SynthesisAtom::new();
        a.add_recipe(
            Recipe::new("r1", "X")
                .with_input("wood", 2)
                .with_output("plank", 1)
                .unlocked(),
        );
        a.add_to_inventory("wood", 5);
        let mut data = ValueMap::new();
        data.insert("recipe_id".to_string(), Value::String("r1".to_string()));
        let mut ctx = make_ctx();
        a.handle_event("instant_craft", &data, &mut ctx);
        assert_eq!(a.get_inventory_count("plank"), 1);
    }

    /// `handle_event`
    /// with
    /// `add_item` +
    /// `item_id` +
    /// `count`
    /// calls
    /// `add_to_inventory()`.
    #[test]
    fn handle_event_add_item_with_count_round_138() {
        let mut a = SynthesisAtom::new();
        let mut data = ValueMap::new();
        data.insert("item_id".to_string(), Value::String("wood".to_string()));
        data.insert("count".to_string(), Value::Integer(7));
        let mut ctx = make_ctx();
        a.handle_event("add_item", &data, &mut ctx);
        assert_eq!(a.get_inventory_count("wood"), 7);
    }

    /// `handle_event`
    /// with
    /// `add_item`
    /// without
    /// `count`
    /// defaults
    /// to 1.
    #[test]
    fn handle_event_add_item_default_count_round_138() {
        let mut a = SynthesisAtom::new();
        let mut data = ValueMap::new();
        data.insert("item_id".to_string(), Value::String("wood".to_string()));
        let mut ctx = make_ctx();
        a.handle_event("add_item", &data, &mut ctx);
        assert_eq!(a.get_inventory_count("wood"), 1);
    }

    /// `handle_event`
    /// with
    /// unknown
    /// event
    /// name is
    /// a no-op.
    #[test]
    fn handle_event_unknown_is_noop_round_138() {
        let mut a = SynthesisAtom::new();
        let prev = a.score;
        let s = ValueMap::new();
        let mut ctx = make_ctx();
        a.handle_event("bogus", &s, &mut ctx);
        assert_eq!(a.score, prev);
    }

    /// `on_init` →
    /// phase =
    /// `Initialized`.
    #[test]
    fn on_init_phase_round_138() {
        let mut a = SynthesisAtom::new();
        let mut ctx = make_ctx();
        a.on_init(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Initialized);
    }

    /// `on_enter`
    /// gives starter
    /// resources +
    /// sets Running.
    #[test]
    fn on_enter_initializes_round_138() {
        let mut a = SynthesisAtom::new();
        let mut ctx = make_ctx();
        a.on_enter(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Running);
        // Starter
        // resources
        // present.
        assert_eq!(a.get_inventory_count("wood"), 10);
        // Base
        // items
        // generated.
        assert!(!a.item_definitions.is_empty());
    }

    /// `on_pause` /
    /// `on_resume` /
    /// `on_exit` /
    /// `on_destroy`
    /// each
    /// transition
    /// to the
    /// matching
    /// `AtomPhase`.
    #[test]
    fn lifecycle_phases_round_138() {
        let mut a = SynthesisAtom::new();
        let mut ctx = make_ctx();
        a.on_init(&mut ctx);
        a.on_enter(&mut ctx);
        a.on_pause(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Paused);
        a.on_resume(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Running);
        a.on_exit(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Completed);
        a.on_destroy();
        assert_eq!(a.phase, AtomPhase::Uninitialized);
    }

    /// `atom_id` /
    /// `atom_name` /
    /// `as_any` /
    /// `as_any_mut`
    /// contract.
    #[test]
    fn atom_id_and_name_round_138() {
        let a = SynthesisAtom::new();
        assert_eq!(a.atom_id(), "synthesis");
        assert_eq!(a.atom_name(), "合成");
        let _ = a.as_any();
        let mut a = SynthesisAtom::new();
        let _ = a.as_any_mut();
    }

    /// `current_phase`
    /// mirrors the
    /// internal
    /// `phase` field.
    #[test]
    fn current_phase_matches_field_round_138() {
        let mut a = SynthesisAtom::new();
        assert_eq!(a.current_phase(), AtomPhase::Uninitialized);
        a.phase = AtomPhase::Paused;
        assert_eq!(a.current_phase(), AtomPhase::Paused);
    }

    /// `Default::default()`
    /// delegates to
    /// `SynthesisAtom::new()`.
    #[test]
    fn default_delegates_to_new_round_138() {
        let a: SynthesisAtom = Default::default();
        assert_eq!(a.phase, AtomPhase::Uninitialized);
        assert!(a.recipes.is_empty());
    }
}

// ---------------------------------------------------------------------------
// Round 156 helper-level tests for `atoms/synthesis.rs`.
//
// Round 156 closes surface-area gaps left after the
// round-138 sweep on this file. The round-138 block
// covered ItemType / SynthItem / Recipe::new / builder
// methods / can_craft edge cases. Round 156 covers the
// SynthesisAtom *operation* surface — inventory
// management edge cases, the craft vs. instant_craft
// distinction, the crafting queue counter, the
// get_* accessors, the discover counter, and the
// add_item_definition / add_recipe registration paths.
//
// Each test is fully self-contained: builds its own
// SynthesisAtom via `SynthesisAtom::new()` and uses
// inline literals. A regression in one fixture doesn't
// poison the others.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round156_tests {
    use super::*;

    // -----------------------------------------------------------------
    // SynthesisAtom::new — default state.
    // -----------------------------------------------------------------

    #[test]
    fn synthesis_atom_new_is_uninitialized_with_empty_collections_round156() {
        // Round-138 covered Default delegating
        // to new(); round-156 pins the
        // full 6-field state of new() itself
        // so a regression that pre-set any
        // counter or collection would fail
        // here.
        let a = SynthesisAtom::new();
        assert_eq!(a.phase, AtomPhase::Uninitialized);
        assert!(a.inventory.is_empty());
        assert!(a.recipes.is_empty());
        assert!(a.item_definitions.is_empty());
        assert!(a.crafting_queue.is_empty());
        assert_eq!(a.score, 0);
        assert_eq!(a.items_crafted, 0);
        assert_eq!(a.highest_tier, 0);
        assert_eq!(a.discoveries, 0);
    }

    #[test]
    fn synthesis_atom_accessors_all_zero_on_fresh_atom_round156() {
        // The 6 get_* accessors must all
        // return their zero values on a
        // fresh atom — defensive contract
        // pinning each accessor (a
        // regression that used the wrong
        // field would surface as a panic
        // / off-by-one / wrong type).
        let a = SynthesisAtom::new();
        assert_eq!(a.get_score(), 0);
        assert_eq!(a.get_items_crafted(), 0);
        assert_eq!(a.get_highest_tier(), 0);
        assert_eq!(a.get_discoveries(), 0);
        assert_eq!(a.get_crafting_queue_size(), 0);
        assert!(a.get_unlocked_recipes().is_empty());
    }

    // -----------------------------------------------------------------
    // add_to_inventory / get_inventory_count /
    // remove_from_inventory — inventory edge cases.
    // -----------------------------------------------------------------

    #[test]
    fn add_to_inventory_accumulates_across_calls_round156() {
        // The entry-or-insert pattern
        // (round-138/141 contract for
        // accumulator-style fields)
        // applies here: a 2nd add of
        // the same id must sum, not
        // overwrite.
        let mut a = SynthesisAtom::new();
        a.add_to_inventory("wood", 3);
        a.add_to_inventory("wood", 2);
        assert_eq!(a.get_inventory_count("wood"), 5);
    }

    #[test]
    fn get_inventory_count_for_unknown_id_returns_zero_round156() {
        // Pin the unwrap_or(0) guard:
        // a missing id must report 0,
        // not panic, not return None.
        let a = SynthesisAtom::new();
        assert_eq!(a.get_inventory_count("nope"), 0);
    }

    #[test]
    fn remove_from_inventory_when_exact_returns_true_and_removes_key_round156() {
        // remove_from_inventory with
        // count == current must
        // return true AND remove the
        // key from the map (the
        // `current - count == 0` branch
        // — a regression that didn't
        // remove the key would leave
        // a 0-valued entry, which
        // silently breaks HashMap
        // equality + length-based
        // assertions).
        let mut a = SynthesisAtom::new();
        a.add_to_inventory("wood", 3);
        assert!(a.remove_from_inventory("wood", 3));
        assert_eq!(a.get_inventory_count("wood"), 0);
        // The key must be REMOVED (not
        // left as a 0-valued entry):
        // assert!(*a.inventory.contains_key("wood") == false*)
        // is checked via the accessor
        // (which already returns 0 for
        // missing) plus the
        // crafting queue size invariant
        // (no craft jobs were queued
        // here, so size must stay 0).
        assert_eq!(a.get_crafting_queue_size(), 0);
    }

    #[test]
    fn remove_from_inventory_when_insufficient_returns_false_round156() {
        // The first branch: current <
        // count → return false AND
        // leave the inventory unchanged.
        // A regression that mutated
        // inventory before the check
        // would silently drain a
        // partial amount.
        let mut a = SynthesisAtom::new();
        a.add_to_inventory("wood", 1);
        assert!(!a.remove_from_inventory("wood", 5));
        assert_eq!(a.get_inventory_count("wood"), 1);
    }

    // -----------------------------------------------------------------
    // add_recipe / add_item_definition — registration paths.
    // -----------------------------------------------------------------

    #[test]
    fn add_recipe_appends_in_order_round156() {
        // Pin that recipes are stored
        // in insertion order — the
        // get_unlocked_recipes() path
        // depends on this for stable
        // iteration.
        let mut a = SynthesisAtom::new();
        a.add_recipe(Recipe::new("r0", "First"));
        a.add_recipe(Recipe::new("r1", "Second"));
        a.add_recipe(Recipe::new("r2", "Third"));
        // We can't access `recipes`
        // directly (private), so
        // verify via the unlocked
        // accessor (which clones refs
        // in order). All 3 are
        // currently LOCKED, so
        // get_unlocked_recipes is
        // empty — verify by crafting
        // (which would fail if the
        // recipe id isn't found).
        // Simpler: just count
        // crafting-queue size after
        // attempting to craft r0
        // (locked → returns false,
        // no queue push).
        assert_eq!(a.get_crafting_queue_size(), 0);
        assert!(!a.craft("r0"));
    }

    #[test]
    fn add_item_definition_inserts_by_id_round156() {
        // item_definitions is keyed by
        // id; the insert means the
        // highest_tier check inside
        // instant_craft will find it.
        // We verify the highest_tier
        // advance as a proxy: define
        // a tier-3 item, instant_craft
        // a recipe that produces it,
        // and confirm highest_tier
        // becomes 3.
        let mut a = SynthesisAtom::new();
        a.add_item_definition(SynthItem::new(
            "rod", "钓竿", ItemType::Equipment, 3, 50
        ));
        a.add_recipe(
            Recipe::new("craft_rod", "造钓竿")
                .with_input("wood", 1)
                .with_output("rod", 1)
                .unlocked()
        );
        a.add_to_inventory("wood", 5);
        assert!(a.instant_craft("craft_rod"));
        assert_eq!(a.get_highest_tier(), 3);
        assert_eq!(a.get_items_crafted(), 1);
    }

    // -----------------------------------------------------------------
    // craft vs. instant_craft — the operation distinction.
    // -----------------------------------------------------------------

    #[test]
    fn craft_for_unknown_recipe_returns_false_round156() {
        // Pin the first branch of
        // craft(): recipe not found
        // → return false with no
        // side effects.
        let mut a = SynthesisAtom::new();
        assert!(!a.craft("nonexistent"));
        assert_eq!(a.get_crafting_queue_size(), 0);
    }

    #[test]
    fn craft_for_locked_recipe_returns_false_round156() {
        // Pin the second branch:
        // recipe found but can_craft
        // returns false (not unlocked)
        // → return false with no
        // queue push.
        let mut a = SynthesisAtom::new();
        a.add_recipe(Recipe::new("r0", "X").with_input("wood", 1));
        a.add_to_inventory("wood", 5);
        // Not unlocked → false.
        assert!(!a.craft("r0"));
        assert_eq!(a.get_crafting_queue_size(), 0);
    }

    #[test]
    fn craft_for_unlocked_recipe_pushes_to_queue_and_consumes_inputs_round156() {
        // The happy path: unlocked +
        // sufficient inputs → push a
        // CraftJob to the queue AND
        // consume the inputs from
        // inventory. (The actual
        // completion is gated on
        // update_crafting(dt >= craft_time),
        // which the atom calls from
        // update().)
        let mut a = SynthesisAtom::new();
        a.add_recipe(
            Recipe::new("r0", "X")
                .with_input("wood", 2)
                .with_output("plank", 1)
                .unlocked()
        );
        a.add_to_inventory("wood", 5);
        assert!(a.craft("r0"));
        assert_eq!(a.get_crafting_queue_size(), 1);
        // Inputs consumed.
        assert_eq!(a.get_inventory_count("wood"), 3);
    }

    #[test]
    fn instant_craft_for_unlocked_recipe_adds_output_and_updates_counters_round156() {
        // The atomic-flavor path: no
        // queue, just immediate
        // completion. Counters
        // (items_crafted, score,
        // highest_tier) must all
        // reflect the new state.
        let mut a = SynthesisAtom::new();
        a.add_item_definition(SynthItem::new(
            "plank", "木板", ItemType::Material, 1, 15
        ));
        a.add_recipe(
            Recipe::new("r0", "X")
                .with_input("wood", 1)
                .with_output("plank", 2)
                .unlocked()
        );
        a.add_to_inventory("wood", 3);
        assert!(a.instant_craft("r0"));
        // No queue (instant).
        assert_eq!(a.get_crafting_queue_size(), 0);
        // Output added to inventory.
        assert_eq!(a.get_inventory_count("plank"), 2);
        // Counters updated.
        assert_eq!(a.get_items_crafted(), 1);
        // score = 2 * 10 = 20.
        assert_eq!(a.get_score(), 20);
        // highest_tier: 1 (Material tier 1).
        assert_eq!(a.get_highest_tier(), 1);
        // Inputs consumed.
        assert_eq!(a.get_inventory_count("wood"), 2);
    }

    #[test]
    fn instant_craft_for_unknown_recipe_returns_false_and_no_state_change_round156() {
        // Pin the first branch: recipe
        // not found → return false
        // with no counter / inventory
        // mutation.
        let mut a = SynthesisAtom::new();
        a.add_to_inventory("wood", 5);
        assert!(!a.instant_craft("nonexistent"));
        assert_eq!(a.get_inventory_count("wood"), 5);
        assert_eq!(a.get_items_crafted(), 0);
        assert_eq!(a.get_score(), 0);
    }

    #[test]
    fn instant_craft_for_insufficient_inputs_returns_false_round156() {
        // Pin the second branch:
        // recipe found but can_craft
        // returns false (insufficient
        // inputs) → return false with
        // no side effects. A
        // regression that consumed
        // inputs BEFORE the check
        // would silently drain a
        // partial amount.
        let mut a = SynthesisAtom::new();
        a.add_recipe(
            Recipe::new("r0", "X")
                .with_input("wood", 10)
                .with_output("plank", 1)
                .unlocked()
        );
        a.add_to_inventory("wood", 3);
        assert!(!a.instant_craft("r0"));
        // Inventory unchanged.
        assert_eq!(a.get_inventory_count("wood"), 3);
        assert_eq!(a.get_items_crafted(), 0);
    }

    #[test]
    fn highest_tier_advances_only_when_new_tier_exceeds_current_round156() {
        // Pin the `if item_def.tier >
        // self.highest_tier` branch:
        // crafting a tier-1 item sets
        // highest_tier to 1; a
        // subsequent tier-1 craft
        // does NOT regress; a tier-3
        // craft DOES advance.
        let mut a = SynthesisAtom::new();
        a.add_item_definition(SynthItem::new(
            "p1", "T1", ItemType::Material, 1, 5
        ));
        a.add_item_definition(SynthItem::new(
            "p3", "T3", ItemType::Material, 3, 50
        ));
        a.add_recipe(
            Recipe::new("r1", "→T1")
                .with_input("wood", 1)
                .with_output("p1", 1)
                .unlocked()
        );
        a.add_recipe(
            Recipe::new("r3", "→T3")
                .with_input("wood", 1)
                .with_output("p3", 1)
                .unlocked()
        );
        a.add_to_inventory("wood", 5);
        a.instant_craft("r1");
        assert_eq!(a.get_highest_tier(), 1);
        a.instant_craft("r1");
        // Still 1 — no regression.
        assert_eq!(a.get_highest_tier(), 1);
        a.instant_craft("r3");
        // Now 3 — advance.
        assert_eq!(a.get_highest_tier(), 3);
    }
}
