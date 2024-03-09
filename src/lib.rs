// The doc-test in README.md needs formatter
#![cfg_attr(feature = "formatter", doc = include_str!("../README.md"))]
#![cfg_attr(not(feature = "std"), no_std)]

//! ## Navigation
//!
//! - [Decode instructions and process them programmatically][`Decoder`]
//! - [Format instructions to human-readable text][`Formatter`]
//! - [Decode, change and re-encode instructions][`EncoderRequest`]
//! - [Encode new instructions from scratch][`EncoderRequest`]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod status;
mod decoder;
#[cfg(feature = "encoder")]
mod encoder;
mod enums;
pub mod ffi;
#[cfg(feature = "formatter")]
mod formatter;

pub use decoder::*;
#[cfg(feature = "encoder")]
pub use encoder::*;
pub use enums::*;
#[cfg(feature = "formatter")]
pub use formatter::*;
pub use status::*;

/// Returns the version of the zydis C library as a quadruple
/// `(major, minor, patch, build)`.
///
/// # Examples
///
/// ```
/// let (major, minor, patch, build) = zydis::get_version();
/// println!("Zydis version: {}.{}.{}.{}", major, minor, patch, build);
/// assert_eq!(major, 4);
/// ```
pub fn version() -> (u16, u16, u16, u16) {
    let combined_ver = unsafe { ffi::ZydisGetVersion() };
    let major = ((combined_ver << 0) >> 48) as u16;
    let minor = ((combined_ver << 16) >> 48) as u16;
    let patch = ((combined_ver << 32) >> 48) as u16;
    let build = ((combined_ver << 48) >> 48) as u16;
    (major, minor, patch, build)
}

#[doc(hidden)]
#[deprecated(since = "4.0.0", note = "use `version()` instead")]
pub fn get_version() -> (u16, u16, u16, u16) {
    version()
}
