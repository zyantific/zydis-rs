//! Provides raw type definitions and function definitions.

// TODO
use crate::{enums::*, status::Status};

use std::os::raw::{c_char, c_void};

pub type FormatterFunc =
    Option<extern "C" fn(*const Formatter, *mut ZyanString, *mut Context) -> Status>;

pub type FormatterAddressFunc =
    Option<extern "C" fn(*const Formatter, *mut ZyanString, *mut Context, u64) -> Status>;

pub type FormatterDecoratorFunc =
    Option<extern "C" fn(*const Formatter, *mut ZyanString, *mut Context, DecoratorType) -> Status>;

pub type FormatterRegisterFunc =
    Option<extern "C" fn(*const Formatter, *mut ZyanString, *mut Context, Register) -> Status>;

pub type RegisterWidth = u16;

#[repr(C)]
pub struct ShortString {
    pub data: *const c_char,
    pub size: u8,
}

#[link(name = "zydis")]
extern "C" {
    fn ZydisGetVersion() -> u64;

    fn ZydisIsFeatureEnabled(feature: Feature) -> Status;

    fn ZydisCalcAbsoluteAddress(
        instruction: *const DecodedInstruction,
        operand: *const DecodedOperand,
        runtime_address: u64,
        target_address: *mut u64,
    ) -> Status;

    fn ZydisGetAccessedFlagsByAction(
        instruction: *const DecodedInstruction,
        action: CPUFlagAction,
        flags: *mut CPUFlag,
    ) -> Status;

    fn ZydisGetAccessedFlagsRead(
        instruction: *const DecodedInstruction,
        flags: *mut CPUFlag,
    ) -> Status;

    fn ZydisGetAccessedFlagsWritten(
        instruction: *const DecodedInstruction,
        flags: *mut CPUFlag,
    ) -> Status;

    fn ZydisGetInstructionSegments(
        instruction: *const DecodedInstruction,
        segments: *mut InstructionSegment,
    ) -> Status;

    fn ZydisDecoderInit(
        decoder: *mut Decoder,
        machine_mode: MachineMode,
        address_width: AddressWidth,
    ) -> Status;

    fn ZydisDecoderEnableMode(decoder: *mut Decoder, mode: DecoderMode, enabled: bool) -> Status;

    fn ZydisDecoderDecodeBuffer(
        decoder: *const Decoder,
        buffer: *const c_void,
        length: usize,
        instruction: *mut DecodedInstruction,
    ) -> Status;

    fn ZydisMnemonicGetString(mnemonic: Mnemonic) -> *const c_char;

    fn ZydisMnemonicGetShortString(mnemonic: Mnemonic) -> *const ShortString;

    fn ZydisRegisterEncode(register_class: RegisterClass, id: u8) -> Register;

    fn ZydisRegisterGetId(regster: Register) -> i16;

    fn ZydisRegisterGetClass(register: Register) -> RegisterClass;

    fn ZydisRegisterGetWidth(register: Register) -> RegisterWidth;

    fn ZydisRegisterGetWidth64(register: Register) -> RegisterWidth;

    fn ZydisRegisterGetString(register: Register) -> *const c_char;

    fn ZydisRegisterGetStringWrapped(register: Register) -> *const ShortString;

    fn ZydisCategoryGetString(category: InstructionCategory) -> *const c_char;

    fn ZydisISASetGetString(isa_set: ISASet) -> *const c_char;

    fn ZydisISAExtGetString(isa_ext: ISAExt) -> *const c_char;

    fn ZydisFormatterInit(formatter: *mut Formatter, style: FormatterStyle) -> Status;

    fn ZydisFormatterSetProperty(
        formatter: *mut Formatter,
        property: FormatterProperty,
        value: usize,
    ) -> Status;

    fn ZydisFormatterSetHook(
        formatter: *mut Formatter,
        hook: HookType,
        callback: *mut *const c_void,
    ) -> Status;

    fn ZydisFormatterFormatInstruction(
        formatter: *const Formatter,
        instruction: *const DecodedInstruction,
        buffer: *mut c_char,
        buffer_length: usize,
        address: u64,
    ) -> Status;

    fn ZydisFormatterFormatInstructionEx(
        formatter: *const Formatter,
        instruction: *const DecodedInstruction,
        buffer: *mut c_char,
        buffer_length: usize,
        address: u64,
        user_data: *mut c_void,
    ) -> Status;

    fn ZydisFormatterFormatOperand(
        formatter: *const Formatter,
        instruction: *const DecodedInstruction,
        operand_index: u8,
        buffer: *mut c_char,
        buffer_length: usize,
        address: u64,
    ) -> Status;

    fn ZydisFormatterFormatOperandEx(
        formatter: *const Formatter,
        instruction: *const DecodedInstruction,
        operand_index: u8,
        buffer: *mut c_char,
        buffer_length: usize,
        address: u64,
        user_data: *mut c_void,
    ) -> Status;
}
