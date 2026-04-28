#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sorting {
    pub sorting_layer: u32,
    pub sorting_order: i32,
}

impl Sorting {
    pub fn new() -> Self {
        Sorting {
            sorting_layer: 0,
            sorting_order: 0,
        }
    }

    pub fn with_layer(mut self, layer: u32) -> Self {
        self.sorting_layer = layer;
        self
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.sorting_order = order;
        self
    }

    pub fn get_priority(&self) -> i64 {
        ((self.sorting_layer as i64) << 16) + self.sorting_order as i64
    }

    pub fn compare(&self, other: &Sorting) -> std::cmp::Ordering {
        self.get_priority().cmp(&other.get_priority())
    }
}

impl Default for Sorting {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialOrd for Sorting {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

impl Ord for Sorting {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.compare(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorting_new() {
        let s = Sorting::new();
        assert_eq!(s.sorting_layer, 0);
        assert_eq!(s.sorting_order, 0);
    }

    #[test]
    fn test_sorting_priority() {
        let s1 = Sorting::new().with_layer(0).with_order(5);
        let s2 = Sorting::new().with_layer(1).with_order(0);
        assert!(s2.get_priority() > s1.get_priority());
    }

    #[test]
    fn test_sorting_order_matters() {
        let s1 = Sorting::new().with_order(10);
        let s2 = Sorting::new().with_order(5);
        assert!(s1 > s2);
    }

    #[test]
    fn test_sorting_sort_vec() {
        let mut items = [
            Sorting::new().with_order(3),
            Sorting::new().with_order(1),
            Sorting::new().with_order(2),
        ];
        items.sort();
        assert_eq!(items[0].sorting_order, 1);
        assert_eq!(items[1].sorting_order, 2);
        assert_eq!(items[2].sorting_order, 3);
    }

    #[test]
    fn test_sorting_layer_higher_than_order() {
        let s1 = Sorting::new().with_layer(0).with_order(1000);
        let s2 = Sorting::new().with_layer(1).with_order(-1000);
        assert!(s2 > s1);
    }
}
