use std::collections::HashMap;
use std::any::TypeId;
use crate::core::scene_graph::{Component, NodePtr};

pub type ComponentFactory = Box<dyn Fn() -> Box<dyn Component> + Send + Sync>;

#[allow(dead_code)]
struct ComponentMeta {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    type_id: TypeId,
    factory: ComponentFactory,
    execution_order: i32,
}

pub struct ComponentRegistry {
    entries: HashMap<String, ComponentMeta>,
    type_to_name: HashMap<TypeId, String>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        ComponentRegistry {
            entries: HashMap::new(),
            type_to_name: HashMap::new(),
        }
    }

    pub fn register<C: Component + Default + 'static>(&mut self, name: &str) {
        let type_id = TypeId::of::<C>();
        let meta = ComponentMeta {
            name: name.to_string(),
            type_id,
            factory: Box::new(|| Box::new(C::default())),
            execution_order: 0,
        };
        self.entries.insert(name.to_string(), meta);
        self.type_to_name.insert(type_id, name.to_string());
    }

    pub fn register_with_order<C: Component + Default + 'static>(
        &mut self,
        name: &str,
        order: i32,
    ) {
        let type_id = TypeId::of::<C>();
        let meta = ComponentMeta {
            name: name.to_string(),
            type_id,
            factory: Box::new(|| Box::new(C::default())),
            execution_order: order,
        };
        self.entries.insert(name.to_string(), meta);
        self.type_to_name.insert(type_id, name.to_string());
    }

    pub fn create(&self, name: &str) -> Option<Box<dyn Component>> {
        self.entries.get(name).map(|m| (m.factory)())
    }

    pub fn add_to_node(&self, name: &str, node: &NodePtr) -> bool {
        if let Some(component) = self.create(name) {
            if let Ok(mut n) = node.lock() {
                n.add_component_boxed(component);
                return true;
            }
        }
        false
    }

    pub fn get_name_by_type<C: Component + 'static>(&self) -> Option<&str> {
        let type_id = TypeId::of::<C>();
        self.type_to_name.get(&type_id).map(|s| s.as_str())
    }

    pub fn is_registered(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    pub fn is_type_registered<C: Component + 'static>(&self) -> bool {
        let type_id = TypeId::of::<C>();
        self.type_to_name.contains_key(&type_id)
    }

    pub fn get_execution_order(&self, name: &str) -> Option<i32> {
        self.entries.get(name).map(|m| m.execution_order)
    }

    pub fn get_registered_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.entries.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    pub fn registered_count(&self) -> usize {
        self.entries.len()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::{Any, TypeId};

    #[derive(Default)]
    struct TestComponentA {
        #[allow(dead_code)]
        pub value: i32,
    }

    impl Component for TestComponentA {
        fn get_type_id(&self) -> TypeId { TypeId::of::<TestComponentA>() }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    #[derive(Default)]
    struct TestComponentB {
        #[allow(dead_code)]
        pub label: String,
    }

    impl Component for TestComponentB {
        fn get_type_id(&self) -> TypeId { TypeId::of::<TestComponentB>() }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    #[test]
    fn test_registry_new() {
        let reg = ComponentRegistry::new();
        assert_eq!(reg.registered_count(), 0);
    }

    #[test]
    fn test_register_and_is_registered() {
        let mut reg = ComponentRegistry::new();
        reg.register::<TestComponentA>("TestA");
        assert!(reg.is_registered("TestA"));
        assert!(!reg.is_registered("TestB"));
    }

    #[test]
    fn test_is_type_registered() {
        let mut reg = ComponentRegistry::new();
        reg.register::<TestComponentA>("TestA");
        assert!(reg.is_type_registered::<TestComponentA>());
        assert!(!reg.is_type_registered::<TestComponentB>());
    }

    #[test]
    fn test_create_component() {
        let mut reg = ComponentRegistry::new();
        reg.register::<TestComponentA>("TestA");
        let comp = reg.create("TestA");
        assert!(comp.is_some());
    }

    #[test]
    fn test_create_unknown_returns_none() {
        let reg = ComponentRegistry::new();
        assert!(reg.create("Unknown").is_none());
    }

    #[test]
    fn test_get_name_by_type() {
        let mut reg = ComponentRegistry::new();
        reg.register::<TestComponentA>("TestA");
        assert_eq!(reg.get_name_by_type::<TestComponentA>(), Some("TestA"));
        assert_eq!(reg.get_name_by_type::<TestComponentB>(), None);
    }

    #[test]
    fn test_execution_order() {
        let mut reg = ComponentRegistry::new();
        reg.register_with_order::<TestComponentA>("TestA", 100);
        reg.register_with_order::<TestComponentB>("TestB", -10);
        assert_eq!(reg.get_execution_order("TestA"), Some(100));
        assert_eq!(reg.get_execution_order("TestB"), Some(-10));
    }

    #[test]
    fn test_registered_names_sorted() {
        let mut reg = ComponentRegistry::new();
        reg.register::<TestComponentB>("Zebra");
        reg.register::<TestComponentA>("Apple");
        let names = reg.get_registered_names();
        assert_eq!(names, vec!["Apple", "Zebra"]);
    }

    #[test]
    fn test_registered_count() {
        let mut reg = ComponentRegistry::new();
        reg.register::<TestComponentA>("A");
        reg.register::<TestComponentB>("B");
        assert_eq!(reg.registered_count(), 2);
    }
}
