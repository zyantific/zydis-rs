//! Register helper functions.

use gen::*;
use std::ffi::CStr;
use std::ptr;
use std::borrow::Cow;


/// Returns the register specified by `class` and `id`.
///
/// # Examples
/// ```
/// let eax = zydis::register::encode(zydis::gen::ZYDIS_REGCLASS_GPR32, 0);
/// assert_eq!(eax, zydis::gen::ZYDIS_REGISTER_EAX);
/// ```
pub fn encode(class: ZydisRegisterClasses, id: u8) -> ZydisRegisters {
    unsafe { ZydisRegisterEncode(class as ZydisRegisterClass, id) as ZydisRegisters }
}

/// Returns the ID of the specified register.
///
/// # Examples
/// ```
/// let id = zydis::register::get_id(zydis::gen::ZYDIS_REGISTER_RAX);
/// assert_eq!(id, 0);
/// ```
pub fn get_id(reg: ZydisRegisters) -> i16 {
    unsafe { ZydisRegisterGetId(reg as ZydisRegister) }
}

/// Returns the register-class of the specified register.
///
/// # Examples
/// ```
/// let ecx_class = zydis::register::get_class(zydis::gen::ZYDIS_REGISTER_ECX);
/// assert_eq!(ecx_class, zydis::gen::ZYDIS_REGCLASS_GPR32);
/// ```
pub fn get_class(reg: ZydisRegisters) -> ZydisRegisterClasses {
    unsafe { ZydisRegisterGetClass(reg as ZydisRegister) as ZydisRegisterClasses }
}

/// Returns the width of the specified register.
///
/// # Examples
/// ```
/// let dr0_width = zydis::register::get_width(zydis::gen::ZYDIS_REGISTER_DR0);
/// assert_eq!(dr0_width, 32);
/// ```
pub fn get_width(reg: ZydisRegisters) -> ZydisRegisterWidth {
    unsafe { ZydisRegisterGetWidth(reg as ZydisRegister) }
}

/// Returns the width of the specified register in 64-bit mode.
///
/// # Examples
/// ```
/// let dr0_width = zydis::register::get_width64(zydis::gen::ZYDIS_REGISTER_DR0);
/// assert_eq!(dr0_width, 64);
/// ```
pub fn get_width64(reg: ZydisRegisters) -> ZydisRegisterWidth {
    unsafe { ZydisRegisterGetWidth64(reg as ZydisRegister) }
}

/// Returns the textual representation for a register.
/// 
/// # Examples
/// ```
/// let reg_str = zydis::register::get_str(zydis::gen::ZYDIS_REGISTER_EAX);
/// assert_eq!(reg_str.unwrap(), "eax");
/// ```
pub fn get_str(reg: ZydisRegisters) -> Option<Cow<'static, str>> {
    unsafe {
        let ptr = ZydisRegisterGetString(reg as ZydisRegister);
        if ptr == ptr::null() { return None }
        Some(CStr::from_ptr(ptr).to_string_lossy())
    }
}