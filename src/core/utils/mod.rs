/****************************************************************************
Rust port of Cocos Creator core/utils module
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

pub mod id_generator;
pub mod path;

pub use id_generator::{IDGenerator, NON_UUID_MARK, get_new_global_id};
pub use path::{
    join, extname, main_filename, basename, dirname,
    change_extname, change_basename, normalize, strip_sep, get_separator,
};
