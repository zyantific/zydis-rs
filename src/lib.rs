// The doc-test in README.md needs formatter
#![cfg_attr(feature = "formatter", doc = include_str!("../README.md"))]

#[macro_use]
mod status;
mod decoder;
mod enums;
pub mod ffi;
#[cfg(feature = "formatter")]
mod formatter;

#[cfg(feature = "full-decoder")]
pub use decoder::{AllOperands, VisibleOperands};
pub use decoder::{Decoder, NoOperands};
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
