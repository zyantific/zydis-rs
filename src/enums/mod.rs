//! Contains definition for all enums used in zydis and some utility functions
//! on them.

// It is really not nice to write stuff like `Avx512Bitalg` instead of `AVX512_BITALG`, thus we
// sometimes use UPPERCASE where it makes sense.
#![allow(non_camel_case_types)]

use bitflags::bitflags;

pub mod instructioncategory;
pub mod isaext;
pub mod isaset;
pub mod mnemonic;
pub mod register;

use core::fmt;

pub use self::{instructioncategory::*, isaext::*, isaset::*, mnemonic::*, register::*};

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
    /// use zydis::{Register, MachineMode};
    ///
    /// let width = Register::DR0.get_width(MachineMode::Legacy32);
    /// assert_eq!(32, width);
    /// ```
    pub fn get_width(self, mode: MachineMode) -> ffi::RegisterWidth {
        unsafe { ffi::ZydisRegisterGetWidth(mode, self) }
    }

    /// Returns the largest enclosing register of the given register.
    ///
    /// # Examples
    /// ```
    /// use zydis::{Register, MachineMode};
    ///
    /// let reg = Register::EAX.get_largest_enclosing(MachineMode::Long64);
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

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum Feature {
    AVX512,
    KNC,
}

pub const FEATURE_MAX_VALUE: Feature = Feature::KNC;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum InstructionSegment {
    NONE,
    PREFIXES,
    REX,
    XOP,
    VEX,
    EVEX,
    MVEX,
    OPCODE,
    MODRM,
    SIB,
    DISPLACEMENT,
    IMMEDIATE,
}

pub const INSTRUCTION_SEGMENT_MAX_VALUE: InstructionSegment = InstructionSegment::IMMEDIATE;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum DecoderMode {
    MINIMAL,
    AMD_BRANCHES,
    KNC,
    MPX,
    CET,
    LZCNT,
    TZCNT,
    WBNOINVD,
    CLDEMOTE,
}

pub const DECODER_MODE_MAX_VALUE: DecoderMode = DecoderMode::CLDEMOTE;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum RegisterClass {
    INVALID,
    GPR8,
    GPR16,
    GPR32,
    GPR64,
    X87,
    MMX,
    XMM,
    YMM,
    ZMM,
    FLAGS,
    IP,
    SEGMENT,
    TEST,
    CONTROL,
    DEBUG,
    MASK,
    BOUND,
}

pub const REGISTER_CLASS_MAX_VALUE: RegisterClass = RegisterClass::BOUND;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum FormatterStyle {
    ATT,
    Intel,
    IntelMasm,
}

pub const FORMATTER_STYLE_MAX_VALUE: FormatterStyle = FormatterStyle::IntelMasm;

/// We wrap this in a nicer rust enum `FormatterProperty` already, use that
/// instead.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum ZydisFormatterProperty {
    FORCE_SIZE,
    FORCE_SEGMENT,
    FORCE_RELATIVE_BRANCHES,
    FORCE_RELATIVE_RIPREL,
    PRINT_BRANCH_SIZE,
    DETAILED_PREFIXES,
    ADDR_BASE,
    ADDR_SIGNEDNESS,
    ADDR_PADDING_ABSOLUTE,
    ADDR_PADDING_RELATIVE,
    DISP_BASE,
    DISP_SIGNEDNESS,
    DISP_PADDING,
    IMM_BASE,
    IMM_SIGNEDNESS,
    IMM_PADDING,
    UPPERCASE_PREFIXES,
    UPPERCASE_MNEMONIC,
    UPPERCASE_REGISTERS,
    UPPERCASE_TYPECASTS,
    UPPERCASE_DECORATORS,
    DEC_PREFIX,
    DEC_SUFFIX,
    HEX_UPPERCASE,
    HEX_PREFIX,
    HEX_SUFFIX,
}

pub const FORMATTER_PROPERTY_MAX_VALUE: ZydisFormatterProperty = ZydisFormatterProperty::HEX_SUFFIX;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum NumericBase {
    Decimal,
    Hex,
}

pub const NUMERIC_BASE_MAX_VALUE: NumericBase = NumericBase::Hex;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum Signedness {
    Auto,
    Signed,
    Unsigned,
}

pub const SIGNEDNESS_MAX_VALUE: Signedness = Signedness::Unsigned;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum Padding {
    Disabled,
    Auto = -1,
}

pub const PADDING_MAX_VALUE: Padding = Padding::Auto;

/// Use `formatter::Hook` instead.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum HookType {
    PRE_INSTRUCTION,
    POST_INSTRUCTION,
    FORMAT_INSTRUCTION,
    PRE_OPERAND,
    POST_OPERAND,
    FORMAT_OPERAND_REG,
    FORMAT_OPERAND_MEM,
    FORMAT_OPERAND_PTR,
    FORMAT_OPERAND_IMM,
    PRINT_MNEMONIC,
    PRINT_REGISTER,
    PRINT_ADDRESS_ABS,
    PRINT_ADDRESS_REL,
    PRINT_DISP,
    PRINT_IMM,
    PRINT_TYPECAST,
    PRINT_PREFIXES,
    PRINT_DECORATOR,
}

pub const HOOK_TYPE_MAX_VALUE: HookType = HookType::PRINT_DECORATOR;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum Decorator {
    INVALID,
    MASK,
    BC,
    RC,
    SAE,
    SWIZZLE,
    CONVERSION,
    EH,
}

pub const DECORATOR_MAX_VALUE: Decorator = Decorator::EH;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum MachineMode {
    Long64,
    LongCompat32,
    LongCompat16,
    Legacy32,
    Legacy16,
    Real16,
}

pub const MACHINE_MODE_MAX_VALUE: MachineMode = MachineMode::Real16;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum AddressWidth {
    _16,
    _32,
    _64,
}

pub const ADDRESS_WIDTH_MAX_VALUE: AddressWidth = AddressWidth::_64;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum ElementType {
    INVALID,
    STRUCT,
    UINT,
    INT,
    FLOAT16,
    FLOAT32,
    FLOAT64,
    FLOAT80,
    LONGBCD,
    CC,
}

pub const ELEMENT_TYPE_MAX_VALUE: ElementType = ElementType::CC;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum OperandType {
    Unused,
    Register,
    Memory,
    Pointer,
    Immediate,
}

pub const OPERAND_TYPE_MAX_VALUE: OperandType = OperandType::Immediate;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum OperandEncoding {
    NONE,
    MODRM_REG,
    MODRM_RM,
    OPCODE,
    NDSNDD,
    IS4,
    MASK,
    DISP8,
    DISP16,
    DISP32,
    DISP16_32_64,
    DISP32_32_64,
    DISP16_32_32,
    UIMM8,
    UIMM16,
    UIMM32,
    UIMM64,
    UIMM16_32_64,
    UIMM32_32_64,
    UIMM16_32_32,
    SIMM8,
    SIMM16,
    SIMM32,
    SIMM64,
    SIMM16_32_64,
    SIMM32_32_64,
    SIMM16_32_32,
    JIMM8,
    JIMM16,
    JIMM32,
    JIMM64,
    JIMM16_32_64,
    JIMM32_32_64,
    JIMM16_32_32,
}

pub const OPERAND_ENCODING_MAX_VALUE: OperandEncoding = OperandEncoding::JIMM16_32_32;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum OperandVisibility {
    Invalid,
    Explicit,
    Implicit,
    Hidden,
}

pub const OPERAND_VISIBILITY_MAX_VALUE: OperandVisibility = OperandVisibility::Hidden;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum OperandAction {
    INVALID,
    READ,
    WRITE,
    READWRITE,
    CONDREAD,
    CONDWRITE,
    READ_CONDWRITE,
    CONDREAD_WRITE,
}

pub const OPERAND_ACTION_MASK_WRITE: usize = OperandAction::WRITE as usize
    | OperandAction::READWRITE as usize
    | OperandAction::CONDWRITE as usize
    | OperandAction::READ_CONDWRITE as usize
    | OperandAction::CONDREAD_WRITE as usize;

pub const OPERAND_ACTION_MASK_READ: usize = OperandAction::READ as usize
    | OperandAction::READWRITE as usize
    | OperandAction::CONDREAD as usize
    | OperandAction::READ_CONDWRITE as usize
    | OperandAction::CONDREAD_WRITE as usize;

pub const OPERAND_ACTION_MAX_VALUE: OperandAction = OperandAction::CONDREAD_WRITE;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum InstructionEncoding {
    DEFAULT,
    _3DNOW,
    XOP,
    VEC,
    EVEX,
    MVEX,
}

pub const INSTRUCTION_ENCODING_MAX_VALUE: InstructionEncoding = InstructionEncoding::MVEX;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum OpcodeMap {
    DEFAULT,
    _0F,
    _0F38,
    _0F3A,
    _0F0F,
    XOP8,
    XOP9,
    XOPA,
}

pub const OPCODE_MAP_MAX_VALUE: OpcodeMap = OpcodeMap::XOPA;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum MemoryOperandType {
    INVALID,
    MEM,
    AGEN,
    MIB,
}

pub const MEMORY_OPERAND_TYPE_MAX_VALUE: MemoryOperandType = MemoryOperandType::MIB;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum CPUFlag {
    CF,
    PF,
    AF,
    ZF,
    SF,
    TF,
    IF,
    DF,
    OF,
    IOPL,
    NT,
    RF,
    VM,
    AC,
    VIF,
    VIP,
    ID,
    C0,
    C1,
    C2,
    C3,
}

pub const CPU_FLAG_MAX_VALUE: CPUFlag = CPUFlag::C3;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum CPUFlagAction {
    NONE,
    TESTED,
    TESTED_MODIFIED,
    MODIFIED,
    SET_0,
    SET_1,
    UNDEFINED,
}

pub const CPU_FLAG_ACTION_MAX_VALUE: CPUFlagAction = CPUFlagAction::UNDEFINED;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum ExceptionClass {
    NONE,
    SSE1,
    SSE2,
    SSE3,
    SSE4,
    SSE5,
    SSE7,
    AVX1,
    AVX2,
    AVX3,
    AVX4,
    AVX5,
    AVX6,
    AVX7,
    AVX8,
    AVX11,
    AVX12,
    E1,
    E1NF,
    E2,
    E2NF,
    E3,
    E3NF,
    E4,
    E4NF,
    E5,
    E5NF,
    E6,
    E6NF,
    E7NM,
    E7NM128,
    E9NF,
    E10,
    E10NF,
    E11,
    E11NF,
    E12,
    E12NP,
    K20,
    K21,
}

pub const EXCEPTION_CLASS_MAX_VALUE: ExceptionClass = ExceptionClass::K21;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum MaskMode {
    Invalid,
    Disabled,
    Merging,
    Zeroing,
    Control,
    ControlZeroing,
}

pub const MASK_MODE_MAX_VALUE: MaskMode = MaskMode::ControlZeroing;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum BroadcastMode {
    INVALID,
    _1_TO_2,
    _1_TO_4,
    _1_TO_8,
    _1_TO_16,
    _1_TO_32,
    _1_TO_64,
    _2_TO_4,
    _2_TO_8,
    _2_TO_16,
    _4_TO_8,
    _4_TO_16,
    _8_TO_16,
}

pub const BROADCAST_MODE_MAX_VALUE: BroadcastMode = BroadcastMode::_8_TO_16;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum RoundingMode {
    INVALID,
    RN,
    RD,
    RU,
    RZ,
}

pub const ROUNDING_MODE_MAX_VALUE: RoundingMode = RoundingMode::RZ;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum SwizzleMode {
    INVALID,
    DCBA,
    CDAB,
    BADC,
    DACB,
    AAAA,
    BBBB,
    CCCC,
    DDDD,
}

pub const SWIZZLE_MODE_MAX_VALUE: SwizzleMode = SwizzleMode::DDDD;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum ConversionMode {
    INVALID,
    FLOAT16,
    SINT8,
    UINT8,
    SINT16,
    UINT16,
}

pub const CONVERISON_MODE_MAX_VALUE: ConversionMode = ConversionMode::UINT16;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum PrefixType {
    IGNORED,
    EFFECTIVE,
    MANDATORY,
}

pub const PREFIX_TYPE_MAX_VALUE: PrefixType = PrefixType::MANDATORY;

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

#[cfg_attr(feature = "serialization", derive(Deserilaize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum BranchType {
    NONE,
    SHORT,
    NEAR,
    FAR,
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
        const IS_PRIVILIGED             = 1 << 8;
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
