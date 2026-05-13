/****************************************************************************
Rust port of Cocos Creator CallbackPass / Executable
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::device_pass::DevicePassResourceTable;

pub trait Executable {
    fn execute(&self, resource_table: &DevicePassResourceTable);
}

pub struct CallbackPass<D, F>
where
    F: Fn(&D, &DevicePassResourceTable),
{
    data: D,
    execute_fn: F,
}

impl<D, F> CallbackPass<D, F>
where
    F: Fn(&D, &DevicePassResourceTable),
{
    pub fn new(data: D, execute_fn: F) -> Self {
        CallbackPass { data, execute_fn }
    }

    pub fn get_data(&self) -> &D {
        &self.data
    }

    pub fn get_data_mut(&mut self) -> &mut D {
        &mut self.data
    }
}

impl<D, F> Executable for CallbackPass<D, F>
where
    F: Fn(&D, &DevicePassResourceTable),
{
    fn execute(&self, resource_table: &DevicePassResourceTable) {
        (self.execute_fn)(&self.data, resource_table);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PassData {
        value: i32,
    }

    #[test]
    fn test_callback_pass() {
        let data = PassData { value: 42 };
        let rt = DevicePassResourceTable::new();
        let pass = CallbackPass::new(data, |d, _rt| {
            assert_eq!(d.value, 42);
        });
        pass.execute(&rt);
    }

    #[test]
    fn test_callback_pass_mutate() {
        let data = PassData { value: 0 };
        let mut pass = CallbackPass::new(data, |_d, _rt| {});
        pass.get_data_mut().value = 10;
        assert_eq!(pass.get_data().value, 10);
    }
}
