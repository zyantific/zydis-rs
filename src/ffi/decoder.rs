use super::*;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Decoder {
    machine_mode: MachineMode,
    stack_width: StackWidth,
    decoder_mode: [bool; DECODER_MODE_MAX_VALUE + 1],
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub enum DecodedOperandKind {
    Unused,
    Reg(Register),
    Mem(MemoryInfo),
    Ptr(PointerInfo),
    Imm(ImmediateInfo),
}

/// A decoded operand.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct DecodedOperand {
    /// The operand id.
    pub id: u8,
    /// The visibility of the operand.
    pub visibility: OperandVisibility,
    /// The operand action.
    pub action: OperandAction,
    /// The operand encoding.
    pub encoding: OperandEncoding,
    /// The logical size of the operand, in bits.
    pub size: u16,
    /// The element type.
    pub element_type: ElementType,
    /// The size of a single element.
    pub element_size: u16,
    /// The number of elements.
    pub element_count: u16,
    /// Additional operand attributes.
    pub attributes: OperandAttributes,
    /// Operand information specific to the kind of the operand.
    pub kind: DecodedOperandKind,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct MemoryInfo {
    pub ty: MemoryOperandType,
    pub segment: Register,
    pub base: Register,
    pub index: Register,
    pub scale: u8,
    pub disp: DisplacementInfo,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct DisplacementInfo {
    /// Signals if displacement is present.
    pub has_displacement: bool,
    /// The displacement value
    pub displacement: i64,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct PointerInfo {
    pub segment: u16,
    pub offset: u32,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct ImmediateInfo {
    /// Signals, if the immediate is signed.
    pub is_signed: bool,
    /// Signals, if the immediate is relative.
    pub is_relative: bool,
    /// This is actually an i64 if `is_signed` is true.
    // TODO: Should we use an union?
    // C definition:
    //   union ZydisDecodedOperandImmValue_{ ZyanU64 u; ZyanI64 s; } value;
    pub value: u64,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct AccessedFlags<FlagType> {
    /// Flags that may be read by the instruction.
    pub tested: FlagType,
    /// Flags that may be modified by the instruction.
    pub modified: FlagType,
    /// Flags that the instruction sets to 0.
    pub set_0: FlagType,
    /// Flags that the instruction sets to 1.
    pub set_1: FlagType,
    /// Flags where access behavior is undefined / CPU model specific.
    pub undefined: FlagType,
}

// NOTE: can't implement `deserialize` due to the static refs (no easy way to
// recover)
#[cfg_attr(feature = "serialization", derive(Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct DecodedInstruction {
    /// The machine mode used to decode this instruction.
    pub machine_mode: MachineMode,
    /// The instruction-mnemonic.
    pub mnemonic: Mnemonic,
    /// The length of the decoded instruction.
    pub length: u8,
    /// The instruction-encoding.
    pub encoding: InstructionEncoding,
    /// The opcode map.
    pub opcode_map: OpcodeMap,
    /// The instruction opcode.
    pub opcode: u8,
    /// The stack width.
    pub stack_width: u8,
    /// The effective operand width.
    pub operand_width: u8,
    /// The effective address width.
    pub address_width: u8,
    /// The number of instruction operands.
    pub operand_count: u8,
    /// The number of explicit (visible) instruction operands.
    pub operand_count_visible: u8,
    /// Instruction attributes.
    pub attributes: InstructionAttributes,
    /// Information about CPU flags accessed by the instruction.
    ///
    /// The bits in the masks correspond to the actual bits in the
    /// `FLAGS/EFLAGS/RFLAGS` register.
    // https://github.com/zyantific/zydis/issues/319
    pub cpu_flags: Option<&'static AccessedFlags<CpuFlag>>,
    /// Information about FPU flags accessed by the instruction.
    pub fpu_flags: Option<&'static AccessedFlags<FpuFlag>>,
    /// Extended information for `AVX` instructions.
    pub avx: AvxInfo,
    /// Meta info.
    pub meta: MetaInfo,
    /// Detailed information about different instruction-parts.
    pub raw: RawInfo,
}

impl DecodedInstruction {
    /// Calculates the absolute address for the given instruction operand,
    /// using the given `address` as the address for this instruction.
    #[inline]
    pub fn calc_absolute_address(&self, address: u64, operand: &DecodedOperand) -> Result<u64> {
        unsafe {
            let mut addr = 0u64;
            ZydisCalcAbsoluteAddress(self, operand, address, &mut addr).as_result()?;
            Ok(addr)
        }
    }

    /// Behaves like `calc_absolute_address`, but takes runtime-known values of
    /// registers passed in the `context` into account.
    #[inline]
    pub fn calc_absolute_address_ex(
        &self,
        address: u64,
        operand: &DecodedOperand,
        context: &RegisterContext,
    ) -> Result<u64> {
        unsafe {
            let mut addr = 0u64;
            ZydisCalcAbsoluteAddressEx(self, operand, address, context, &mut addr).as_result()?;
            Ok(addr)
        }
    }
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct AvxInfo {
    /// The `AVX` vector-length.
    pub vector_length: u16,
    /// The masking mode.
    pub mask_mode: MaskMode,
    /// The mask register.
    pub mask_reg: Register,
    /// Signals if the broadcast is a static broadcast.
    ///
    /// This is the case for instructions with inbuild broadcast functionality,
    /// which is always active.
    pub broadcast_static: bool,
    /// The `AVX` broadcast-mode.
    pub broadcast_mode: BroadcastMode,
    /// The `AVX` rounding-mode.
    pub rounding_mode: RoundingMode,
    /// The `AVX` register-swizzle mode.
    pub swizzle_mode: SwizzleMode,
    /// The `AVX` data-conversion mode.
    pub conversion_mode: ConversionMode,
    /// Signals if the "SAE" (supress-all-exceptions) functionality is enabled
    /// for the instruction.
    pub has_sae: bool,
    /// Signals, if the instruction has a memory-eviction-hint (`KNC` only).
    pub has_eviction_hint: bool,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct MetaInfo {
    /// The category this instruction belongs to.
    pub category: InstructionCategory,
    /// The instruction set this instruction belongs to.
    pub isa_set: ISASet,
    /// The instruction set extension this instruction belongs to.
    pub isa_ext: ISAExt,
    /// The branch type.
    pub branch_type: BranchType,
    /// The exception class of this instruction.
    pub exception_class: ExceptionClass,
}

/// Detailed info about the `REX` prefix.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct RawInfoRex {
    /// 64-bit operand-size promotion.
    pub W: u8,
    /// Extension of the `ModRM.reg` field.
    pub R: u8,
    /// Extension of the `SIB.index` field.
    pub X: u8,
    /// Extension of the `ModRM.rm`, `SIB.base` or `opcode.reg` field.
    pub B: u8,
    /// The offset to the effective `REX` byte, relative to the beginning of
    /// the instruction, in bytes.
    ///
    /// This offset always points to the "effective" `REX` prefix (the one
    /// closest to the instruction opcode), if multiple `REX` prefixes are
    /// present.
    ///
    /// This can be `0`, if the `REX` byte is the first byte of the instruction.
    ///
    /// Refer to the instruction attributes to check for the presence of the
    /// `REX` prefix.
    pub rex_offset: u8,
}

/// Detailed info about the `XOP` prefix.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct RawInfoXop {
    /// Extension of the `ModRM.reg` field (inverted).
    pub aR: u8,
    /// Extension of the `SIB.index` field (inverted).
    pub X: u8,
    /// Extension of the `ModRM.rm`, `SIB.base` or `opcode.reg` (inverted).
    pub B: u8,
    /// Opcode-map specifier.
    pub m_mmmm: u8,
    /// 64-bit operand-size promotion or opcode-extension.
    pub W: u8,
    /// `NDS`/`NDD` (non-destructive-source/destination) register specifier
    /// (inverted).
    pub vvvv: u8,
    /// Vector-length specifier.
    pub L: u8,
    /// Compressed legacy prefix.
    pub pp: u8,
    /// The offset of the first xop byte, relative to the beginning of the
    /// instruction, in bytes.
    pub offset: u8,
}

/// Detailed info about the `VEX` prefix.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct RawInfoVex {
    /// Extension of the `modRM.reg` field (inverted).
    pub R: u8,
    /// Extension of the `SIB.index` field (inverted).
    pub X: u8,
    /// Extension of the `ModRM.rm`, `SIB.base` or `opcode.reg` field
    /// (inverted).
    pub B: u8,
    /// Opcode-map specifier.
    pub mmmm: u8,
    /// 64-bit operand-size promotion or opcode-extension.
    pub W: u8,
    /// `NDS`/`NDD` (non-destructive-source/destination) register specifier
    /// (inverted).
    pub vvvv: u8,
    /// Vector-length specifier.
    pub L: u8,
    /// Compressed legacy prefix.
    pub pp: u8,
    /// The offset of the first `VEX` byte, relative to the beginning of the
    /// instruction, in bytes.
    pub offset: u8,
    /// The size of the `VEX` prefix, in bytes.
    pub size: u8,
}

/// Detailed info about the `EVEX` prefix.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct RawInfoEvex {
    /// Extension of the `ModRM.reg` field (inverted).
    pub R: u8,
    /// Extension of the `SIB.index/vidx` field (inverted).
    pub X: u8,
    /// Extension of the `ModRM.rm` or `SIB.base` field (inverted).
    pub B: u8,
    /// High-16 register specifier modifier (inverted).
    pub R2: u8,
    /// Opcode-map specifier.
    pub mm: u8,
    /// 64-bit operand-size promotion or opcode-extension.
    pub W: u8,
    /// `NDS`/`NDD` (non-destructive-source/destination) register specifier
    /// (inverted).
    pub vvvv: u8,
    /// Compressed legacy prefix.
    pub pp: u8,
    /// Zeroing/Merging.
    pub z: u8,
    /// Vector-length specifier or rounding-control (most significant bit).
    pub L2: u8,
    /// Vector-length specifier or rounding-control (least significant bit).
    pub L: u8,
    /// Broadcast/RC/SAE context.
    pub b: u8,
    /// High-16 `NDS`/`VIDX` register specifier.
    pub V2: u8,
    /// Embedded opmask register specifier.
    pub aaa: u8,
    /// The offset of the first evex byte, relative to the beginning of the
    /// instruction, in bytes.
    pub offset: u8,
}

/// Detailed info about the `MVEX` prefix.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct RawInfoMvex {
    /// Extension of the `ModRM.reg` field (inverted).
    pub R: u8,
    /// Extension of the `SIB.index/vidx` field (inverted).
    pub X: u8,
    /// Extension of the `ModRM.rm` or `SIB.base` field (inverted).
    pub B: u8,
    /// High-16 register specifier modifier (inverted).
    pub R2: u8,
    /// Opcode-map specifier.
    pub mmmm: u8,
    /// 64-bit operand-size promotion or opcode-extension.
    pub W: u8,
    /// `NDS`/`NDD` (non-destructive-source/destination) register specifier
    /// (inverted).
    pub vvvv: u8,
    /// Compressed legacy prefix.
    pub pp: u8,
    /// Non-temporal/eviction hint.
    pub E: u8,
    /// Swizzle/broadcast/up-convert/down-convert/static-rounding controls.
    pub SSS: u8,
    /// High-16 `NDS`/`VIDX` register specifier.
    pub V2: u8,
    /// Embedded opmask register specifier.
    pub kkk: u8,
    /// The offset of the first mvex byte, relative to the beginning of the
    /// instruction, in bytes.
    pub offset: u8,
}

/// Detailed info about the `ModRM` byte.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct RawInfoModRm {
    /// The addressing mode.
    pub mod_: u8,
    /// Register specifier or opcode-extension.
    pub reg: u8,
    /// Register specifier or opcode-extension.
    pub rm: u8,
    /// The offset of the `ModRM` byte, relative to the beginning of the
    /// instruction, in bytes.
    pub offset: u8,
}

/// Detailed info about the `SIB` byte.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct RawInfoSib {
    /// The scale factor.
    pub scale: u8,
    /// The index-register specifier.
    pub index: u8,
    /// The base-register specifier.
    pub base: u8,
    /// THe offset of the `SIB` byte, relative to the beginning of the
    /// instruction, in bytes.
    pub offset: u8,
}

/// Detailed info about displacement-bytes.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct RawInfoDisp {
    /// The displacement value.
    pub value: i64,
    /// The physical displacement size, in bits.
    pub size: u8,
    /// The offset of the displacement data, relative to the beginning of the
    /// instruction, in bytes.
    pub offset: u8,
}

/// Union for raw info from various mutually exclusive vector extensions.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
#[allow(non_snake_case)]
pub enum RawInfoKindSpecific {
    // Note: this must match the order in `ZydisInstructionEncoding`.
    Legacy(RawInfoRex),
    _3DNOW,
    Xop(RawInfoXop),
    Vex(RawInfoVex),
    Evex(RawInfoEvex),
    Mvex(RawInfoMvex),
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct RawInfo {
    /// The number of legacy prefixes.
    pub prefix_count: u8,
    /// Detailed info about the legacy prefixes (including `REX`).
    pub prefixes: [Prefix; MAX_INSTRUCTION_LENGTH],
    /// Raw info depending on the instruction kind.
    ///
    /// Note: this is an anonymous union in the C library.
    pub kind_specific: RawInfoKindSpecific,
    /// Detailed info about the `ModRM` byte.
    pub modrm: RawInfoModRm,
    /// Detailed info about the `SIB` byte.
    pub sib: RawInfoSib,
    /// Detailed info about displacement-bytes.
    pub disp: RawInfoDisp,
    /// Detailed information about immediate-bytes.
    pub imm: [RawImmediateInfo; 2],
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct RawImmediateInfo {
    /// Signals, if the immediate value is signed.
    pub is_signed: bool,
    /// Signals, if the immediate value contains a relative offset. You can use
    /// `calc_absolute_address` to determine the absolute address value.
    pub is_relative: bool,
    /// The immediate value.
    ///
    /// This is an `i64` if `is_signed` is true.
    // TODO: union?
    // C definition:
    //   union ZydisDecodedInstructionRawImmValue_ { ZyanU64 u; ZyanI64 s; } value;
    pub value: u64,
    /// The physical immediate size, in bits.
    pub size: u8,
    /// The offset of the immediate data, relative to the beginning of the
    /// instruction, in bytes.
    pub offset: u8,
}

/// Detailed info about the legacy prefixes (including `REX`).
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct Prefix {
    /// The type of this prefix.
    pub ty: PrefixType,
    /// The value of this prefix.
    pub value: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct ContextVectorUnified {
    pub W: u8,
    pub R: u8,
    pub X: u8,
    pub B: u8,
    pub L: u8,
    pub LL: u8,
    pub R2: u8,
    pub V2: u8,
    pub vvvv: u8,
    pub mask: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct ContextRegInfo {
    /// Signals if the `modrm.mod == 3` or `reg` form is forced for the
    /// instruction.
    pub is_mod_reg: bool,
    /// The final register id for the `reg` encoded register.
    pub id_reg: u8,
    /// The final register id for the `rm` encoded register.
    /// This value is only set, if a register is encoded in `modrm.rm`.
    pub id_rm: u8,
    /// The final register id for the `ndsndd` (`.vvvv`) encoded register.
    pub id_ndsndd: u8,
    /// The final register id for the base register.
    /// This value is only set, if a memory operand is encoded in `modrm.rm`.
    pub id_base: u8,
    /// The final register id for the index register.
    /// This value is only set, if a memory operand is encoded in `modrm.rm` and
    /// the `SIB` byte is present.
    pub id_index: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct ContextEvex {
    /// The EVEX tuple-type.
    ty: u8,
    /// The EVEX element-size.
    element_size: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct ContextMvex {
    /// The MVEX functionality.
    functionality: u8,
}

/// Opaque internal instruction definition.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct ContextDefinition(u8 /* dummy */);

/// Internal decoder context kept between instruction and operand decoding.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct DecoderContext {
    /// A pointer to the internal instruction definition.
    definition: *const ContextDefinition,
    /// Contains the effective operand-size index.
    /// 0 = 16 bit, 1 = 32 bit, 2 = 64 bit
    eosz_index: u8,
    /// Contains the effective address-size index.
    /// 0 = 16 bit, 1 = 32 bit, 2 = 64 bit
    easz_index: u8,
    /// Contains some cached REX/XOP/VEX/EVEX/MVEX values to provide uniform
    /// access.
    vector_unified: ContextVectorUnified,
    /// Information about encoded operand registers.
    reg_info: ContextRegInfo,
    /// Internal EVEX-specific information.
    evex: ContextEvex,
    /// Internal MVEX-specific information.
    mvex: ContextMvex,
    /// The scale factor for EVEX/MVEX compressed 8-bit displacement values.
    cd8_scal: u8,
}

extern "C" {
    pub fn ZydisDecoderInit(
        decoder: *mut Decoder,
        machine_mode: MachineMode,
        stack_width: StackWidth,
    ) -> Status;

    pub fn ZydisDecoderEnableMode(
        decoder: *mut Decoder,
        mode: DecoderMode,
        enabled: bool,
    ) -> Status;

    pub fn ZydisDecoderDecodeFull(
        decoder: *const Decoder,
        buffer: *const c_void,
        length: usize,
        instruction: *mut DecodedInstruction,
        operands: *mut [DecodedOperand; MAX_OPERAND_COUNT],
    ) -> Status;

    pub fn ZydisDecoderDecodeInstruction(
        decoder: *const Decoder,
        context: *mut DecoderContext,
        buffer: *const c_void,
        length: usize,
        instruction: *mut DecodedInstruction,
    ) -> Status;

    pub fn ZydisDecoderDecodeOperands(
        decoder: *const Decoder,
        context: *const DecoderContext,
        instruction: *const DecodedInstruction,
        operands: *mut DecodedOperand,
        operand_count: u8,
    ) -> Status;
}
