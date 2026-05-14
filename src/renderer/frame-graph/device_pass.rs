/****************************************************************************
Rust port of Cocos Creator DevicePass + DevicePassResourceTable
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::callback_pass::Executable;
use super::pass::Handle;
use crate::renderer::gfx_base::{Rect, Viewport};
use std::collections::HashMap;

pub struct DevicePassResourceTable {
    reads: HashMap<u16, u32>,
    writes: HashMap<u16, u32>,
    subpass_index: u32,
}

impl DevicePassResourceTable {
    pub fn new() -> Self {
        DevicePassResourceTable {
            reads: HashMap::new(),
            writes: HashMap::new(),
            subpass_index: 0,
        }
    }

    pub fn from_pass_node(reads: &[Handle], writes: &[Handle], subpass_index: u32) -> Self {
        let mut rt = DevicePassResourceTable::new();
        rt.subpass_index = subpass_index;
        for h in reads {
            rt.reads.insert(h.index, 0);
        }
        for h in writes {
            rt.writes.insert(h.index, 0);
        }
        rt
    }

    pub fn get_read(&self, handle: Handle) -> Option<u32> {
        self.reads.get(&handle.index).copied()
    }

    pub fn get_write(&self, handle: Handle) -> Option<u32> {
        self.writes.get(&handle.index).copied()
    }

    pub fn get_subpass_index(&self) -> u32 {
        self.subpass_index
    }

    pub fn set_read(&mut self, handle: Handle, resource_id: u32) {
        self.reads.insert(handle.index, resource_id);
    }

    pub fn set_write(&mut self, handle: Handle, resource_id: u32) {
        self.writes.insert(handle.index, resource_id);
    }
}

impl Default for DevicePassResourceTable {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LogicPass {
    executable: Option<Box<dyn Executable>>,
    custom_viewport: bool,
    viewport: Viewport,
    scissor: Rect,
}

impl LogicPass {
    pub fn new(
        executable: Option<Box<dyn Executable>>,
        custom_viewport: bool,
        viewport: Viewport,
        scissor: Rect,
    ) -> Self {
        LogicPass {
            executable,
            custom_viewport,
            viewport,
            scissor,
        }
    }

    pub fn execute(&self, resource_table: &DevicePassResourceTable) {
        if let Some(ref exec) = self.executable {
            exec.execute(resource_table);
        }
    }

    pub fn has_custom_viewport(&self) -> bool {
        self.custom_viewport
    }

    pub fn get_viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn get_scissor(&self) -> &Rect {
        &self.scissor
    }
}

pub struct Subpass {
    desc: u32,
    logic_passes: Vec<LogicPass>,
    barrier_id: u32,
}

impl Subpass {
    pub fn new(desc: u32, barrier_id: u32) -> Self {
        Subpass {
            desc,
            logic_passes: Vec::new(),
            barrier_id,
        }
    }

    pub fn add_logic_pass(&mut self, pass: LogicPass) {
        self.logic_passes.push(pass);
    }

    pub fn execute(&self, resource_table: &DevicePassResourceTable) {
        for lp in &self.logic_passes {
            lp.execute(resource_table);
        }
    }

    pub fn get_logic_passes(&self) -> &[LogicPass] {
        &self.logic_passes
    }

    pub fn get_barrier_id(&self) -> u32 {
        self.barrier_id
    }
}

pub struct Attachment {
    attachment: u32,
    render_target: u32,
}

impl Attachment {
    pub fn new(attachment: u32, render_target: u32) -> Self {
        Attachment {
            attachment,
            render_target,
        }
    }

    pub fn get_attachment(&self) -> u32 {
        self.attachment
    }

    pub fn get_render_target(&self) -> u32 {
        self.render_target
    }
}

pub struct DevicePass {
    subpasses: Vec<Subpass>,
    attachments: Vec<Attachment>,
    used_render_target_slot_mask: u16,
    resource_table: DevicePassResourceTable,
    viewport: Viewport,
    scissor: Rect,
    render_pass_handle: Handle,
    fbo_handle: Handle,
}

impl DevicePass {
    pub fn new(
        subpasses: Vec<Subpass>,
        attachments: Vec<Attachment>,
        used_render_target_slot_mask: u16,
        resource_table: DevicePassResourceTable,
        viewport: Viewport,
        scissor: Rect,
        render_pass_handle: Handle,
        fbo_handle: Handle,
    ) -> Self {
        DevicePass {
            subpasses,
            attachments,
            used_render_target_slot_mask,
            resource_table,
            viewport,
            scissor,
            render_pass_handle,
            fbo_handle,
        }
    }

    pub fn execute(&self) {
        self.begin();
        for subpass in &self.subpasses {
            subpass.execute(&self.resource_table);
            self.next_subpass();
        }
        self.end();
    }

    fn begin(&self) {}

    fn next_subpass(&self) {}

    fn end(&self) {}

    pub fn get_subpasses(&self) -> &[Subpass] {
        &self.subpasses
    }

    pub fn get_attachments(&self) -> &[Attachment] {
        &self.attachments
    }

    pub fn get_resource_table(&self) -> &DevicePassResourceTable {
        &self.resource_table
    }

    pub fn get_viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn get_scissor(&self) -> &Rect {
        &self.scissor
    }

    pub fn get_render_pass_handle(&self) -> Handle {
        self.render_pass_handle
    }

    pub fn get_fbo_handle(&self) -> Handle {
        self.fbo_handle
    }
}

#[cfg(test)]
mod tests {
    use super::super::callback_pass::CallbackPass;
    use super::*;

    #[test]
    fn test_device_pass_resource_table() {
        let rt = DevicePassResourceTable::new();
        assert!(rt.get_read(Handle::new(0)).is_none());
        assert!(rt.get_write(Handle::new(0)).is_none());
        assert_eq!(rt.get_subpass_index(), 0);
    }

    #[test]
    fn test_device_pass_resource_table_set() {
        let mut rt = DevicePassResourceTable::new();
        rt.set_read(Handle::new(0), 100);
        rt.set_write(Handle::new(1), 200);
        assert_eq!(rt.get_read(Handle::new(0)), Some(100));
        assert_eq!(rt.get_write(Handle::new(1)), Some(200));
    }

    #[test]
    fn test_device_pass_resource_table_from_pass() {
        let reads = [Handle::new(0), Handle::new(1)];
        let writes = [Handle::new(2)];
        let rt = DevicePassResourceTable::from_pass_node(&reads, &writes, 0);
        assert!(rt.get_read(Handle::new(0)).is_some());
        assert!(rt.get_read(Handle::new(1)).is_some());
        assert!(rt.get_write(Handle::new(2)).is_some());
    }

    #[test]
    fn test_logic_pass() {
        let data = 42i32;
        let pass = CallbackPass::new(data, |d, _rt| {
            assert_eq!(*d, 42);
        });
        let lp = LogicPass::new(
            Some(Box::new(pass)),
            false,
            Viewport::default(),
            Rect::default(),
        );
        let rt = DevicePassResourceTable::new();
        lp.execute(&rt);
    }

    #[test]
    fn test_subpass() {
        let data = 10i32;
        let pass = CallbackPass::new(data, |d, _rt| {
            assert_eq!(*d, 10);
        });
        let lp = LogicPass::new(
            Some(Box::new(pass)),
            false,
            Viewport::default(),
            Rect::default(),
        );
        let mut subpass = Subpass::new(0, 0);
        subpass.add_logic_pass(lp);
        let rt = DevicePassResourceTable::new();
        subpass.execute(&rt);
        assert_eq!(subpass.get_logic_passes().len(), 1);
    }

    #[test]
    fn test_device_pass() {
        let data = 1i32;
        let pass = CallbackPass::new(data, |_d, _rt| {});
        let lp = LogicPass::new(
            Some(Box::new(pass)),
            false,
            Viewport::default(),
            Rect::default(),
        );
        let mut subpass = Subpass::new(0, 0);
        subpass.add_logic_pass(lp);
        let rt = DevicePassResourceTable::new();
        let dp = DevicePass::new(
            vec![subpass],
            Vec::new(),
            0,
            rt,
            Viewport::default(),
            Rect::default(),
            Handle::INVALID,
            Handle::INVALID,
        );
        dp.execute();
        assert_eq!(dp.get_subpasses().len(), 1);
    }

    #[test]
    fn test_attachment() {
        let a = Attachment::new(1, 2);
        assert_eq!(a.get_attachment(), 1);
        assert_eq!(a.get_render_target(), 2);
    }
}
