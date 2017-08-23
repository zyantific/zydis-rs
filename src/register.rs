//! Register helper functions.

use gen::*;
use std::ffi::CStr;

/// Extensions for `ZydisRegister`
pub trait ZydisRegisterMethods {
    /// Returns the ID of the specified register.
    ///
    /// Returns `None` if the register is invalid.
    ///
    /// # Examples
    /// ```
    /// use zydis::register::ZydisRegisterMethods;
    /// let id = zydis::gen::ZYDIS_REGISTER_RAX.get_id();
    /// assert_eq!(id, Some(0));
    /// ```
    fn get_id(self) -> Option<i16>;

    /// Returns the register-class of the specified register.
    ///
    /// # Examples
    /// ```
    /// use zydis::register::ZydisRegisterMethods;
    /// let ecx_class = zydis::gen::ZYDIS_REGISTER_ECX.get_class();
    /// assert_eq!(ecx_class, zydis::gen::ZYDIS_REGCLASS_GPR32);
    /// ```
    fn get_class(self) -> ZydisRegisterClasses;

    /// Returns the textual representation for a register.
    ///
    /// # Examples
    /// ```
    /// use zydis::register::ZydisRegisterMethods;
    /// let reg_str = zydis::gen::ZYDIS_REGISTER_EAX.get_string();
    /// assert_eq!(reg_str.unwrap(), "eax");
    /// ```
    fn get_string(self) -> Option<&'static str>;

    /// Returns the width of the specified register.
    ///
    /// # Examples
    /// ```
    /// use zydis::register::ZydisRegisterMethods;
    /// let dr0_width = zydis::gen::ZYDIS_REGISTER_DR0.get_width();
    /// assert_eq!(dr0_width, 32);
    /// ```
    fn get_width(self) -> ZydisRegisterWidth;

    /// Returns the width of the specified register in 64-bit mode.
    ///
    /// # Examples
    /// ```
    /// use zydis::register::ZydisRegisterMethods;
    /// let dr0_width = zydis::gen::ZYDIS_REGISTER_DR0.get_width64();
    /// assert_eq!(dr0_width, 64);
    /// ```
    fn get_width64(self) -> ZydisRegisterWidth;
}

impl ZydisRegisterMethods for ZydisRegisters {
    fn get_id(self) -> Option<i16> {
        unsafe {
            match ZydisRegisterGetId(self as _) {
                -1 => None,
                x => Some(x),
            }
        }
    }

    fn get_class(self) -> ZydisRegisterClasses {
        unsafe { ZydisRegisterGetClass(self as _) as _ }
    }

    fn get_string(self) -> Option<&'static str> {
        unsafe { check_string!(ZydisRegisterGetString(self as _)) }
    }

    fn get_width(self) -> ZydisRegisterWidth {
        unsafe { ZydisRegisterGetWidth(self as _) }
    }

    fn get_width64(self) -> ZydisRegisterWidth {
        unsafe { ZydisRegisterGetWidth64(self as _) }
    }
}

/// Extensions for `ZydisRegisterClassesExtensions`.
pub trait ZydisRegisterClassesExtensions {
    /// Returns the register specified by `class` and `id`.
    ///
    /// # Examples
    /// ```
    /// use zydis::register::ZydisRegisterClassesExtensions;
    /// let eax = zydis::gen::ZYDIS_REGCLASS_GPR32.encode(0);
    /// assert_eq!(eax, zydis::gen::ZYDIS_REGISTER_EAX);
    /// ```
    fn encode(self, id: u8) -> ZydisRegisters;
}

impl ZydisRegisterClassesExtensions for ZydisRegisterClasses {
    fn encode(self, id: u8) -> ZydisRegisters {
        unsafe { ZydisRegisterEncode(self as _, id) as _ }
    }
}
