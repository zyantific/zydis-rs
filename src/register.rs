//! Register helper functions.

use std::ffi::CStr;

use gen::*;

/// Extensions for `Register`
pub trait RegisterMethods {
    /// Returns the ID of the specified register.
    ///
    /// Returns `None` if the register is invalid.
    ///
    /// # Examples
    /// ```
    /// use zydis::register::RegisterMethods;
    /// let id = zydis::gen::ZYDIS_REGISTER_RAX.get_id();
    /// assert_eq!(id, Some(0));
    /// ```
    fn get_id(self) -> Option<i16>;

    /// Returns the register-class of the specified register.
    ///
    /// # Examples
    /// ```
    /// use zydis::register::RegisterMethods;
    /// let ecx_class = zydis::gen::ZYDIS_REGISTER_ECX.get_class();
    /// assert_eq!(ecx_class, zydis::gen::ZYDIS_REGCLASS_GPR32);
    /// ```
    fn get_class(self) -> ZydisRegisterClass;

    /// Returns the textual representation for a register.
    ///
    /// # Examples
    /// ```
    /// use zydis::register::RegisterMethods;
    /// let reg_str = zydis::gen::ZYDIS_REGISTER_EAX.get_string();
    /// assert_eq!(reg_str.unwrap(), "eax");
    /// ```
    fn get_string(self) -> Option<&'static str>;

    /// Returns the width of the specified register.
    ///
    /// # Examples
    /// ```
    /// use zydis::register::RegisterMethods;
    /// let dr0_width = zydis::gen::ZYDIS_REGISTER_DR0.get_width();
    /// assert_eq!(dr0_width, 32);
    /// ```
    fn get_width(self) -> ZydisRegisterWidth;

    /// Returns the width of the specified register in 64-bit mode.
    ///
    /// # Examples
    /// ```
    /// use zydis::register::RegisterMethods;
    /// let dr0_width = zydis::gen::ZYDIS_REGISTER_DR0.get_width64();
    /// assert_eq!(dr0_width, 64);
    /// ```
    fn get_width64(self) -> ZydisRegisterWidth;
}

impl RegisterMethods for Register {
    fn get_id(self) -> Option<i16> {
        unsafe {
            match ZydisRegisterGetId(self as _) {
                -1 => None,
                x => Some(x),
            }
        }
    }

    fn get_class(self) -> ZydisRegisterClass {
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

/// Extensions for `RegisterClass`.
pub trait RegisterClassExtensions {
    /// Returns the register specified by `class` and `id`.
    ///
    /// # Examples
    /// ```
    /// use zydis::register::ZydisRegisterClassExtensions;
    /// let eax = zydis::gen::ZYDIS_REGCLASS_GPR32.encode(0);
    /// assert_eq!(eax, zydis::gen::ZYDIS_REGISTER_EAX);
    /// ```
    fn encode(self, id: u8) -> ZydisRegister;
}

impl RegisterClassExtensions for RegisterClass {
    fn encode(self, id: u8) -> ZydisRegister {
        unsafe { ZydisRegisterEncode(self as _, id) as _ }
    }
}
