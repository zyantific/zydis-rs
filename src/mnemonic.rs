//! Mnemonic helper functions.

use gen::*;
use std::ffi::CStr;

/// Extensions for `ZydisMnemonic`
pub trait ZydisMnemonicMethods {
    /// Given a mnemonic ID, returns the corresponding string.
    ///
    /// # Examples
    /// ```
    /// use zydis::mnemonic::ZydisMnemonicMethods;
    /// let mnem = zydis::gen::ZYDIS_MNEMONIC_CMOVP.get_string();
    /// assert_eq!(mnem.unwrap(), "cmovp");
    /// ```
    fn get_string(self) -> Option<&'static str>;
}

impl ZydisMnemonicMethods for ZydisMnemonics {
    fn get_string(self) -> Option<&'static str> {
        unsafe { check_string!(ZydisMnemonicGetString(self as _)) }
    }
}
