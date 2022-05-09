//! Provides type aliases, struct definitions and the unsafe function
//! declarations.

use core::{fmt, marker::PhantomData, mem::MaybeUninit, slice};

// TODO: use libc maybe, or wait for this to move into core?
use std::{
    ffi::CStr,
    os::raw::{c_char, c_void},
};

use super::{
    enums::*,
    status::{Result, Status},
};

pub mod decoder;
// pub mod encoder;
pub mod formatter;
pub mod misc;
pub mod utils;
pub mod zycore;

pub use decoder::*;
// pub use encoder::*;
pub use formatter::*;
pub use misc::*;
pub use utils::*;
pub use zycore::*;
