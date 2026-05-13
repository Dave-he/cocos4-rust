/****************************************************************************
Rust port of Cocos Creator Render Queue
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::defines::{RenderPriority, SortingOrder, RenderPassItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderQueueSortMode {
    #[default]
    FrontToBack,
    BackToFront,
    ByPriority,
}

#[derive(Debug, Clone)]
pub struct RenderItem {
    pub model_id: u64,
    pub sub_model_index: u32,
    pub pass_index: u32,
    pub depth: f32,
    pub priority: i32,
}

impl RenderItem {
    pub fn new(model_id: u64, sub_model_index: u32, pass_index: u32, depth: f32) -> Self {
        RenderItem {
            model_id,
            sub_model_index,
            pass_index,
            depth,
            priority: RenderPriority::DEFAULT as i32,
        }
    }
}

impl Default for RenderItem {
    fn default() -> Self {
        RenderItem {
            model_id: 0,
            sub_model_index: 0,
            pass_index: 0,
            depth: 0.0,
            priority: RenderPriority::DEFAULT as i32,
        }
    }
}

#[derive(Debug)]
pub struct RenderQueue {
    pub items: Vec<RenderItem>,
    pub pass_items: Vec<RenderPassItem>,
    pub sort_order: SortingOrder,
    pub is_transparent: bool,
    pub queue_id: u32,
}

impl RenderQueue {
    pub fn new(is_transparent: bool) -> Self {
        RenderQueue {
            items: Vec::new(),
            pass_items: Vec::new(),
            sort_order: if is_transparent {
                SortingOrder::BackToFront
            } else {
                SortingOrder::FrontToBack
            },
            is_transparent,
            queue_id: 0,
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.pass_items.clear();
    }

    pub fn add(&mut self, item: RenderItem) {
        self.items.push(item);
    }

    pub fn add_pass(&mut self, item: RenderPassItem) {
        self.pass_items.push(item);
    }

    pub fn sort(&mut self) {
        match self.sort_order {
            SortingOrder::FrontToBack => {
                self.items.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal));
            }
            SortingOrder::BackToFront => {
                self.items.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));
            }
            SortingOrder::ByPriority => {
                self.items.sort_by_key(|i| i.priority);
            }
        }
        self.pass_items.sort_by_key(|p| p.priority);
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get_items(&self) -> &[RenderItem] {
        &self.items
    }

    pub fn get_pass_items(&self) -> &[RenderPassItem] {
        &self.pass_items
    }

    pub fn get_queue_id(&self) -> u32 {
        self.queue_id
    }

    pub fn set_queue_id(&mut self, id: u32) {
        self.queue_id = id;
    }

    pub fn is_transparent(&self) -> bool {
        self.is_transparent
    }

    pub fn get_sort_order(&self) -> SortingOrder {
        self.sort_order
    }

    pub fn set_sort_order(&mut self, order: SortingOrder) {
        self.sort_order = order;
    }
}

impl Default for RenderQueue {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_queue_new() {
        let q = RenderQueue::new(false);
        assert!(!q.is_transparent());
        assert_eq!(q.get_sort_order(), SortingOrder::FrontToBack);
        assert!(q.is_empty());
    }

    #[test]
    fn test_render_queue_transparent() {
        let q = RenderQueue::new(true);
        assert!(q.is_transparent());
        assert_eq!(q.get_sort_order(), SortingOrder::BackToFront);
    }

    #[test]
    fn test_render_queue_add_and_sort() {
        let mut q = RenderQueue::new(false);
        q.add(RenderItem::new(1, 0, 0, 5.0));
        q.add(RenderItem::new(2, 0, 0, 2.0));
        q.add(RenderItem::new(3, 0, 0, 8.0));
        q.sort();
        assert_eq!(q.items[0].model_id, 2);
        assert_eq!(q.items[1].model_id, 1);
        assert_eq!(q.items[2].model_id, 3);
    }

    #[test]
    fn test_render_queue_clear() {
        let mut q = RenderQueue::new(false);
        q.add(RenderItem::new(1, 0, 0, 1.0));
        q.clear();
        assert!(q.is_empty());
    }

    #[test]
    fn test_render_queue_queue_id() {
        let mut q = RenderQueue::new(false);
        q.set_queue_id(42);
        assert_eq!(q.get_queue_id(), 42);
    }

    #[test]
    fn test_render_queue_pass_items() {
        let mut q = RenderQueue::new(false);
        q.add_pass(RenderPassItem::default());
        assert_eq!(q.get_pass_items().len(), 1);
    }
}
