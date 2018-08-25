//! Mnemonic helper functions.

use std::ffi::CStr;

use gen::*;

/// Extensions for `Mnemonic`
pub trait MnemonicMethods {
    /// Given a mnemonic ID, returns the corresponding string.
    ///
    /// # Examples
    /// ```
    /// use zydis::mnemonic::MnemonicMethods;
    /// let mnem = zydis::gen::ZYDIS_MNEMONIC_CMOVP.get_string();
    /// assert_eq!(mnem.unwrap(), "cmovp");
    /// ```
    fn get_string(self) -> Option<&'static str>;
}

impl MnemonicMethods for Mnemonic {
    fn get_string(self) -> Option<&'static str> {
        unsafe { check_string!(ZydisMnemonicGetString(self as _)) }
    }
}
