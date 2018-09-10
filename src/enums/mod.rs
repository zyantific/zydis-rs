pub mod instructioncategory;
pub mod isaext;
pub mod isaset;
pub mod mnemonic;
pub mod register;

pub use instructioncategory::*;
pub use isaext::*;
pub use isaset::*;
pub use mnemonic::*;
pub use register::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum Feature {
    AVX512,
    KNC,
}

pub const FEATURE_MAX_VALUE: Feature = Feature::KNC;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum FormatterStyle {
    INTEL,
    INTEL_MASM,
}

pub const FORMATTER_STYLE_MAX_VALUE: FormatterStyle = FormatterStyle::INTEL_MASM;

/// We wrap this in a nicer rust enum already, use that instead.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum ZydisFormatterProperty {
    UPPERCASE,
    FORCE_MEMSEG,
    FORCE_MEMSIZE,
    ADDR_FORMAT,
    DISP_FORMAT,
    IMM_FORMAT,
    HEX_UPPERCASE,
    HEX_PREFIX,
    HEX_SUFFIX,
    HEX_PADDING_ADDR,
    HEX_PADDING_DISP,
    HEX_PADDING_IMM,
}

pub const FORMATTER_PROPERTY_MAX_VALUE: ZydisFormatterProperty =
    ZydisFormatterProperty::HEX_PADDING_IMM;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum AddressFormat {
    ABSOLUTE,
    RELATIVE_UNSIGNED,
    RELATIVE_SIGNED,
    RELATIVE_ASSEMBLER,
}

pub const ADDRESS_FORMAT_MAX_VALUE: AddressFormat = AddressFormat::RELATIVE_ASSEMBLER;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum DisplacementFormat {
    HEX_SIGNED,
    HEX_UNSIGNED,
}

pub const DISPLACEMENT_FORMAT_MAX_VALUE: DisplacementFormat = DisplacementFormat::HEX_UNSIGNED;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum ImmediateFormat {
    HEX_AUTO,
    HEX_SIGNED,
    HEX_UNSIGNED,
}

pub const IMMEDIATE_FORMAT_MAX_VALUE: ImmediateFormat = ImmediateFormat::HEX_UNSIGNED;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum HookType {
    PRE_INSTRUCTION,
    POST_INSTRUCTION,
    PRE_OPERAND,
    POST_OPERAND,
    FORMAT_INSTRUCTION,
    FORMAT_OPERAND_REG,
    FORMAT_OPERAND_MEM,
    FORMAT_OPERAND_PTR,
    FORMAT_OPERAND_IMM,
    PRINT_MNEMONIC,
    PRINT_REGISTER,
    PRINT_ADDRESS,
    PRINT_DISP,
    PRINT_IMM,
    PRINT_MEMSIZE,
    PRINT_PREFIXES,
    PRINT_DECORATOR,
}

pub const HOOK_TYPE_MAX_VALUE: HookType = HookType::PRINT_DECORATOR;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum DecoratorType {
    INVALID,
    MASK,
    BC,
    RC,
    SAE,
    SWIZZLE,
    CONVERSION,
    EH,
}

pub const DECORATOR_TYPE_MAX_VALUE: DecoratorType = DecoratorType::EH;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum MachineMode {
    LONG_64,
    LONG_COMPAT_32,
    LONG_COMPAT_16,
    LEGACY_32,
    LEGACY_16,
    REAL_16,
}

pub const MACHINE_MODE_MAX_VALUE: MachineMode = MachineMode::REAL_16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum AddressWidth {
    _16,
    _32,
    _64,
}

pub const ADDRESS_WIDTH_MAX_VALUE: AddressWidth = AddressWidth::_64;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum OperandType {
    UNUSED,
    REGISTER,
    MEMORY,
    POINTER,
    IMMEDIATE,
}

pub const OPERAND_TYPE_MAX_VALUE: OperandType = OperandType::IMMEDIATE;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum OperandVisibility {
    INVALID,
    EXPLICIT,
    IMPLICIT,
    HIDDEN,
}

pub const OPERAND_VISIBILITY_MAX_VALUE: OperandVisibility = OperandVisibility::HIDDEN;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum MemoryOperandType {
    INVALID,
    MEM,
    AGEN,
    MIB,
}

pub const MEMORY_OPERAND_TYPE_MAX_VALUE: MemoryOperandType = MemoryOperandType::MIB;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum MaskMode {
    INVALID,
    DISABLED,
    MERGING,
    ZEROING,
    CONTROL,
    CONTROL_ZEROING,
}

pub const MASK_MODE_MAX_VALUE: MaskMode = MaskMode::CONTROL_ZEROING;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum PrefixType {
    IGNORED,
    EFFECTIVE,
    MANDATORY,
}

pub const PREFIX_TYPE_MAX_VALUE: PrefixType = PrefixType::MANDATORY;

bitflags! {
    #[repr(transparent)]
    struct InstructionAttributes: u64 {
        const HAS_MODRM                 = 1 << 0;
        const HAS_SIB                   = 1 << 1;
        const HAS_REX                   = 1 << 2;
        const HAS_XOP                   = 1 << 3;
        const HAS_VEX                   = 1 << 4;
        const HAS_EVEX                  = 1 << 5;
        const HAS_MVEX                  = 1 << 6;
        const IS_RELATIVE               = 1 << 7;
        const IS_PRIVILIGED             = 1 << 8;
        const IS_FAR_BRANCH             = 1 << 36;
        const CPUFLAG_ACCESS            = 1 << 37;
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
        const HAS_SEGMENT               = HAS_SEGMENT_CS
            | HAS_SEGMENT_SS
            | HAS_SEGMENT_DS
            | HAS_SEGMENT_ES
            | HAS_SEGMENT_FS
            | HAS_SEGMENT_GS;
        const HAS_OPERANDSIZE           = 1 << 34;
        const HAS_ADDRESSIZE            = 1 << 35;
    }
}
