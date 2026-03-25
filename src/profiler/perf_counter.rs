use crate::profiler::counter::Counter;

pub struct PerfCounter {
    pub counter: Counter,
    start_time: Option<f64>,
    last_frame_time: Option<f64>,
}

impl PerfCounter {
    pub fn new(id: &str, desc: &str, ave: bool) -> Self {
        PerfCounter {
            counter: Counter::new(id, desc, ave),
            start_time: None,
            last_frame_time: None,
        }
    }

    pub fn start(&mut self, now: f64) {
        self.start_time = Some(now);
    }

    pub fn end(&mut self, now: f64) {
        if let Some(start) = self.start_time {
            let elapsed = now - start;
            self.counter.sample(elapsed);
            self.start_time = None;
        }
    }

    pub fn tick(&mut self, now: f64) {
        if let Some(last) = self.last_frame_time {
            let dt = now - last;
            self.counter.sample(dt);
        }
        self.last_frame_time = Some(now);
    }

    pub fn frame(&mut self, now: f64) {
        self.tick(now);
    }

    pub fn get_fps(&self) -> f64 {
        let avg = self.counter.get_average();
        if avg > 0.0 { 1000.0 / avg } else { 0.0 }
    }

    pub fn get_average_ms(&self) -> f64 {
        self.counter.get_average()
    }
}

impl std::fmt::Debug for PerfCounter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerfCounter")
            .field("id", &self.counter.id)
            .field("average_ms", &self.counter.get_average())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_counter_start_end() {
        let mut pc = PerfCounter::new("render", "Render time", true);
        pc.start(0.0);
        pc.end(16.0);
        assert!((pc.get_average_ms() - 16.0).abs() < 1e-9);
    }

    #[test]
    fn test_perf_counter_tick() {
        let mut pc = PerfCounter::new("frame", "Frame time", true);
        pc.tick(0.0);
        pc.tick(16.0);
        pc.tick(32.0);
        assert_eq!(pc.counter.get_sample_count(), 2);
        assert!((pc.get_average_ms() - 16.0).abs() < 1e-6);
    }

    #[test]
    fn test_perf_counter_fps() {
        let mut pc = PerfCounter::new("fps", "FPS", true);
        pc.tick(0.0);
        pc.tick(16.666);
        let fps = pc.get_fps();
        assert!((fps - 60.0).abs() < 2.0, "fps={}", fps);
    }

    #[test]
    fn test_perf_counter_end_without_start() {
        let mut pc = PerfCounter::new("test", "", false);
        pc.end(100.0);
        assert_eq!(pc.counter.get_sample_count(), 0);
    }
}
