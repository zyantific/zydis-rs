//! Contains definition for all enums used in zydis and some utility functions
//! on them.
#![allow(non_camel_case_types)]

use core::fmt;

use bitflags::bitflags;

pub mod generated;

pub use self::generated::*;

use super::ffi;

impl Mnemonic {
    /// Returns a string corresponding to this mnemonic.
    ///
    /// # Examples
    /// ```
    /// use zydis::Mnemonic;
    /// let str = Mnemonic::CMOVP.get_string().unwrap();
    /// assert_eq!("cmovp", str);
    /// ```
    pub fn get_string(self) -> Option<&'static str> {
        unsafe { check_string!(ffi::ZydisMnemonicGetString(self)) }
    }
}

impl Register {
    /// Returns the ID of this register.
    ///
    /// # Examples
    /// ```
    /// use zydis::Register;
    /// assert_eq!(0, Register::RAX.get_id());
    /// ```
    pub fn get_id(self) -> u8 {
        unsafe { ffi::ZydisRegisterGetId(self) as u8 }
    }

    /// Returns the register-class of this register.
    ///
    /// # Examples
    /// ```
    /// use zydis::{Register, RegisterClass};
    ///
    /// let class = Register::ECX.get_class();
    /// assert_eq!(RegisterClass::GPR32, class);
    /// ```
    pub fn get_class(self) -> RegisterClass {
        unsafe { ffi::ZydisRegisterGetClass(self) }
    }

    /// Returns the textual representation of this register.
    ///
    /// # Examples
    /// ```
    /// use zydis::Register;
    ///
    /// let str = Register::EAX.get_string().unwrap();
    /// assert_eq!("eax", str);
    /// ```
    pub fn get_string(self) -> Option<&'static str> {
        unsafe { check_string!(ffi::ZydisRegisterGetString(self)) }
    }

    /// Returns the width of this register, in bits.
    ///
    /// # Examples
    /// ```
    /// use zydis::{MachineMode, Register};
    ///
    /// let width = Register::DR0.get_width(MachineMode::LEGACY_32);
    /// assert_eq!(32, width);
    /// ```
    pub fn get_width(self, mode: MachineMode) -> ffi::RegisterWidth {
        unsafe { ffi::ZydisRegisterGetWidth(mode, self) }
    }

    /// Returns the largest enclosing register of the given register.
    ///
    /// # Examples
    /// ```
    /// use zydis::{MachineMode, Register};
    ///
    /// let reg = Register::EAX.get_largest_enclosing(MachineMode::LONG_64);
    /// assert_eq!(reg, Register::RAX);
    /// ```
    pub fn get_largest_enclosing(self, mode: MachineMode) -> Register {
        unsafe { ffi::ZydisRegisterGetLargestEnclosing(mode, self) }
    }
}

impl RegisterClass {
    /// Returns the register specified by this register class and `id`.
    ///
    /// # Examples
    /// ```
    /// use zydis::{Register, RegisterClass};
    /// let eax = RegisterClass::GPR32.encode(0);
    /// assert_eq!(Register::EAX, eax);
    /// ```
    pub fn encode(self, id: u8) -> Register {
        unsafe { ffi::ZydisRegisterEncode(self, id) }
    }

    /// Returns the width of the specified register-class.
    pub fn get_width(self, mode: MachineMode) -> ffi::RegisterWidth {
        unsafe { ffi::ZydisRegisterClassGetWidth(mode, self) }
    }
}

/// The type of a formatter token.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct Token(pub u8);

pub const TOKEN_INVALID: Token = Token(0x0);
pub const TOKEN_WHITESPACE: Token = Token(0x1);
pub const TOKEN_DELIMITER: Token = Token(0x2);
pub const TOKEN_PARENTHESIS_OPEN: Token = Token(0x3);
pub const TOKEN_PARENTHESIS_CLOSE: Token = Token(0x4);
pub const TOKEN_PREFIX: Token = Token(0x5);
pub const TOKEN_MNEMONIC: Token = Token(0x6);
pub const TOKEN_REGISTER: Token = Token(0x7);
pub const TOKEN_ADDRESS_ABS: Token = Token(0x8);
pub const TOKEN_ADDRESS_REL: Token = Token(0x9);
pub const TOKEN_DISPLACEMENT: Token = Token(0xA);
pub const TOKEN_IMMEDIATE: Token = Token(0xB);
pub const TOKEN_TYPECAST: Token = Token(0xC);
pub const TOKEN_DECORATOR: Token = Token(0xD);
pub const TOKEN_SYMBOL: Token = Token(0xE);
/// The base for user defined tokens.
pub const TOKEN_USER: Token = Token(0x80);

static TOKEN_NAMES: [&'static str; 0xF] = [
    "invalid",
    "whitespace",
    "delimiter",
    "opening parenthesis",
    "closing parenthesis",
    "prefix",
    "mnemonic",
    "register",
    "absolute address",
    "relative address",
    "displacement",
    "immediate",
    "typecast",
    "decorator",
    "symbol",
];

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 <= 0xE {
            write!(f, "{}", TOKEN_NAMES[self.0 as usize])
        } else {
            write!(f, "<unknown>")
        }
    }
}

bitflags! {
    #[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
    #[repr(transparent)]
    pub struct OperandAction: u32 {
        const READ =  1;
        const WRITE = 2;
        const CONDREAD = 4;
        const CONDWRITE = 8;
        const READWRITE = 3;
        const CONDREAD_CONDWRITE = 12;
        const READ_CONDWRITE = 9;
        const CONDREAD_WRITE = 6;
        const MASK_READ = 5;
        const MASK_WRITE = 10;
    }
}

bitflags! {
    #[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
    #[repr(transparent)]
    pub struct InstructionAttributes: u64 {
        const HAS_MODRM                 = 1 << 0;
        const HAS_SIB                   = 1 << 1;
        const HAS_REX                   = 1 << 2;
        const HAS_XOP                   = 1 << 3;
        const HAS_VEX                   = 1 << 4;
        const HAS_EVEX                  = 1 << 5;
        const HAS_MVEX                  = 1 << 6;
        const IS_RELATIVE               = 1 << 7;
        const IS_PRIVILEGED             = 1 << 8;
        const ACCEPTS_LOCK              = 1 << 9;
        const ACCEPTS_REP               = 1 << 10;
        const ACCEPTS_REPE              = 1 << 11;
        const ACCEPTS_REPZ              = 1 << 11;
        const ACCEPTS_REPNE             = 1 << 12;
        const ACCEPTS_REPNZ             = 1 << 12;
        const ACCEPTS_BND               = 1 << 13;
        const ACCEPTS_XACQUIRE          = 1 << 14;
        const ACCEPTS_XRELEASE          = 1 << 15;
        const ACCEPTS_HLE_WITHOUT_LOCK  = 1 << 16;
        const ACCEPTS_BRANCH_HINTS      = 1 << 17;
        const ACCEPTS_SEGMENT           = 1 << 18;
        const HAS_LOCK                  = 1 << 19;
        const HAS_REP                   = 1 << 20;
        const HAS_REPE                  = 1 << 21;
        const HAS_REPZ                  = 1 << 21;
        const HAS_REPNE                 = 1 << 22;
        const HAS_REPNZ                 = 1 << 22;
        const HAS_BND                   = 1 << 23;
        const HAS_XACQUIRE              = 1 << 24;
        const HAS_XRELEASE              = 1 << 25;
        const HAS_BRANCH_NOT_TAKEN      = 1 << 26;
        const HAS_BRNACH_TAKEN          = 1 << 27;
        const HAS_SEGMENT_CS            = 1 << 28;
        const HAS_SEGMENT_SS            = 1 << 29;
        const HAS_SEGMENT_DS            = 1 << 30;
        const HAS_SEGMENT_ES            = 1 << 31;
        const HAS_SEGMENT_FS            = 1 << 32;
        const HAS_SEGMENT_GS            = 1 << 33;
        const HAS_SEGMENT               =
              InstructionAttributes::HAS_SEGMENT_CS.bits
            | InstructionAttributes::HAS_SEGMENT_SS.bits
            | InstructionAttributes::HAS_SEGMENT_DS.bits
            | InstructionAttributes::HAS_SEGMENT_ES.bits
            | InstructionAttributes::HAS_SEGMENT_FS.bits
            | InstructionAttributes::HAS_SEGMENT_GS.bits;
        const HAS_OPERANDSIZE           = 1 << 34;
        const HAS_ADDRESSIZE            = 1 << 35;
        const CPUFLAG_ACCESS            = 1 << 36;
        const CPU_STATE_CR              = 1 << 37;
        const CPU_STATE_CW              = 1 << 38;
        const FPU_STATE_CR              = 1 << 39;
        const FPU_STATE_CW              = 1 << 40;
        const XMM_STATE_CR              = 1 << 41;
        const XMM_STATE_CW              = 1 << 42;
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_encoding() {
        const CODE: &'static [u8] = &[0xE8, 0xFB, 0xFF, 0xFF, 0xFF];

        let decoder = Decoder::new(MachineMode::LONG_COMPAT_32, AddressWidth::_32).unwrap();
        let (insn, _) = decoder.instruction_iterator(CODE, 0x0).next().unwrap();
        assert_eq!(insn.operands[0].encoding, OperandEncoding::JIMM16_32_32);
    }
}
