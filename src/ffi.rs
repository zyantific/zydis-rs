//! Provides type aliases, struct definitions and the unsafe function
//! declrations.

use core::{fmt, mem, ptr};

// TODO: use libc maybe, or wait for this to move into core?
use std::os::raw::{c_char, c_void};

use super::{
    enums::*,
    status::{Result, Status},
};

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type FormatterFunc = Option<unsafe extern "C" fn(
    *const ZydisFormatter,
    *mut FormatterBuffer,
    *mut FormatterContext) -> Status>;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type FormatterAddressFunc = Option<unsafe extern "C" fn(
    *const ZydisFormatter,
    *mut FormatterBuffer,
    *mut FormatterContext,
    u64) -> Status>;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type FormatterDecoratorFunc = Option<unsafe extern "C" fn(
    *const ZydisFormatter,
    *mut FormatterBuffer,
    *mut FormatterContext,
    Decorator) -> Status>;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type FormatterRegisterFunc = Option<unsafe extern "C" fn(
    *const ZydisFormatter,
    *mut FormatterBuffer,
    *mut FormatterContext,
    Register) -> Status>;

pub type RegisterWidth = u16;

#[derive(Debug)]
#[repr(C, packed)]
pub struct FormatterToken {
    ty: Token,
    next: u8,
}

impl FormatterToken {
    /// Returns the value and type of this token.
    // TODO: How about not returning a *mut c_char here.
    pub fn get_value(&self) -> Result<(Token, *mut c_char)> {
        unsafe {
            let mut ty = mem::uninitialized();
            let mut val = mem::uninitialized();
            check!(
                ZydisFormatterTokenGetValue(self, &mut ty, &mut val),
                (ty, val)
            )
        }
    }

    /// Returns the next token.
    pub fn next(&self) -> Result<FormatterToken> {
        unsafe {
            let mut res = self as *const FormatterToken;
            check!(ZydisFormatterTokenNext(&mut res))?;

            if res.is_null() {
                Err(Status::User)
            } else {
                Ok(ptr::read(res))
            }
        }
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
    pub fn get_string<'a>(&'a mut self) -> Result<&'a mut ZyanString> {
        unsafe {
            let mut str = mem::uninitialized();
            check!(ZydisFormatterBufferGetString(self, &mut str))?;

            if str.is_null() {
                Err(Status::User)
            } else {
                Ok(&mut *str)
            }
        }
    }

    /// Appends a new token to this buffer.
    pub fn append(&mut self, token: Token) -> Result<()> {
        unsafe { check!(ZydisFormatterBufferAppend(self, token)) }
    }

    /// Returns a snapshot of the buffer-state.
    pub fn remember(&self) -> Result<FormatterBufferState> {
        unsafe {
            let mut res = mem::uninitialized();
            check!(ZydisFormatterBufferRemember(self, &mut res), res)
        }
    }

    /// Restores a previously saved buffer-state.
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
    pub fn new_ptr(buffer: *mut u8, capacity: usize) -> Result<Self> {
        unsafe {
            let mut string = mem::uninitialized();
            check!(ZyanStringInitCustomBuffer(
                &mut string,
                buffer as *mut i8,
                capacity
            ))?;
            Ok(string)
        }
    }

    /// Appends the given string `s` to this buffer.
    ///
    /// Warning: The actual Rust `&str`ings are encoded in UTF-8 and aren't
    /// converted to any other encoding. They're simply copied, byte by
    /// byte, to the buffer. Therefore, the buffer should be interpreted as
    /// UTF-8 when later being printed.
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
    pub fn new(buffer: &[u8]) -> Result<Self> {
        unsafe {
            let mut view = mem::uninitialized();
            check!(ZyanStringViewInsideViewEx(
                &mut view,
                buffer.as_ptr() as *const i8,
                buffer.len()
            ))?;
            Ok(view)
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
    pub fn new(machine_mode: MachineMode, address_width: AddressWidth) -> Result<Self> {
        unsafe {
            let mut decoder = mem::uninitialized();
            check!(
                ZydisDecoderInit(&mut decoder, machine_mode, address_width),
                decoder
            )
        }
    }

    /// Enables or disables (depending on the `value`) the given decoder `mode`:
    pub fn enable_mode(&mut self, mode: DecoderMode, value: bool) -> Result<()> {
        unsafe { check!(ZydisDecoderEnableMode(self, mode, value as _)) }
    }

    /// Decodes a single binary instruction to a `DecodedInstruction`.
    ///
    /// # Examples
    /// ```
    /// use zydis::{AddressWidth, Decoder, MachineMode, Mnemonic};
    /// static INT3: &'static [u8] = &[0xCC];
    /// let decoder = Decoder::new(MachineMode::Long64, AddressWidth::_64).unwrap();
    ///
    /// let instruction = decoder.decode(INT3).unwrap().unwrap();
    /// assert_eq!(instruction.mnemonic, Mnemonic::INT3);
    /// ```
    pub fn decode(&self, buffer: &[u8]) -> Result<Option<DecodedInstruction>> {
        unsafe {
            let mut instruction = mem::uninitialized();
            check_option!(
                ZydisDecoderDecodeBuffer(
                    self,
                    buffer.as_ptr() as *const c_void,
                    buffer.len(),
                    &mut instruction
                ),
                instruction
            )
        }
    }

    /// Returns an iterator over all the instructions in the buffer.
    ///
    /// The iterator ignores all errors and stops producing values in the case
    /// of an error.
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

impl<'a, 'b> Iterator for InstructionIterator<'a, 'b> {
    type Item = (DecodedInstruction, u64);

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
#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct MemoryInfo {
    pub ty: OperandType,
    pub segment: Register,
    pub base: Register,
    pub index: Register,
    pub scale: u8,
    /// Signals if displacement is present.
    pub has_displacement: bool,
    /// The displacement value
    pub displacement: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct PointerInfo {
    pub segment: u16,
    pub offset: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
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
    pub fn calc_absolute_address(&self, address: u64, operand: &DecodedOperand) -> Result<u64> {
        unsafe {
            let mut addr = 0u64;
            check!(
                ZydisCalcAbsoluteAddress(self, operand, address, &mut addr),
                addr
            )
        }
    }

    /// Returns a mask of CPU-flags that match the given `action`.
    pub fn get_flags(&self, action: CPUFlagAction) -> Result<CPUFlag> {
        unsafe {
            let mut flags = mem::uninitialized();
            check!(
                ZydisGetAccessedFlagsByAction(self, action, &mut flags),
                flags
            )
        }
    }

    /// Returns a mask of CPU-flags that are read (tested) by this instruction.
    pub fn get_flags_read(&self) -> Result<CPUFlag> {
        unsafe {
            let mut flags = mem::uninitialized();
            check!(ZydisGetAccessedFlagsRead(self, &mut flags), flags)
        }
    }

    /// Returns a mask of CPU-flags that are written (modified, undefined) by
    /// this instruction.
    pub fn get_flags_written(&self) -> Result<CPUFlag> {
        unsafe {
            let mut flags = mem::uninitialized();
            check!(ZydisGetAccessedFlagsWritten(self, &mut flags), flags)
        }
    }

    /// Returns offsets and sizes of all logical instruction segments.
    pub fn get_segments(&self) -> Result<InstructionSegments> {
        unsafe {
            let mut segments = mem::uninitialized();
            check!(ZydisGetInstructionSegments(self, &mut segments), segments)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct MetaInfo {
    /// The category this instruction belongs to.
    pub category: InstructionCategory,
    /// The instruction set this instruction belongs to.
    pub isa_set: ISASet,
    /// The instruction set extension this instruction belongs to.
    pub isa_ext: ISAExt,
    /// The exception class of this instruction.
    pub exception_class: ExceptionClass,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct Prefix {
    /// The type of this prefix.
    pub ty: PrefixType,
    /// The value of this prefix.
    pub value: u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct ZydisFormatter {
    letter_case: u32,
    force_memory_size: bool,
    force_memory_segment: bool,
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
    func_print_address_abs: FormatterAddressFunc,
    func_print_address_rel: FormatterAddressFunc,
    func_print_disp: FormatterFunc,
    func_print_imm: FormatterFunc,
    func_print_size: FormatterFunc,
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct InstructionSegments {
    /// The number of logical instruction segments.
    pub count: u8,
    pub segments: [InstructionSegmentsElement; 8],
}

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

extern "C" {
    pub fn ZydisGetVersion() -> u64;

    pub fn ZydisIsFeatureEnabled(feature: Feature) -> Status;

    pub fn ZydisCalcAbsoluteAddress(
        instruction: *const DecodedInstruction,
        operand: *const DecodedOperand,
        runtime_address: u64,
        target_address: *mut u64,
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

    pub fn ZydisRegisterGetId(regster: Register) -> i16;

    pub fn ZydisRegisterGetClass(register: Register) -> RegisterClass;

    pub fn ZydisRegisterGetWidth(register: Register) -> RegisterWidth;

    pub fn ZydisRegisterGetWidth64(register: Register) -> RegisterWidth;

    pub fn ZydisRegisterGetString(register: Register) -> *const c_char;

    pub fn ZydisRegisterGetStringWrapped(register: Register) -> *const ShortString;

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
        hook: HookType,
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

    pub fn ZydisFormatterTokenGetValue(
        token: *const FormatterToken,
        ty: *mut Token,
        value: *mut *mut c_char,
    ) -> Status;

    pub fn ZydisFormatterTokenNext(token: *mut *const FormatterToken) -> Status;

    pub fn ZydisFormatterBufferGetString(
        buffer: *mut FormatterBuffer,
        string: *mut *mut ZyanString,
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

    pub fn ZyanStringViewInsideViewEx(
        view: *mut ZyanStringView,
        buffer: *const c_char,
        length: usize,
    ) -> Status;
}
