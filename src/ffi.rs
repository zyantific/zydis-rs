//! Provides type aliases, struct definitions and the unsafe function
//! declarations.

use core::{fmt, marker::PhantomData, mem::MaybeUninit, slice};

// TODO: use libc maybe, or wait for this to move into core?
use std::{
    ffi::CStr,
    os::raw::{c_char, c_void},
};

use super::{
    enums::*,
    status::{Result, Status},
};

#[rustfmt::skip]
pub type FormatterFunc = Option<unsafe extern "C" fn(
    *const ZydisFormatter,
    *mut FormatterBuffer,
    *mut FormatterContext) -> Status>;

#[rustfmt::skip]
pub type FormatterDecoratorFunc = Option<unsafe extern "C" fn(
    *const ZydisFormatter,
    *mut FormatterBuffer,
    *mut FormatterContext,
    Decorator) -> Status>;

#[rustfmt::skip]
pub type FormatterRegisterFunc = Option<unsafe extern "C" fn(
    *const ZydisFormatter,
    *mut FormatterBuffer,
    *mut FormatterContext,
    Register) -> Status>;

pub type RegisterWidth = u16;

#[derive(Debug)]
#[repr(C, packed)]
pub struct FormatterToken<'a> {
    ty: Token,
    next: u8,

    _p: PhantomData<&'a ()>,
}

impl<'a> FormatterToken<'a> {
    /// Returns the value and type of this token.
    #[inline]
    pub fn get_value(&self) -> Result<(Token, &'a str)> {
        unsafe {
            let mut ty = MaybeUninit::uninit();
            let mut val = MaybeUninit::uninit();
            check!(ZydisFormatterTokenGetValue(
                self,
                ty.as_mut_ptr(),
                val.as_mut_ptr()
            ))?;

            let val = CStr::from_ptr(val.assume_init() as *const _)
                .to_str()
                .map_err(|_| Status::NotUTF8)?;

            Ok((ty.assume_init(), val))
        }
    }

    /// Returns the next token.
    #[inline]
    pub fn next(&self) -> Result<&'a Self> {
        unsafe {
            let mut res = self as *const _;
            check!(ZydisFormatterTokenNext(&mut res))?;

            if res.is_null() {
                Err(Status::User)
            } else {
                Ok(&*res)
            }
        }
    }
}

impl<'a> IntoIterator for &'a FormatterToken<'a> {
    type IntoIter = FormatterTokenIterator<'a>;
    type Item = (Token, &'a str);

    fn into_iter(self) -> Self::IntoIter {
        FormatterTokenIterator { next: Some(self) }
    }
}

pub struct FormatterTokenIterator<'a> {
    next: Option<&'a FormatterToken<'a>>,
}

impl<'a> Iterator for FormatterTokenIterator<'a> {
    type Item = (Token, &'a str);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.next;
        self.next = self.next.and_then(|x| x.next().ok());
        res.and_then(|x| x.get_value().ok())
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FormatterBuffer {
    is_token_list: bool,
    capacity: usize,
    string: ZyanString,
}

impl FormatterBuffer {
    /// Returns the `ZyanString` associated with this buffer.
    ///
    /// The returned string always refers to the literal value of the most
    /// recently added token and remains valid after calling `append` or
    /// `restore`.
    #[inline]
    pub fn get_string(&mut self) -> Result<&mut ZyanString> {
        unsafe {
            let mut str = MaybeUninit::uninit();
            check!(ZydisFormatterBufferGetString(self, str.as_mut_ptr()))?;

            let str = str.assume_init();
            if str.is_null() {
                Err(Status::User)
            } else {
                Ok(&mut *str)
            }
        }
    }

    /// Returns the most recently added `FormatterToken`.
    #[inline]
    pub fn get_token(&self) -> Result<&FormatterToken<'_>> {
        unsafe {
            let mut res = MaybeUninit::uninit();
            check!(
                ZydisFormatterBufferGetToken(self, res.as_mut_ptr()),
                &*res.assume_init()
            )
        }
    }

    /// Appends a new token to this buffer.
    #[inline]
    pub fn append(&mut self, token: Token) -> Result<()> {
        unsafe { check!(ZydisFormatterBufferAppend(self, token)) }
    }

    /// Returns a snapshot of the buffer-state.
    #[inline]
    pub fn remember(&self) -> Result<FormatterBufferState> {
        unsafe {
            let mut res = MaybeUninit::uninit();
            check!(
                ZydisFormatterBufferRemember(self, res.as_mut_ptr()),
                res.assume_init()
            )
        }
    }

    /// Restores a previously saved buffer-state.
    #[inline]
    pub fn restore(&mut self, state: FormatterBufferState) -> Result<()> {
        unsafe { check!(ZydisFormatterBufferRestore(self, state)) }
    }
}

/// Opaque type representing a `FormatterBuffer` state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct FormatterBufferState(usize);

/// The string type used in zydis.
#[derive(Debug)]
#[repr(C)]
pub struct ZyanString {
    flags: u8,
    vector: ZyanVector,
}

impl ZyanString {
    /// Create a new `ZyanString`, using the given `buffer` for storage.
    #[inline]
    pub fn new(buffer: &mut [u8]) -> Result<Self> {
        Self::new_ptr(buffer.as_mut_ptr(), buffer.len())
    }

    /// Create a new `ZyanString` from a given buffer and a capacity.
    #[inline]
    pub fn new_ptr(buffer: *mut u8, capacity: usize) -> Result<Self> {
        unsafe {
            let mut string = MaybeUninit::uninit();
            check!(ZyanStringInitCustomBuffer(
                string.as_mut_ptr(),
                buffer as *mut _,
                capacity
            ))?;
            Ok(string.assume_init())
        }
    }

    /// Appends the given string `s` to this buffer.
    ///
    /// Warning: The actual Rust `&str`ings are encoded in UTF-8 and aren't
    /// converted to any other encoding. They're simply copied, byte by
    /// byte, to the buffer. Therefore, the buffer should be interpreted as
    /// UTF-8 when later being printed.
    #[inline]
    pub fn append<S: AsRef<str> + ?Sized>(&mut self, s: &S) -> Result<()> {
        unsafe {
            let bytes = s.as_ref().as_bytes();
            let view = ZyanStringView::new(bytes)?;
            check!(ZyanStringAppend(self, &view))
        }
    }
}

impl fmt::Write for ZyanString {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.append(s).map_err(|_| fmt::Error)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ZyanStringView {
    string: ZyanString,
}

impl ZyanStringView {
    /// Creates a string view from the given `buffer`.
    #[inline]
    pub fn new(buffer: &[u8]) -> Result<Self> {
        unsafe {
            let mut view = MaybeUninit::uninit();
            check!(ZyanStringViewInsideBufferEx(
                view.as_mut_ptr(),
                buffer.as_ptr() as *const _,
                buffer.len()
            ))?;
            Ok(view.assume_init())
        }
    }
}

#[derive(Debug)]
#[repr(C)]
struct ZyanVector {
    allocator: *mut c_void,
    growth_factor: f32,
    shrink_threshold: f32,
    size: usize,
    capacity: usize,
    element_size: usize,
    destructor: *mut c_void,
    data: *mut c_void,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Decoder {
    machine_mode: MachineMode,
    address_width: AddressWidth,
    // DECODER_MODE_MAX_VALUE + 1
    decoder_mode: [bool; 9],
}

impl Decoder {
    /// Creates a new `Decoder` with the given `machine_mode` and
    /// `address_width`.
    #[inline]
    pub fn new(machine_mode: MachineMode, address_width: AddressWidth) -> Result<Self> {
        unsafe {
            let mut decoder = MaybeUninit::uninit();
            check!(
                ZydisDecoderInit(decoder.as_mut_ptr(), machine_mode, address_width),
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
    /// use zydis::{AddressWidth, Decoder, MachineMode, Mnemonic};
    /// static INT3: &'static [u8] = &[0xCC];
    /// let decoder = Decoder::new(MachineMode::LONG_64, AddressWidth::_64).unwrap();
    ///
    /// let instruction = decoder.decode(INT3).unwrap().unwrap();
    /// assert_eq!(instruction.mnemonic, Mnemonic::INT3);
    /// ```
    #[inline]
    pub fn decode(&self, buffer: &[u8]) -> Result<Option<DecodedInstruction>> {
        unsafe {
            let mut instruction = MaybeUninit::uninit();
            check_option!(
                ZydisDecoderDecodeBuffer(
                    self,
                    buffer.as_ptr() as *const c_void,
                    buffer.len(),
                    instruction.as_mut_ptr(),
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
    type Item = (DecodedInstruction, u64);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.decoder.decode(self.buffer) {
            Ok(Some(insn)) => {
                self.buffer = &self.buffer[insn.length as usize..];
                let ip = self.ip;
                self.ip += u64::from(insn.length);
                Some((insn, ip))
            }
            _ => None,
        }
    }
}

/// A decoded operand.
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct DecodedOperand {
    /// The operand id.
    pub id: u8,
    /// The type of the operand.
    pub ty: OperandType,
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
    /// The register value.
    pub reg: Register,
    /// Extended information for memory operands.
    pub mem: MemoryInfo,
    /// Extended information for pointer operands.
    pub ptr: PointerInfo,
    /// Extended information for immediate operands.
    pub imm: ImmediateInfo,
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
pub struct DecodedInstruction {
    /// The machine mode used to decode this instruction.
    pub machine_mode: MachineMode,
    /// The instruction-mnemonic.
    pub mnemonic: Mnemonic,
    /// The length of the decoded instruction.
    pub length: u8,
    /// The instruciton-encoding.
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
    /// Detailed information for all instruction operands.
    // ZYDIS_MAX_OPERAND_COUNT
    pub operands: [DecodedOperand; 10],
    /// Instruction attributes.
    pub attributes: InstructionAttributes,
    // ZYDIS_CPUFLAG_MAX_VALUE + 1
    /// The action performed to the `CPUFlag` used to index this array.
    pub accessed_flags: [CPUFlagAction; 21],
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

    /// Returns a mask of CPU-flags that match the given `action`.
    pub fn get_flags(&self, action: CPUFlagAction) -> Result<CPUFlag> {
        unsafe {
            let mut flags = MaybeUninit::uninit();
            check!(
                ZydisGetAccessedFlagsByAction(self, action, flags.as_mut_ptr()),
                flags.assume_init()
            )
        }
    }

    /// Returns a mask of CPU-flags that are read (tested) by this instruction.
    #[inline]
    pub fn get_flags_read(&self) -> Result<CPUFlag> {
        unsafe {
            let mut flags = MaybeUninit::uninit();
            check!(
                ZydisGetAccessedFlagsRead(self, flags.as_mut_ptr()),
                flags.assume_init()
            )
        }
    }

    /// Returns a mask of CPU-flags that are written (modified, undefined) by
    /// this instruction.
    #[inline]
    pub fn get_flags_written(&self) -> Result<CPUFlag> {
        unsafe {
            let mut flags = MaybeUninit::uninit();
            check!(
                ZydisGetAccessedFlagsWritten(self, flags.as_mut_ptr()),
                flags.assume_init()
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

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
// For consistency with zydis and the Intel docs.
#[allow(non_snake_case)]
pub struct RawInfo {
    /// The number of legacy prefixes.
    pub prefix_count: u8,
    // ZYDIS_MAX_INSTRUCTION_LENGTH
    pub prefixes: [Prefix; 15],
    /// 64-bit operand-size promotion.
    pub rex_W: u8,
    /// Extension of the `ModRM.reg` field.
    pub rex_R: u8,
    /// Extension of the `SIB.index` field.
    pub rex_X: u8,
    /// Extension of the `ModRM.rm`, `SIB.base` or `opcode.reg` field.
    pub rex_B: u8,
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
    /// Extension of the `ModRM.reg` field (inverted).
    pub xop_R: u8,
    /// Extension of the `SIB.index` field (inverted).
    pub xop_X: u8,
    /// Extension of the `ModRM.rm`, `SIB.base` or `opcode.reg` (inverted).
    pub xop_B: u8,
    /// Opcode-map specifier.
    pub xop_m_mmmm: u8,
    /// 64-bit operand-size promotion or opcode-extension.
    pub xop_W: u8,
    /// `NDS`/`NDD` (non-destructive-source/destination) register specifier
    /// (inverted).
    pub xop_vvvv: u8,
    /// Vector-length specifier.
    pub xop_L: u8,
    /// Compressed legacy prefix.
    pub xop_pp: u8,
    /// The offset of the first xop byte, relative to the beginning of the
    /// instruction, in bytes.
    pub xop_offset: u8,
    /// Extension of the `modRM.reg` field (inverted).
    pub vex_R: u8,
    /// Extension of the `SIB.index` field (inverted).
    pub vex_X: u8,
    /// Extension of the `ModRM.rm`, `SIB.base` or `opcode.reg` field
    /// (inverted).
    pub vex_B: u8,
    /// Opcode-map specifier.
    pub vex_mmmm: u8,
    /// 64-bit operand-size promotion or opcode-extension.
    pub vex_W: u8,
    /// `NDS`/`NDD` (non-destructive-source/destination) register specifier
    /// (inverted).
    pub vex_vvvv: u8,
    /// Vector-length specifier.
    pub vex_L: u8,
    /// Compressed legacy prefix.
    pub vex_pp: u8,
    /// The offset of the first `VEX` byte, relative to the beginning of the
    /// instruction, in bytes.
    pub vex_offset: u8,
    /// The size of the `VEX` prefix, in bytes.
    pub vex_size: u8,
    /// Extension of the `ModRM.reg` field (inverted).
    pub evex_R: u8,
    /// Extension of the `SIB.index/vidx` field (inverted).
    pub evex_X: u8,
    /// Extension of the `ModRM.rm` or `SIB.base` field (inverted).
    pub evex_B: u8,
    /// High-16 register specifier modifier (inverted).
    pub evex_R2: u8,
    /// Opcode-map specifier.
    pub evex_mm: u8,
    /// 64-bit operand-size promotion or opcode-extension.
    pub evex_W: u8,
    /// `NDS`/`NDD` (non-destructive-source/destination) register specifier
    /// (inverted).
    pub evex_vvvv: u8,
    /// Compressed legacy prefix.
    pub evex_pp: u8,
    /// Zeroing/Merging.
    pub evex_z: u8,
    /// Vector-length specifier or rounding-control (most significant bit).
    pub evex_L2: u8,
    /// Vector-length specifier or rounding-control (least significant bit).
    pub evex_L: u8,
    /// Broadcast/RC/SAE context.
    pub evex_b: u8,
    /// High-16 `NDS`/`VIDX` register specifier.
    pub evex_V2: u8,
    /// Embedded opmask register specifier.
    pub evex_aaa: u8,
    /// The offset of the first evex byte, relative to the beginning of the
    /// instruction, in bytes.
    pub evex_offset: u8,
    /// Extension of the `ModRM.reg` field (inverted).
    pub mvex_R: u8,
    /// Extension of the `SIB.index/vidx` field (inverted).
    pub mvex_X: u8,
    /// Extension of the `ModRM.rm` or `SIB.base` field (inverted).
    pub mvex_B: u8,
    /// High-16 register specifier modifier (inverted).
    pub mvex_R2: u8,
    /// Opcode-map specifier.
    pub mvex_mmmm: u8,
    /// 64-bit operand-size promotion or opcode-extension.
    pub mvex_W: u8,
    /// `NDS`/`NDD` (non-destructive-source/destination) register specifier
    /// (inverted).
    pub mvex_vvvv: u8,
    /// Compressed legacy prefix.
    pub mvex_pp: u8,
    /// Non-temporal/eviction hint.
    pub mvex_E: u8,
    /// Swizzle/broadcast/up-convert/down-convert/static-rounding controls.
    pub mvex_SSS: u8,
    /// High-16 `NDS`/`VIDX` register specifier.
    pub mvex_V2: u8,
    /// Embedded opmask register specifier.
    pub mvex_kkk: u8,
    /// The offset of the first mvex byte, relative to the beginning of the
    /// instruction, in bytes.
    pub mvex_offset: u8,
    /// The addressing mode.
    pub modrm_mod: u8,
    /// Register specifier or opcode-extension.
    pub modrm_reg: u8,
    /// Register specifier or opcode-extension.
    pub modrm_rm: u8,
    /// The offset of the `ModRM` byte, relative to the beginning of the
    /// instruction, in bytes.
    pub modrm_offset: u8,
    /// The scale factor.
    pub sib_scale: u8,
    /// The index-register specifier.
    pub sib_index: u8,
    /// The base-register specifier.
    pub sib_base: u8,
    /// THe offset of the `SIB` byte, relative to the beginning of the
    /// instruction, in bytes.
    pub sib_offset: u8,
    /// The displacement value.
    pub disp_value: i64,
    /// THe physical displacement size, in bits.
    pub disp_size: u8,
    /// The offset of the displacement data, relative to the beginning of the
    /// instruction, in bytes.
    pub disp_offset: u8,
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

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct Prefix {
    /// The type of this prefix.
    pub ty: PrefixType,
    /// The value of this prefix.
    pub value: u8,
}

#[repr(C)]
pub struct RegisterContext {
    pub values: [u64; crate::enums::REGISTER_MAX_VALUE + 1],
}

#[derive(Debug)]
#[repr(C)]
pub struct ZydisFormatter {
    style: FormatterStyle,
    force_memory_size: bool,
    force_memory_segment: bool,
    force_relative_branches: bool,
    force_relative_riprel: bool,
    print_branch_size: bool,
    detailed_prefixes: bool,
    addr_base: NumericBase,
    addr_signedness: Signedness,
    addr_padding_absolute: Padding,
    addr_padding_relative: Padding,
    disp_base: NumericBase,
    disp_signedness: Signedness,
    disp_padding: Padding,
    imm_base: NumericBase,
    imm_signedness: Signedness,
    imm_padding: Padding,
    case_prefixes: i32,
    case_mnemonic: i32,
    case_registers: i32,
    case_typecasts: i32,
    case_decorators: i32,
    hex_uppercase: bool,
    // ZYDIS_NUMERIC_BASE_MAX_VALUE + 1
    number_format: [[ZydisFormatterStringData; 2]; 2],

    func_pre_instruction: FormatterFunc,
    func_post_instruction: FormatterFunc,
    func_format_instruction: FormatterFunc,
    func_pre_operand: FormatterFunc,
    func_post_operand: FormatterFunc,
    func_format_operand_reg: FormatterFunc,
    func_format_operand_mem: FormatterFunc,
    func_format_operand_ptr: FormatterFunc,
    func_format_operand_imm: FormatterFunc,
    func_print_mnemonic: FormatterFunc,
    func_print_register: FormatterRegisterFunc,
    func_print_address_abs: FormatterFunc,
    func_print_address_rel: FormatterFunc,
    func_print_disp: FormatterFunc,
    func_print_imm: FormatterFunc,
    func_print_typecast: FormatterFunc,
    func_print_segment: FormatterFunc,
    func_print_prefixes: FormatterFunc,
    func_print_decorator: FormatterDecoratorFunc,
}

#[derive(Debug)]
#[repr(C)]
struct ZydisFormatterStringData {
    string: *const ZyanStringView,
    string_data: ZyanStringView,
    buffer: [c_char; 11],
}

#[derive(Debug)]
#[repr(C)]
pub struct FormatterContext {
    // TODO: Can we do some things with Option<NonNull<T>> here for nicer usage?
    // But how would we enforce constness then?
    /// The instruction being formatted.
    pub instruction: *const DecodedInstruction,
    /// The current operand being formatted.
    pub operand: *const DecodedOperand,
    /// The runtime address of the instruction.
    ///
    /// If invalid, this is equal to u64::max_value()
    pub runtime_address: u64,
    /// A pointer to user-defined data.
    pub user_data: *mut c_void,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct InstructionSegments {
    /// The number of logical instruction segments.
    pub count: u8,
    // ZYDIS_MAX_INSTRUCTION_SEGMENT_COUNT
    pub segments: [InstructionSegmentsElement; 9],
}

impl<'a> IntoIterator for &'a InstructionSegments {
    type IntoIter = slice::Iter<'a, InstructionSegmentsElement>;
    type Item = &'a InstructionSegmentsElement;

    fn into_iter(self) -> Self::IntoIter {
        (&self.segments[..self.count as usize]).into_iter()
    }
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct InstructionSegmentsElement {
    /// The type of this segment.
    pub ty: InstructionSegment,
    /// The offset relative to the start of the instruction.
    pub offset: u8,
    /// The size of the segment, in bytes.
    pub size: u8,
}

#[repr(C)]
pub struct ShortString {
    pub data: *const c_char,
    pub size: u8,
}

/// Returns the version of the zydis C library as a quadruple
/// `(major, minor, patch, build)`.
///
/// # Examples
/// ```
/// use zydis;
/// let (major, minor, patch, build) = zydis::get_version();
/// println!("Zydis version: {}.{}.{}.{}", major, minor, patch, build);
/// assert_eq!(major, 3);
/// ```
pub fn get_version() -> (u16, u16, u16, u16) {
    let combined_ver = unsafe { ZydisGetVersion() };
    let major = ((combined_ver << 0) >> 48) as u16;
    let minor = ((combined_ver << 16) >> 48) as u16;
    let patch = ((combined_ver << 32) >> 48) as u16;
    let build = ((combined_ver << 48) >> 48) as u16;
    (major, minor, patch, build)
}

extern "C" {
    pub fn ZydisGetVersion() -> u64;

    pub fn ZydisIsFeatureEnabled(feature: Feature) -> Status;

    pub fn ZydisCalcAbsoluteAddress(
        instruction: *const DecodedInstruction,
        operand: *const DecodedOperand,
        runtime_address: u64,
        result_address: *mut u64,
    ) -> Status;

    pub fn ZydisCalcAbsoluteAddressEx(
        instruction: *const DecodedInstruction,
        operand: *const DecodedOperand,
        runtime_address: u64,
        register_context: *const RegisterContext,
        result_address: *mut u64,
    ) -> Status;

    pub fn ZydisGetAccessedFlagsByAction(
        instruction: *const DecodedInstruction,
        action: CPUFlagAction,
        flags: *mut CPUFlag,
    ) -> Status;

    pub fn ZydisGetAccessedFlagsRead(
        instruction: *const DecodedInstruction,
        flags: *mut CPUFlag,
    ) -> Status;

    pub fn ZydisGetAccessedFlagsWritten(
        instruction: *const DecodedInstruction,
        flags: *mut CPUFlag,
    ) -> Status;

    pub fn ZydisGetInstructionSegments(
        instruction: *const DecodedInstruction,
        segments: *mut InstructionSegments,
    ) -> Status;

    pub fn ZydisDecoderInit(
        decoder: *mut Decoder,
        machine_mode: MachineMode,
        address_width: AddressWidth,
    ) -> Status;

    pub fn ZydisDecoderEnableMode(
        decoder: *mut Decoder,
        mode: DecoderMode,
        enabled: bool,
    ) -> Status;

    pub fn ZydisDecoderDecodeBuffer(
        decoder: *const Decoder,
        buffer: *const c_void,
        length: usize,
        instruction: *mut DecodedInstruction,
    ) -> Status;

    pub fn ZydisMnemonicGetString(mnemonic: Mnemonic) -> *const c_char;

    pub fn ZydisMnemonicGetShortString(mnemonic: Mnemonic) -> *const ShortString;

    pub fn ZydisRegisterEncode(register_class: RegisterClass, id: u8) -> Register;

    pub fn ZydisRegisterGetId(regster: Register) -> i8;

    pub fn ZydisRegisterGetClass(register: Register) -> RegisterClass;

    pub fn ZydisRegisterGetWidth(mode: MachineMode, register: Register) -> RegisterWidth;

    pub fn ZydisRegisterGetString(register: Register) -> *const c_char;

    pub fn ZydisRegisterGetStringWrapped(register: Register) -> *const ShortString;

    pub fn ZydisRegisterGetLargestEnclosing(mode: MachineMode, reg: Register) -> Register;

    pub fn ZydisRegisterClassGetWidth(mode: MachineMode, class: RegisterClass) -> RegisterWidth;

    pub fn ZydisCategoryGetString(category: InstructionCategory) -> *const c_char;

    pub fn ZydisISASetGetString(isa_set: ISASet) -> *const c_char;

    pub fn ZydisISAExtGetString(isa_ext: ISAExt) -> *const c_char;

    pub fn ZydisFormatterInit(formatter: *mut ZydisFormatter, style: FormatterStyle) -> Status;

    pub fn ZydisFormatterSetProperty(
        formatter: *mut ZydisFormatter,
        property: ZydisFormatterProperty,
        value: usize,
    ) -> Status;

    pub fn ZydisFormatterSetHook(
        formatter: *mut ZydisFormatter,
        hook: FormatterFunction,
        callback: *mut *const c_void,
    ) -> Status;

    pub fn ZydisFormatterFormatInstruction(
        formatter: *const ZydisFormatter,
        instruction: *const DecodedInstruction,
        buffer: *mut c_char,
        buffer_length: usize,
        runtime_address: u64,
    ) -> Status;

    pub fn ZydisFormatterFormatInstructionEx(
        formatter: *const ZydisFormatter,
        instruction: *const DecodedInstruction,
        buffer: *mut c_char,
        buffer_length: usize,
        runtime_address: u64,
        user_data: *mut c_void,
    ) -> Status;

    pub fn ZydisFormatterFormatOperand(
        formatter: *const ZydisFormatter,
        instruction: *const DecodedInstruction,
        operand_index: u8,
        buffer: *mut c_char,
        buffer_length: usize,
        runtime_address: u64,
    ) -> Status;

    pub fn ZydisFormatterFormatOperandEx(
        formatter: *const ZydisFormatter,
        instruction: *const DecodedInstruction,
        operand_index: u8,
        buffer: *mut c_char,
        buffer_length: usize,
        runtime_address: u64,
        user_data: *mut c_void,
    ) -> Status;

    pub fn ZydisFormatterTokenizeInstruction(
        formatter: *const ZydisFormatter,
        instruction: *const DecodedInstruction,
        buffer: *mut c_void,
        length: usize,
        runtime_address: u64,
        token: *mut *const FormatterToken,
    ) -> Status;

    pub fn ZydisFormatterTokenizeInstructionEx(
        formatter: *const ZydisFormatter,
        instruction: *const DecodedInstruction,
        buffer: *mut c_void,
        length: usize,
        runtime_address: u64,
        token: *mut *const FormatterToken,
        user_data: *mut c_void,
    ) -> Status;

    pub fn ZydisFormatterTokenizeOperand(
        formatter: *const ZydisFormatter,
        instruction: *const DecodedInstruction,
        index: u8,
        buffer: *mut c_void,
        length: usize,
        runtime_address: u64,
        token: *mut *const FormatterToken,
    ) -> Status;

    pub fn ZydisFormatterTokenizeOperandEx(
        formatter: *const ZydisFormatter,
        instruction: *const DecodedInstruction,
        index: u8,
        buffer: *mut c_void,
        length: usize,
        runtime_address: u64,
        token: *mut *const FormatterToken,
        user_data: *mut c_void,
    ) -> Status;

    pub fn ZydisFormatterTokenGetValue(
        token: *const FormatterToken,
        ty: *mut Token,
        value: *mut *const c_char,
    ) -> Status;

    pub fn ZydisFormatterTokenNext(token: *mut *const FormatterToken) -> Status;

    pub fn ZydisFormatterBufferGetString(
        buffer: *mut FormatterBuffer,
        string: *mut *mut ZyanString,
    ) -> Status;

    pub fn ZydisFormatterBufferGetToken(
        buffer: *const FormatterBuffer,
        token: *mut *const FormatterToken,
    ) -> Status;

    pub fn ZydisFormatterBufferAppend(buffer: *mut FormatterBuffer, ty: Token) -> Status;

    pub fn ZydisFormatterBufferRemember(
        buffer: *const FormatterBuffer,
        state: *mut FormatterBufferState,
    ) -> Status;

    pub fn ZydisFormatterBufferRestore(
        buffer: *mut FormatterBuffer,
        state: FormatterBufferState,
    ) -> Status;

    // Zycore functions

    pub fn ZyanStringInitCustomBuffer(
        string: *mut ZyanString,
        buffer: *mut c_char,
        capacity: usize,
    ) -> Status;

    pub fn ZyanStringAppend(destination: *mut ZyanString, source: *const ZyanStringView) -> Status;

    pub fn ZyanStringDestroy(string: *mut ZyanString) -> Status;

    pub fn ZyanStringViewInsideBufferEx(
        view: *mut ZyanStringView,
        buffer: *const c_char,
        length: usize,
    ) -> Status;
}
