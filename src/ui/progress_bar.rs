#[derive(Debug, Clone, PartialEq)]
pub struct ProgressBar {
    pub total_length: f32,
    pub progress: f32,
    pub reverse: bool,
}

impl ProgressBar {
    pub fn new() -> Self {
        ProgressBar {
            total_length: 100.0,
            progress: 0.0,
            reverse: false,
        }
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    pub fn get_progress(&self) -> f32 {
        self.progress
    }

    pub fn get_fill_length(&self) -> f32 {
        if self.reverse {
            self.total_length * (1.0 - self.progress)
        } else {
            self.total_length * self.progress
        }
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_new() {
        let pb = ProgressBar::new();
        assert!((pb.get_progress() - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_progress() {
        let mut pb = ProgressBar::new();
        pb.set_progress(0.75);
        assert!((pb.get_progress() - 0.75).abs() < 1e-5);
        assert!((pb.get_fill_length() - 75.0).abs() < 1e-4);
    }

    #[test]
    fn test_progress_clamp() {
        let mut pb = ProgressBar::new();
        pb.set_progress(1.5);
        assert!((pb.get_progress() - 1.0).abs() < 1e-5);
        pb.set_progress(-0.5);
        assert!((pb.get_progress() - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_reverse_progress() {
        let mut pb = ProgressBar::new();
        pb.reverse = true;
        pb.set_progress(0.25);
        assert!((pb.get_fill_length() - 75.0).abs() < 1e-4);
    }
}
