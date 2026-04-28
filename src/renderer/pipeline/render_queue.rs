#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderPriority {
    Min = 0,
    #[default]
    Default = 100,
    Max = 200,
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
            priority: RenderPriority::Default as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortingOrder {
    #[default]
    FrontToBack,
    BackToFront,
    ByPriority,
}

#[derive(Debug)]
pub struct RenderQueue {
    pub items: Vec<RenderItem>,
    pub sort_order: SortingOrder,
    pub is_transparent: bool,
}

impl RenderQueue {
    pub fn new(is_transparent: bool) -> Self {
        RenderQueue {
            items: Vec::new(),
            sort_order: if is_transparent {
                SortingOrder::BackToFront
            } else {
                SortingOrder::FrontToBack
            },
            is_transparent,
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn add(&mut self, item: RenderItem) {
        self.items.push(item);
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
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
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
        assert!(!q.is_transparent);
        assert_eq!(q.sort_order, SortingOrder::FrontToBack);
        assert!(q.is_empty());
    }

    #[test]
    fn test_render_queue_transparent() {
        let q = RenderQueue::new(true);
        assert_eq!(q.sort_order, SortingOrder::BackToFront);
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
}
