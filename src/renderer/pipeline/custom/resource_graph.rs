use super::types::ResourceDesc;

#[derive(Debug, Clone)]
pub struct ManagedResource {
    pub id: u32,
    pub desc: ResourceDesc,
    pub allocated: bool,
    pub alias: Option<u32>,
    pub memory_size: u64,
}

#[derive(Debug, Clone)]
pub struct PersistentResource {
    pub id: u32,
    pub desc: ResourceDesc,
    pub is_external: bool,
}

#[derive(Debug, Clone)]
pub struct ResourceGraph {
    pub managed: Vec<ManagedResource>,
    pub persistent: Vec<PersistentResource>,
    next_id: u32,
}

impl ResourceGraph {
    pub fn new() -> Self {
        Self {
            managed: Vec::new(),
            persistent: Vec::new(),
            next_id: 0,
        }
    }

    pub fn create_managed(&mut self, desc: ResourceDesc) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let size = (desc.width as u64) * (desc.height as u64) * (desc.depth as u64) * 4;
        self.managed.push(ManagedResource {
            id,
            desc,
            allocated: false,
            alias: None,
            memory_size: size,
        });
        id
    }

    pub fn create_persistent(&mut self, desc: ResourceDesc, is_external: bool) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.persistent.push(PersistentResource {
            id,
            desc,
            is_external,
        });
        id
    }

    pub fn get_managed(&self, id: u32) -> Option<&ManagedResource> {
        self.managed.iter().find(|r| r.id == id)
    }

    pub fn get_managed_mut(&mut self, id: u32) -> Option<&mut ManagedResource> {
        self.managed.iter_mut().find(|r| r.id == id)
    }

    pub fn get_persistent(&self, id: u32) -> Option<&PersistentResource> {
        self.persistent.iter().find(|r| r.id == id)
    }

    pub fn allocate_managed(&mut self, id: u32) {
        if let Some(r) = self.managed.iter_mut().find(|r| r.id == id) {
            r.allocated = true;
        }
    }

    pub fn free_managed(&mut self, id: u32) {
        if let Some(r) = self.managed.iter_mut().find(|r| r.id == id) {
            r.allocated = false;
        }
    }

    pub fn set_alias(&mut self, id: u32, alias_id: u32) {
        if let Some(r) = self.managed.iter_mut().find(|r| r.id == id) {
            r.alias = Some(alias_id);
        }
    }

    pub fn is_allocated(&self, id: u32) -> bool {
        self.managed
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.allocated)
            .unwrap_or(false)
    }

    pub fn get_total_memory(&self) -> u64 {
        self.managed.iter().map(|r| r.memory_size).sum()
    }

    pub fn clear(&mut self) {
        self.managed.clear();
        self.persistent.clear();
        self.next_id = 0;
    }
}

impl Default for ResourceGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ResourceKind;

    fn test_resource(name: &str) -> ResourceDesc {
        ResourceDesc {
            name: name.to_string(),
            kind: ResourceKind::ManagedTexture,
            width: 1024,
            height: 1024,
            format: 0,
            ..Default::default()
        }
    }

    #[test]
    fn test_resource_graph_new() {
        let rg = ResourceGraph::new();
        assert_eq!(rg.managed.len(), 0);
        assert_eq!(rg.persistent.len(), 0);
    }

    #[test]
    fn test_create_managed() {
        let mut rg = ResourceGraph::new();
        let id = rg.create_managed(test_resource("rt0"));
        assert!(rg.get_managed(id).is_some());
        assert!(!rg.is_allocated(id));
    }

    #[test]
    fn test_allocate_free_managed() {
        let mut rg = ResourceGraph::new();
        let id = rg.create_managed(test_resource("rt0"));
        rg.allocate_managed(id);
        assert!(rg.is_allocated(id));
        rg.free_managed(id);
        assert!(!rg.is_allocated(id));
    }

    #[test]
    fn test_memory_tracking() {
        let mut rg = ResourceGraph::new();
        rg.create_managed(ResourceDesc {
            name: "rt0".into(),
            kind: ResourceKind::ManagedTexture,
            width: 1024,
            height: 1024,
            depth: 1,
            format: 0,
            ..Default::default()
        });
        rg.create_managed(ResourceDesc {
            name: "rt1".into(),
            kind: ResourceKind::ManagedTexture,
            width: 512,
            height: 512,
            depth: 1,
            format: 0,
            ..Default::default()
        });
        let total = rg.get_total_memory();
        assert!(total > 0);
    }

    #[test]
    fn test_alias_tracking() {
        let mut rg = ResourceGraph::new();
        let a = rg.create_managed(test_resource("a"));
        let b = rg.create_managed(test_resource("b"));
        rg.set_alias(a, b);
        let res = rg.get_managed(a).unwrap();
        assert_eq!(res.alias, Some(b));
    }
}
