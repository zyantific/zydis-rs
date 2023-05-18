//! Provides the types, enums, constants and functions of the raw, unwrapped C library.
//!
//! This is essentially what would usually be in a separate `...-sys` crate.

use core::{
    ffi::{c_char, c_void, CStr},
    fmt,
    marker::PhantomData,
    mem::MaybeUninit,
    slice,
};

use super::{
    enums::*,
    status::{Result, Status},
};

pub mod decoder;
pub mod encoder;
pub mod formatter;
pub mod misc;
pub mod utils;
pub mod zycore;

pub use decoder::*;
pub use encoder::*;
pub use formatter::*;
pub use misc::*;
pub use utils::*;
pub use zycore::*;
