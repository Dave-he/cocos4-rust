/****************************************************************************
Rust port of Cocos Creator GFX Resource Tracker
Tracks GFX resource lifecycle for leak detection.
****************************************************************************/

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    CommandBuffer,
    Queue,
    QueryPool,
    Swapchain,
    Buffer,
    Texture,
    Shader,
    InputAssembler,
    RenderPass,
    Framebuffer,
    DescriptorSet,
    DescriptorSetLayout,
    PipelineLayout,
    PipelineState,
    Sampler,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceType::CommandBuffer => write!(f, "CommandBuffer"),
            ResourceType::Queue => write!(f, "Queue"),
            ResourceType::QueryPool => write!(f, "QueryPool"),
            ResourceType::Swapchain => write!(f, "Swapchain"),
            ResourceType::Buffer => write!(f, "Buffer"),
            ResourceType::Texture => write!(f, "Texture"),
            ResourceType::Shader => write!(f, "Shader"),
            ResourceType::InputAssembler => write!(f, "InputAssembler"),
            ResourceType::RenderPass => write!(f, "RenderPass"),
            ResourceType::Framebuffer => write!(f, "Framebuffer"),
            ResourceType::DescriptorSet => write!(f, "DescriptorSet"),
            ResourceType::DescriptorSetLayout => write!(f, "DescriptorSetLayout"),
            ResourceType::PipelineLayout => write!(f, "PipelineLayout"),
            ResourceType::PipelineState => write!(f, "PipelineState"),
            ResourceType::Sampler => write!(f, "Sampler"),
        }
    }
}

pub const ALL_RESOURCE_TYPES: [ResourceType; 15] = [
    ResourceType::CommandBuffer,
    ResourceType::Queue,
    ResourceType::QueryPool,
    ResourceType::Swapchain,
    ResourceType::Buffer,
    ResourceType::Texture,
    ResourceType::Shader,
    ResourceType::InputAssembler,
    ResourceType::RenderPass,
    ResourceType::Framebuffer,
    ResourceType::DescriptorSet,
    ResourceType::DescriptorSetLayout,
    ResourceType::PipelineLayout,
    ResourceType::PipelineState,
    ResourceType::Sampler,
];

pub struct ResourceTracker {
    pub resources: HashMap<ResourceType, Vec<u32>>,
    pub enabled: bool,
}

impl ResourceTracker {
    pub fn new() -> Self {
        let mut resources = HashMap::new();
        for rt in &ALL_RESOURCE_TYPES {
            resources.insert(*rt, Vec::new());
        }
        ResourceTracker {
            resources,
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        let tracker = Self::new();
        ResourceTracker {
            resources: tracker.resources,
            enabled: false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn push(&mut self, resource_type: ResourceType, id: u32) {
        if self.enabled {
            self.resources.get_mut(&resource_type).unwrap().push(id);
        }
    }

    pub fn erase(&mut self, resource_type: ResourceType, id: u32) {
        if self.enabled {
            let list = self.resources.get_mut(&resource_type).unwrap();
            if let Some(pos) = list.iter().position(|x| *x == id) {
                list.swap_remove(pos);
            }
        }
    }

    pub fn check_empty(&self, resource_type: ResourceType) -> bool {
        self.resources.get(&resource_type).unwrap().is_empty()
    }

    pub fn check_all_empty(&self) -> bool {
        for rt in &ALL_RESOURCE_TYPES {
            if !self.check_empty(*rt) {
                return false;
            }
        }
        true
    }

    pub fn get_leaked(&self) -> Vec<(ResourceType, usize)> {
        let mut leaked = Vec::new();
        for rt in &ALL_RESOURCE_TYPES {
            let count = self.resources.get(rt).unwrap().len();
            if count > 0 {
                leaked.push((*rt, count));
            }
        }
        leaked
    }

    pub fn clear(&mut self) {
        for rt in &ALL_RESOURCE_TYPES {
            self.resources.get_mut(rt).unwrap().clear();
        }
    }
}

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_tracker_push_erase() {
        let mut tracker = ResourceTracker::new();
        tracker.push(ResourceType::Buffer, 1);
        tracker.push(ResourceType::Buffer, 2);
        tracker.push(ResourceType::Texture, 3);
        assert!(!tracker.check_empty(ResourceType::Buffer));
        assert!(!tracker.check_empty(ResourceType::Texture));
        assert!(tracker.check_empty(ResourceType::Shader));

        tracker.erase(ResourceType::Buffer, 1);
        assert!(!tracker.check_empty(ResourceType::Buffer));

        tracker.erase(ResourceType::Buffer, 2);
        assert!(tracker.check_empty(ResourceType::Buffer));
    }

    #[test]
    fn test_resource_tracker_check_all_empty() {
        let mut tracker = ResourceTracker::new();
        assert!(tracker.check_all_empty());
        tracker.push(ResourceType::Buffer, 1);
        assert!(!tracker.check_all_empty());
        tracker.erase(ResourceType::Buffer, 1);
        assert!(tracker.check_all_empty());
    }

    #[test]
    fn test_resource_tracker_leaked() {
        let mut tracker = ResourceTracker::new();
        tracker.push(ResourceType::Buffer, 1);
        tracker.push(ResourceType::Buffer, 2);
        tracker.push(ResourceType::Texture, 3);
        let leaked = tracker.get_leaked();
        assert_eq!(leaked.len(), 2);
        assert!(leaked
            .iter()
            .any(|(rt, count)| *rt == ResourceType::Buffer && *count == 2));
        assert!(leaked
            .iter()
            .any(|(rt, count)| *rt == ResourceType::Texture && *count == 1));
    }

    #[test]
    fn test_resource_tracker_disabled() {
        let mut tracker = ResourceTracker::disabled();
        tracker.push(ResourceType::Buffer, 1);
        assert!(tracker.check_empty(ResourceType::Buffer));
        assert!(tracker.check_all_empty());
    }

    #[test]
    fn test_resource_tracker_clear() {
        let mut tracker = ResourceTracker::new();
        tracker.push(ResourceType::Buffer, 1);
        tracker.push(ResourceType::Texture, 2);
        tracker.clear();
        assert!(tracker.check_all_empty());
    }

    #[test]
    fn test_resource_type_display() {
        assert_eq!(ResourceType::Buffer.to_string(), "Buffer");
        assert_eq!(ResourceType::CommandBuffer.to_string(), "CommandBuffer");
    }
}
