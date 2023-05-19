//! Provides the types, enums, constants and functions of the raw, unwrapped C library.
//!
//! This is essentially what would usually be in a separate `...-sys` crate.

#[allow(unused_imports)]
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
#[cfg(feature = "encoder")]
pub mod encoder;
#[cfg(feature = "formatter")]
pub mod formatter;
pub mod misc;
pub mod utils;
pub mod zycore;

pub use decoder::*;
#[cfg(feature = "encoder")]
pub use encoder::*;
#[cfg(feature = "formatter")]
pub use formatter::*;
pub use misc::*;
pub use utils::*;
pub use zycore::*;
