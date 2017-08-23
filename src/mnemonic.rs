//! Mnemonic helper functions.

use gen::*;
use std::ffi::CStr;
use std::ptr;
use std::borrow::Cow;


/// Given a mnemonic ID, returns the corresponding string.
///
/// # Examples
/// ```
/// let mnem = zydis::mnemonic::get_str(zydis::gen::ZYDIS_MNEMONIC_CMOVP);
/// assert_eq!(mnem.unwrap(), "cmovp");
/// ```
pub fn get_str(mnem: u32) -> Option<Cow<'static, str>> {
    unsafe {
        let ptr = ZydisMnemonicGetString(mnem as ZydisMnemonic);
        if ptr == ptr::null() {
            return None;
        }
        Some(CStr::from_ptr(ptr).to_string_lossy())
    }
}