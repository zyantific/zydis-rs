//! The official Rust bindings for the Zyan Disassembler Engine.

#![deny(bare_trait_objects)]

pub mod gen;
#[macro_use]
pub mod status;
pub mod decoder;
pub mod formatter;
pub mod mnemonic;
pub mod register;
pub mod utils;

pub use decoder::*;
pub use formatter::*;
pub use mnemonic::*;
pub use register::*;
pub use status::*;
pub use utils::*;

/// Returns the version of the zydis C library as a quadruple `(major, minor, patch, build)`
/// 
/// # Examples
/// ```
/// use zydis;
/// let (major, minor, patch, build) = zydis::get_version();
/// println!("Zydis version: {}.{}.{}.{}", major, minor, patch, build);
/// ```
pub fn get_version() -> (u16, u16, u16, u16) {
    let combined_ver = unsafe {
        gen::ZydisGetVersion()
    };
    let major = ((combined_ver << 0) >> 48) as u16;
    let minor = ((combined_ver << 16) >> 48) as u16;
    let patch = ((combined_ver << 32) >> 48) as u16;
    let build = ((combined_ver << 48) >> 48) as u16;
    (major, minor, patch, build)
}
