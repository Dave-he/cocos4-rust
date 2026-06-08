pub struct AsyncRuntime {
    task_count: usize,
}

impl AsyncRuntime {
    pub fn new() -> Self {
        Self { task_count: 0 }
    }

    pub fn spawn<F>(&mut self, _task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.task_count += 1;
    }

    pub fn run_until_complete(&mut self) {
        self.task_count = 0;
    }

    pub fn pending_tasks(&self) -> usize {
        self.task_count
    }
}

impl Default for AsyncRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_runtime_new() {
        let rt = AsyncRuntime::new();
        assert_eq!(rt.pending_tasks(), 0);
    }

    #[test]
    fn test_async_runtime_spawn() {
        let mut rt = AsyncRuntime::new();
        rt.spawn(|| {});
        assert_eq!(rt.pending_tasks(), 1);
    }

    #[test]
    fn test_async_runtime_run() {
        let mut rt = AsyncRuntime::new();
        rt.spawn(|| {});
        rt.run_until_complete();
        assert_eq!(rt.pending_tasks(), 0);
    }
}
