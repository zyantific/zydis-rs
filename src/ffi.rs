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

mod decoder;
#[cfg(feature = "encoder")]
mod encoder;
#[cfg(feature = "formatter")]
mod formatter;
mod misc;
mod utils;
mod zycore;

pub use decoder::*;
#[cfg(feature = "encoder")]
pub use encoder::*;
#[cfg(feature = "formatter")]
pub use formatter::*;
pub use misc::*;
pub use utils::*;
pub use zycore::*;
