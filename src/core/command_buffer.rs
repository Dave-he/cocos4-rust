use std::collections::VecDeque;

pub trait Command: Send + Sync {
    fn execute(&mut self);
    fn undo(&mut self);
    fn description(&self) -> &str { "" }
    fn is_mergeable_with(&self, _other: &dyn Command) -> bool { false }
}

pub struct CommandBuffer {
    history: VecDeque<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    max_history: usize,
    executing: bool,
    command_count: u64,
}

impl CommandBuffer {
    pub fn new(max_history: usize) -> Self {
        CommandBuffer {
            history: VecDeque::new(),
            redo_stack: Vec::new(),
            max_history,
            executing: false,
            command_count: 0,
        }
    }

    pub fn execute<C: Command + 'static>(&mut self, mut cmd: C) {
        cmd.execute();
        self.redo_stack.clear();
        self.push_to_history(Box::new(cmd));
        self.command_count += 1;
    }

    pub fn execute_boxed(&mut self, mut cmd: Box<dyn Command>) {
        cmd.execute();
        self.redo_stack.clear();
        self.push_to_history(cmd);
        self.command_count += 1;
    }

    fn push_to_history(&mut self, cmd: Box<dyn Command>) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(cmd);
    }

    pub fn undo(&mut self) -> bool {
        if let Some(mut cmd) = self.history.pop_back() {
            cmd.undo();
            self.redo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute();
            self.history.push_back(cmd);
            true
        } else {
            false
        }
    }

    pub fn undo_count(&self) -> usize {
        self.history.len()
    }

    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.redo_stack.clear();
    }

    pub fn clear_redo(&mut self) {
        self.redo_stack.clear();
    }

    pub fn get_command_count(&self) -> u64 {
        self.command_count
    }

    pub fn get_max_history(&self) -> usize {
        self.max_history
    }

    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        while self.history.len() > max {
            self.history.pop_front();
        }
    }

    pub fn get_history_descriptions(&self) -> Vec<&str> {
        self.history.iter().map(|c| c.description()).collect()
    }
}

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new(100)
    }
}

pub struct LambdaCommand {
    desc: String,
    execute_fn: Box<dyn Fn() + Send + Sync>,
    undo_fn: Box<dyn Fn() + Send + Sync>,
}

impl LambdaCommand {
    pub fn new<E, U>(desc: &str, execute: E, undo: U) -> Self
    where
        E: Fn() + Send + Sync + 'static,
        U: Fn() + Send + Sync + 'static,
    {
        LambdaCommand {
            desc: desc.to_string(),
            execute_fn: Box::new(execute),
            undo_fn: Box::new(undo),
        }
    }
}

impl Command for LambdaCommand {
    fn execute(&mut self) { (self.execute_fn)(); }
    fn undo(&mut self) { (self.undo_fn)(); }
    fn description(&self) -> &str { &self.desc }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct IncrementCommand {
        counter: Arc<Mutex<i32>>,
        delta: i32,
    }

    impl Command for IncrementCommand {
        fn execute(&mut self) { *self.counter.lock().unwrap() += self.delta; }
        fn undo(&mut self) { *self.counter.lock().unwrap() -= self.delta; }
        fn description(&self) -> &str { "increment" }
    }

    fn make_buf() -> CommandBuffer { CommandBuffer::new(10) }

    #[test]
    fn test_command_buffer_new() {
        let buf = make_buf();
        assert!(!buf.can_undo());
        assert!(!buf.can_redo());
        assert_eq!(buf.get_command_count(), 0);
    }

    #[test]
    fn test_execute_command() {
        let counter = Arc::new(Mutex::new(0));
        let mut buf = make_buf();
        buf.execute(IncrementCommand { counter: Arc::clone(&counter), delta: 5 });
        assert_eq!(*counter.lock().unwrap(), 5);
        assert!(buf.can_undo());
        assert_eq!(buf.get_command_count(), 1);
    }

    #[test]
    fn test_undo() {
        let counter = Arc::new(Mutex::new(0));
        let mut buf = make_buf();
        buf.execute(IncrementCommand { counter: Arc::clone(&counter), delta: 10 });
        let ok = buf.undo();
        assert!(ok);
        assert_eq!(*counter.lock().unwrap(), 0);
        assert!(!buf.can_undo());
        assert!(buf.can_redo());
    }

    #[test]
    fn test_redo() {
        let counter = Arc::new(Mutex::new(0));
        let mut buf = make_buf();
        buf.execute(IncrementCommand { counter: Arc::clone(&counter), delta: 3 });
        buf.undo();
        let ok = buf.redo();
        assert!(ok);
        assert_eq!(*counter.lock().unwrap(), 3);
        assert!(!buf.can_redo());
    }

    #[test]
    fn test_new_command_clears_redo() {
        let counter = Arc::new(Mutex::new(0));
        let mut buf = make_buf();
        buf.execute(IncrementCommand { counter: Arc::clone(&counter), delta: 1 });
        buf.undo();
        assert!(buf.can_redo());
        buf.execute(IncrementCommand { counter: Arc::clone(&counter), delta: 2 });
        assert!(!buf.can_redo());
    }

    #[test]
    fn test_undo_empty_returns_false() {
        let mut buf = make_buf();
        assert!(!buf.undo());
    }

    #[test]
    fn test_redo_empty_returns_false() {
        let mut buf = make_buf();
        assert!(!buf.redo());
    }

    #[test]
    fn test_max_history_limit() {
        let counter = Arc::new(Mutex::new(0));
        let mut buf = CommandBuffer::new(3);
        for i in 0..5 {
            buf.execute(IncrementCommand { counter: Arc::clone(&counter), delta: i });
        }
        assert_eq!(buf.undo_count(), 3);
    }

    #[test]
    fn test_clear() {
        let counter = Arc::new(Mutex::new(0));
        let mut buf = make_buf();
        buf.execute(IncrementCommand { counter: Arc::clone(&counter), delta: 1 });
        buf.undo();
        buf.clear();
        assert!(!buf.can_undo());
        assert!(!buf.can_redo());
    }

    #[test]
    fn test_lambda_command() {
        let state = Arc::new(Mutex::new(0i32));
        let s_exec = Arc::clone(&state);
        let s_undo = Arc::clone(&state);
        let cmd = LambdaCommand::new(
            "set_10",
            move || { *s_exec.lock().unwrap() = 10; },
            move || { *s_undo.lock().unwrap() = 0; },
        );
        let mut buf = make_buf();
        buf.execute(cmd);
        assert_eq!(*state.lock().unwrap(), 10);
        buf.undo();
        assert_eq!(*state.lock().unwrap(), 0);
        buf.redo();
        assert_eq!(*state.lock().unwrap(), 10);
    }

    #[test]
    fn test_history_descriptions() {
        let counter = Arc::new(Mutex::new(0));
        let mut buf = make_buf();
        buf.execute(IncrementCommand { counter: Arc::clone(&counter), delta: 1 });
        buf.execute(IncrementCommand { counter: Arc::clone(&counter), delta: 2 });
        let descs = buf.get_history_descriptions();
        assert_eq!(descs.len(), 2);
        assert!(descs.iter().all(|&d| d == "increment"));
    }

    #[test]
    fn test_multi_undo_redo_sequence() {
        let counter = Arc::new(Mutex::new(0));
        let mut buf = make_buf();
        for i in 1..=5 {
            buf.execute(IncrementCommand { counter: Arc::clone(&counter), delta: i });
        }
        assert_eq!(*counter.lock().unwrap(), 15);
        for _ in 0..3 { buf.undo(); }
        assert_eq!(*counter.lock().unwrap(), 3);
        for _ in 0..2 { buf.redo(); }
        assert_eq!(*counter.lock().unwrap(), 10);
    }

    #[test]
    fn test_set_max_history_shrinks() {
        let counter = Arc::new(Mutex::new(0));
        let mut buf = make_buf();
        for i in 0..8 {
            buf.execute(IncrementCommand { counter: Arc::clone(&counter), delta: i });
        }
        assert_eq!(buf.undo_count(), 8);
        buf.set_max_history(4);
        assert_eq!(buf.undo_count(), 4);
    }
}
