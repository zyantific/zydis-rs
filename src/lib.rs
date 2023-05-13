#![doc = include_str!("../README.md")]

#[macro_use]
pub mod status;
mod decoder;
pub mod enums;
pub mod ffi;
#[cfg(not(feature = "minimal"))]
pub mod formatter;

pub use decoder::{AllOperands, Decoder, NoOperands, VisibleOperands};
pub use enums::*;
pub use status::{Result, Status};

#[cfg(not(feature = "minimal"))]
pub use formatter::{
    Formatter, FormatterProperty, Hook, OutputBuffer, WrappedDecoratorFunc, WrappedGeneralFunc,
    WrappedRegisterFunc,
};

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
pub fn get_version() -> (u16, u16, u16, u16) {
    let combined_ver = unsafe { ffi::ZydisGetVersion() };
    let major = ((combined_ver << 0) >> 48) as u16;
    let minor = ((combined_ver << 16) >> 48) as u16;
    let patch = ((combined_ver << 32) >> 48) as u16;
    let build = ((combined_ver << 48) >> 48) as u16;
    (major, minor, patch, build)
}
