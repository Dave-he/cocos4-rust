/****************************************************************************
Rust port of Cocos Creator GFX Command Buffer
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::{CommandBufferType, QueueType, Rect, Viewport};

#[derive(Debug, Clone)]
pub struct CommandBufferInfo {
    pub queue_type: QueueType,
    pub buffer_type: CommandBufferType,
}

impl Default for CommandBufferInfo {
    fn default() -> Self {
        CommandBufferInfo {
            queue_type: QueueType::Graphics,
            buffer_type: CommandBufferType::Primary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandBufferState {
    Idle,
    Recording,
    Executable,
    Pending,
}

#[derive(Debug)]
pub struct GfxCommandBuffer {
    pub id: u32,
    pub info: CommandBufferInfo,
    pub state: CommandBufferState,
    pub num_draw_calls: u32,
    pub num_instances: u32,
    pub num_tris: u32,
}

impl GfxCommandBuffer {
    pub fn new(id: u32, info: CommandBufferInfo) -> Self {
        GfxCommandBuffer {
            id,
            info,
            state: CommandBufferState::Idle,
            num_draw_calls: 0,
            num_instances: 0,
            num_tris: 0,
        }
    }

    pub fn begin(&mut self) {
        self.state = CommandBufferState::Recording;
        self.num_draw_calls = 0;
        self.num_instances = 0;
        self.num_tris = 0;
    }

    pub fn end(&mut self) {
        assert_eq!(self.state, CommandBufferState::Recording);
        self.state = CommandBufferState::Executable;
    }

    pub fn set_viewport(&mut self, _viewport: &Viewport) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn set_scissor(&mut self, _rect: &Rect) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn draw(&mut self, index_count: u32, instance_count: u32) {
        assert_eq!(self.state, CommandBufferState::Recording);
        self.num_draw_calls += 1;
        self.num_instances += instance_count;
        self.num_tris += index_count / 3 * instance_count;
    }

    pub fn is_recording(&self) -> bool {
        self.state == CommandBufferState::Recording
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_buffer_begin_end() {
        let mut cmd = GfxCommandBuffer::new(1, CommandBufferInfo::default());
        assert_eq!(cmd.state, CommandBufferState::Idle);
        cmd.begin();
        assert!(cmd.is_recording());
        cmd.end();
        assert_eq!(cmd.state, CommandBufferState::Executable);
    }

    #[test]
    fn test_command_buffer_draw() {
        let mut cmd = GfxCommandBuffer::new(1, CommandBufferInfo::default());
        cmd.begin();
        cmd.draw(6, 1);
        cmd.draw(12, 2);
        assert_eq!(cmd.num_draw_calls, 2);
        assert_eq!(cmd.num_instances, 3);
        assert_eq!(cmd.num_tris, 2 + 8);
    }
}
