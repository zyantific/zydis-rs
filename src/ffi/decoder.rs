use super::*;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Decoder {
    machine_mode: MachineMode,
    stack_width: StackWidth,
    decoder_mode: [bool; DECODER_MODE_MAX_VALUE + 1],
}

impl Decoder {
    /// Creates a new `Decoder` with the given `machine_mode` and
    /// `stack_width`.
    #[inline]
    pub fn new(machine_mode: MachineMode, stack_width: StackWidth) -> Result<Self> {
        unsafe {
            let mut decoder = MaybeUninit::uninit();
            check!(
                ZydisDecoderInit(decoder.as_mut_ptr(), machine_mode, stack_width),
                decoder.assume_init()
            )
        }
    }

    /// Enables or disables (depending on the `value`) the given decoder `mode`:
    #[inline]
    pub fn enable_mode(&mut self, mode: DecoderMode, value: bool) -> Result<()> {
        unsafe { check!(ZydisDecoderEnableMode(self, mode, value as _)) }
    }

    /// Decodes a single binary instruction to a `DecodedInstruction`.
    ///
    /// # Examples
    /// ```
    /// use zydis::{Decoder, MachineMode, Mnemonic, StackWidth};
    /// static INT3: &'static [u8] = &[0xCC];
    /// let decoder = Decoder::new(MachineMode::LONG_64, StackWidth::_64).unwrap();
    ///
    /// let instruction = decoder.decode(INT3).unwrap().unwrap();
    /// assert_eq!(instruction.mnemonic, Mnemonic::INT3);
    /// ```
    #[inline]
    pub fn decode(
        &self,
        buffer: &[u8],
        operands: &mut [DecodedOperand],
    ) -> Result<Option<DecodedInstruction>> {
        // TODO(ath): is there a more elegant way to construct this MaybeUninit slice?
        self.decode_uninit_operands(buffer, unsafe { std::mem::transmute(operands) })
    }

    /// Decodes a single binary instruction to a `DecodedInstruction` without
    /// initializing the operands first.
    #[inline]
    pub fn decode_uninit_operands(
        &self,
        buffer: &[u8],
        operands: &mut [MaybeUninit<DecodedOperand>],
    ) -> Result<Option<DecodedInstruction>> {
        if operands.len() > usize::from(u8::MAX) {
            return Err(Status::InvalidArgument);
        }

        unsafe {
            let mut instruction = MaybeUninit::uninit();
            check_option!(
                ZydisDecoderDecodeFull(
                    self,
                    buffer.as_ptr() as *const c_void,
                    buffer.len(),
                    instruction.as_mut_ptr(),
                    operands.as_mut_ptr() as _,
                    operands.len() as u8,
                    DecodingFlags::empty(), // TODO(ath): expose to user
                ),
                instruction.assume_init()
            )
        }
    }

    /// Returns an iterator over all the instructions in the buffer.
    ///
    /// The iterator ignores all errors and stops producing values in the case
    /// of an error.
    #[inline]
    pub fn instruction_iterator<'a, 'b>(
        &'a self,
        buffer: &'b [u8],
        ip: u64,
    ) -> InstructionIterator<'a, 'b> {
        InstructionIterator {
            decoder: self,
            buffer,
            ip,
        }
    }
}

pub struct InstructionIterator<'a, 'b> {
    decoder: &'a Decoder,
    buffer: &'b [u8],
    ip: u64,
}

impl Iterator for InstructionIterator<'_, '_> {
    // TODO(ath): should we switch this to yield a struct instead?
    type Item = (DecodedInstruction, [DecodedOperand; MAX_OPERAND_COUNT], u64);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut operands = unsafe {
            // Shamelessly pasted from the unstable impl of `uninit_array`:
            // https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html#method.uninit_array
            MaybeUninit::<[MaybeUninit<DecodedOperand>; MAX_OPERAND_COUNT]>::uninit().assume_init()
        };

        let insn = self
            .decoder
            .decode_uninit_operands(self.buffer, &mut operands)
            .ok()
            .flatten()?;

        self.buffer = &self.buffer[insn.length as usize..];

        let ip = self.ip;
        self.ip += u64::from(insn.length);

        // SAFETY: Zydis zeroes all operands passed, regardless of whether
        //         they are used or not.
        let operands: [DecodedOperand; MAX_OPERAND_COUNT] = unsafe {
            // `array_assume_init` is still unstable :(
            std::mem::transmute(operands)
        };

        Some((insn, operands, ip))
    }
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub enum DecodedOperandKind {
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
    pub value: u64,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct AccessedFlags<FlagType> {
    tested: FlagType,
    modified: FlagType,
    set_0: FlagType,
    set_1: FlagType,
    undefined: FlagType,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
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
            check!(
                ZydisCalcAbsoluteAddress(self, operand, address, &mut addr),
                addr
            )
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
            check!(
                ZydisCalcAbsoluteAddressEx(self, operand, address, context, &mut addr),
                addr
            )
        }
    }

    /// Returns offsets and sizes of all logical instruction segments.
    #[inline]
    pub fn get_segments(&self) -> Result<InstructionSegments> {
        unsafe {
            let mut segments = MaybeUninit::uninit();
            check!(
                ZydisGetInstructionSegments(self, segments.as_mut_ptr()),
                segments.assume_init()
            )
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
#[repr(C)] // For consistency with zydis and the Intel docs.
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
#[repr(C)] // For consistency with zydis and the Intel docs.
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
#[repr(C)] // For consistency with zydis and the Intel docs.
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
#[repr(C)] // For consistency with zydis and the Intel docs.
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
#[repr(C)] // For consistency with zydis and the Intel docs.
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
#[repr(C)] // For consistency with zydis and the Intel docs.
#[allow(non_snake_case)]
pub struct RawInfoModRm {
    /// The addressing mode.
    pub modrm_mod: u8,
    /// Register specifier or opcode-extension.
    pub modrm_reg: u8,
    /// Register specifier or opcode-extension.
    pub modrm_rm: u8,
    /// The offset of the `ModRM` byte, relative to the beginning of the
    /// instruction, in bytes.
    pub modrm_offset: u8,
}

/// Detailed info about the `SIB` byte.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)] // For consistency with zydis and the Intel docs.
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
#[repr(C)] // For consistency with zydis and the Intel docs.
#[allow(non_snake_case)]
pub struct RawInfoDisp {
    /// The displacement value.
    pub disp_value: i64,
    /// THe physical displacement size, in bits.
    pub disp_size: u8,
    /// The offset of the displacement data, relative to the beginning of the
    /// instruction, in bytes.
    pub disp_offset: u8,
}

/// Union for raw info from various mutually exclusive vector extensions.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)] // For consistency with zydis and the Intel docs.
#[allow(non_snake_case)]
pub enum RawInfoKindSpecific {
    Xop(RawInfoXop),
    Vex(RawInfoVex),
    Evex(RawInfoEvex),
    Mvex(RawInfoMvex),
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)] // For consistency with zydis and the Intel docs.
#[allow(non_snake_case)]
pub struct RawInfo {
    /// The number of legacy prefixes.
    pub prefix_count: u8,
    /// Detailed info about the legacy prefixes (including `REX`).
    pub prefixes: [Prefix; MAX_INSTRUCTION_LENGTH],
    /// Detailed info about the `REX` prefix.
    pub rex: RawInfoRex,
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

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
// For consistency with zydis and the Intel docs.
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

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct ContextRegInfo {
    pub is_mod_reg: bool,
    pub id_reg: u8,
    pub id_rm: u8,
    pub id_ndsndd: u8,
    pub id_base: u8,
    pub id_index: u8,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct ContextEvex {
    pub ty: u8,
    pub element_size: u8,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct ContextMvex {
    pub functionality: u8,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct DecoderContext {
    pub definition: *const c_void,
    pub eosz_index: u8,
    pub easz_index: u8,
    pub vector_unified: ContextVectorUnified,
    pub reg_info: ContextRegInfo,
    pub evex: ContextEvex,
    pub mvex: ContextMvex,
    pub cd8_scal: u8,
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
        operands: *mut DecodedOperand,
        operand_count: u8,
        flags: DecodingFlags,
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
        context: *mut DecoderContext,
        operands: *mut DecodedOperand,
        operand_count: u8,
    ) -> Status;
}
