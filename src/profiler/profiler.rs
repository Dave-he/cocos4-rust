use std::collections::HashMap;
use crate::profiler::counter::Counter;
use crate::profiler::perf_counter::PerfCounter;

#[derive(Debug, Clone, PartialEq)]
pub struct ProfilerStats {
    pub fps: f64,
    pub frame_time_ms: f64,
    pub draw_calls: u64,
    pub triangle_count: u64,
    pub node_count: u64,
}

impl Default for ProfilerStats {
    fn default() -> Self {
        ProfilerStats {
            fps: 0.0,
            frame_time_ms: 0.0,
            draw_calls: 0,
            triangle_count: 0,
            node_count: 0,
        }
    }
}

pub struct Profiler {
    pub show_fps: bool,
    pub stats: ProfilerStats,
    frame_counter: PerfCounter,
    counters: HashMap<String, Counter>,
    frame_count: u64,
}

impl Profiler {
    pub fn new() -> Self {
        Profiler {
            show_fps: false,
            stats: ProfilerStats::default(),
            frame_counter: PerfCounter::new("frame", "Frame time", true),
            counters: HashMap::new(),
            frame_count: 0,
        }
    }

    pub fn begin_frame(&mut self, now_ms: f64) {
        self.frame_counter.start(now_ms);
        self.frame_count += 1;
    }

    pub fn end_frame(&mut self, now_ms: f64) {
        self.frame_counter.end(now_ms);
        let avg = self.frame_counter.get_average_ms();
        self.stats.frame_time_ms = avg;
        self.stats.fps = if avg > 0.0 { 1000.0 / avg } else { 0.0 };
    }

    pub fn set_draw_calls(&mut self, count: u64) {
        self.stats.draw_calls = count;
    }

    pub fn set_triangle_count(&mut self, count: u64) {
        self.stats.triangle_count = count;
    }

    pub fn set_node_count(&mut self, count: u64) {
        self.stats.node_count = count;
    }

    pub fn get_fps(&self) -> f64 {
        self.stats.fps
    }

    pub fn get_frame_time(&self) -> f64 {
        self.stats.frame_time_ms
    }

    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn add_counter(&mut self, id: &str, desc: &str, ave: bool) {
        self.counters.insert(id.to_string(), Counter::new(id, desc, ave));
    }

    pub fn sample(&mut self, id: &str, value: f64) {
        if let Some(c) = self.counters.get_mut(id) {
            c.sample(value);
        }
    }

    pub fn get_counter(&self, id: &str) -> Option<&Counter> {
        self.counters.get(id)
    }

    pub fn reset(&mut self) {
        self.stats = ProfilerStats::default();
        self.frame_count = 0;
        for c in self.counters.values_mut() {
            c.reset();
        }
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Profiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Profiler")
            .field("fps", &self.stats.fps)
            .field("frame_time_ms", &self.stats.frame_time_ms)
            .field("draw_calls", &self.stats.draw_calls)
            .field("frame_count", &self.frame_count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_new() {
        let p = Profiler::new();
        assert!((p.get_fps() - 0.0).abs() < 1e-9);
        assert_eq!(p.get_frame_count(), 0);
    }

    #[test]
    fn test_profiler_frame() {
        let mut p = Profiler::new();
        p.begin_frame(0.0);
        p.end_frame(16.0);
        assert!((p.get_frame_time() - 16.0).abs() < 1e-9);
        assert_eq!(p.get_frame_count(), 1);
    }

    #[test]
    fn test_profiler_fps() {
        let mut p = Profiler::new();
        for i in 0..10 {
            p.begin_frame(i as f64 * 16.666);
            p.end_frame(i as f64 * 16.666 + 16.666);
        }
        assert!((p.get_fps() - 60.0).abs() < 1.0);
    }

    #[test]
    fn test_profiler_draw_calls() {
        let mut p = Profiler::new();
        p.set_draw_calls(150);
        assert_eq!(p.stats.draw_calls, 150);
    }

    #[test]
    fn test_profiler_custom_counter() {
        let mut p = Profiler::new();
        p.add_counter("physics", "Physics time", true);
        p.sample("physics", 2.5);
        p.sample("physics", 3.5);
        let c = p.get_counter("physics").unwrap();
        assert!((c.get_average() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_profiler_reset() {
        let mut p = Profiler::new();
        p.begin_frame(0.0);
        p.end_frame(16.0);
        p.set_draw_calls(100);
        p.reset();
        assert_eq!(p.stats.draw_calls, 0);
        assert_eq!(p.get_frame_count(), 0);
    }
}
