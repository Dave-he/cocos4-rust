/****************************************************************************
Rust port of Cocos Creator Render Entity
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};
use crate::math::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderEntityType {
    Static = 0,
    Dynamic = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityDirtyFlag {
    None = 0,
    Position = 1 << 0,
    Uv = 1 << 1,
    Color = 1 << 2,
    Texture = 1 << 3,
    All = 0xFF,
}

pub trait RenderEntity: RefCounted {
    fn get_entity_type(&self) -> RenderEntityType;
    fn get_dirty_flag(&self) -> u32;
    fn set_dirty_flag(&mut self, flag: u32);
    fn clear_dirty_flag(&mut self);
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
    fn get_color(&self) -> Color;
    fn set_color(&mut self, color: Color);
}

#[derive(Debug)]
pub struct RenderEntityImpl {
    pub entity_type: RenderEntityType,
    pub dirty_flag: u32,
    pub enabled: bool,
    pub color: Color,
    pub layer: u32,
    pub visible: bool,
    ref_count: RefCountedImpl,
}

impl RenderEntityImpl {
    pub fn new(entity_type: RenderEntityType) -> Self {
        RenderEntityImpl {
            entity_type,
            dirty_flag: EntityDirtyFlag::All as u32,
            enabled: true,
            color: Color::WHITE,
            layer: 0,
            visible: true,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn mark_dirty(&mut self, flag: EntityDirtyFlag) {
        self.dirty_flag |= flag as u32;
    }

    pub fn get_layer(&self) -> u32 {
        self.layer
    }

    pub fn set_layer(&mut self, layer: u32) {
        self.layer = layer;
    }

    pub fn get_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl Default for RenderEntityImpl {
    fn default() -> Self {
        Self::new(RenderEntityType::Dynamic)
    }
}

impl RefCounted for RenderEntityImpl {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

impl RenderEntity for RenderEntityImpl {
    fn get_entity_type(&self) -> RenderEntityType { self.entity_type }
    fn get_dirty_flag(&self) -> u32 { self.dirty_flag }
    fn set_dirty_flag(&mut self, flag: u32) { self.dirty_flag = flag; }
    fn clear_dirty_flag(&mut self) { self.dirty_flag = EntityDirtyFlag::None as u32; }
    fn is_enabled(&self) -> bool { self.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn get_color(&self) -> Color { self.color }
    fn set_color(&mut self, color: Color) {
        self.color = color;
        self.dirty_flag |= EntityDirtyFlag::Color as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_entity_new() {
        let entity = RenderEntityImpl::new(RenderEntityType::Dynamic);
        assert_eq!(entity.get_entity_type(), RenderEntityType::Dynamic);
        assert!(entity.is_enabled());
        assert_eq!(entity.get_dirty_flag(), EntityDirtyFlag::All as u32);
    }

    #[test]
    fn test_render_entity_color() {
        let mut entity = RenderEntityImpl::default();
        entity.clear_dirty_flag();
        entity.set_color(Color::RED);
        assert_eq!(entity.get_color(), Color::RED);
        assert_ne!(entity.get_dirty_flag(), 0);
    }

    #[test]
    fn test_render_entity_dirty() {
        let mut entity = RenderEntityImpl::default();
        entity.clear_dirty_flag();
        assert_eq!(entity.get_dirty_flag(), 0);
        entity.mark_dirty(EntityDirtyFlag::Position);
        assert_ne!(entity.get_dirty_flag() & EntityDirtyFlag::Position as u32, 0);
    }

    #[test]
    fn test_render_entity_layer() {
        let mut entity = RenderEntityImpl::default();
        entity.set_layer(5);
        assert_eq!(entity.get_layer(), 5);
    }
}
