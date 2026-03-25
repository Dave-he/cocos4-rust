pub struct Counter {
    pub id: String,
    pub desc: String,
    pub ave: bool,
    pub min: f64,
    pub max: f64,
    sample_count: u64,
    accumulator: f64,
    average: f64,
    history: Vec<f64>,
    history_size: usize,
}

impl Counter {
    pub fn new(id: &str, desc: &str, ave: bool) -> Self {
        Counter {
            id: id.to_string(),
            desc: desc.to_string(),
            ave,
            min: f64::MAX,
            max: f64::MIN,
            sample_count: 0,
            accumulator: 0.0,
            average: 0.0,
            history: Vec::new(),
            history_size: 30,
        }
    }

    pub fn set_history_size(&mut self, size: usize) {
        self.history_size = size;
    }

    pub fn sample(&mut self, value: f64) {
        self.sample_count += 1;
        self.accumulator += value;

        if value < self.min { self.min = value; }
        if value > self.max { self.max = value; }

        self.history.push(value);
        if self.history.len() > self.history_size {
            self.history.remove(0);
        }

        if self.ave {
            self.average = self.accumulator / self.sample_count as f64;
        } else {
            self.average = value;
        }
    }

    pub fn get_average(&self) -> f64 {
        self.average
    }

    pub fn get_sample_count(&self) -> u64 {
        self.sample_count
    }

    pub fn get_history(&self) -> &[f64] {
        &self.history
    }

    pub fn reset(&mut self) {
        self.min = f64::MAX;
        self.max = f64::MIN;
        self.sample_count = 0;
        self.accumulator = 0.0;
        self.average = 0.0;
        self.history.clear();
    }

    pub fn is_alarm(&self, threshold: f64) -> bool {
        self.average > threshold
    }
}

impl std::fmt::Debug for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Counter")
            .field("id", &self.id)
            .field("average", &self.average)
            .field("sample_count", &self.sample_count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_sample() {
        let mut c = Counter::new("fps", "Frames Per Second", true);
        c.sample(60.0);
        c.sample(58.0);
        c.sample(62.0);
        assert!((c.get_average() - 60.0).abs() < 0.1);
        assert_eq!(c.get_sample_count(), 3);
    }

    #[test]
    fn test_counter_min_max() {
        let mut c = Counter::new("dt", "Delta Time", false);
        c.sample(16.0);
        c.sample(14.0);
        c.sample(20.0);
        assert!((c.min - 14.0).abs() < 1e-9);
        assert!((c.max - 20.0).abs() < 1e-9);
    }

    #[test]
    fn test_counter_reset() {
        let mut c = Counter::new("test", "", false);
        c.sample(100.0);
        c.reset();
        assert_eq!(c.get_sample_count(), 0);
        assert!(c.get_history().is_empty());
    }

    #[test]
    fn test_counter_alarm() {
        let mut c = Counter::new("draw", "", false);
        c.sample(500.0);
        assert!(c.is_alarm(100.0));
        assert!(!c.is_alarm(1000.0));
    }

    #[test]
    fn test_counter_history_size() {
        let mut c = Counter::new("h", "", false);
        c.set_history_size(3);
        for i in 0..5 {
            c.sample(i as f64);
        }
        assert_eq!(c.get_history().len(), 3);
    }
}
