/****************************************************************************
Rust port of Cocos Creator core/utils module
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

pub mod id_generator;
pub mod path;

pub use id_generator::{get_new_global_id, IDGenerator, NON_UUID_MARK};
pub use path::{
    basename, change_basename, change_extname, dirname, extname, get_separator, join,
    main_filename, normalize, strip_sep,
};
