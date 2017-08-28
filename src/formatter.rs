//! Textual instruction formatting routines.

use gen::*;
use status::ZydisResult;
use std::mem;
use std::ffi::CStr;
use std::os::raw::c_void;


#[derive(Clone)]
pub enum Hook {
    FuncPre(ZydisFormatterNotifyFunc),
    FuncPost(ZydisFormatterNotifyFunc),
    FuncFormatInstruction(ZydisFormatterFormatFunc),
    FuncPrintPrefixes(ZydisFormatterFormatFunc),
    FuncPrintMnemonic(ZydisFormatterFormatFunc),
    FuncFormatOperandReg(ZydisFormatterFormatOperandFunc),
    FuncFormatOperandMem(ZydisFormatterFormatOperandFunc),
    FuncFormatOperandPtr(ZydisFormatterFormatOperandFunc),
    FuncFormatOperandImm(ZydisFormatterFormatOperandFunc),
    FuncPrintOperandsize(ZydisFormatterFormatOperandFunc),
    FuncPrintSegment(ZydisFormatterFormatOperandFunc),
    FuncPrintDecorator(ZydisFormatterFormatOperandFunc),
    FuncPrintDisplacement(ZydisFormatterFormatOperandFunc),
    FuncPrintImmediate(ZydisFormatterFormatOperandFunc),
    FuncPrintAddress(ZydisFormatterFormatAddressFunc),
}

impl Hook {
    pub fn to_id(&self) -> ZydisFormatterHookTypes {
        use self::Hook::*;
        match *self {
            FuncPre(_) => ZYDIS_FORMATTER_HOOK_PRE,
            FuncPost(_) => ZYDIS_FORMATTER_HOOK_POST,
            FuncFormatInstruction(_) => ZYDIS_FORMATTER_HOOK_FORMAT_INSTRUCTION,
            FuncPrintPrefixes(_) => ZYDIS_FORMATTER_HOOK_PRINT_PREFIXES,
            FuncPrintMnemonic(_) => ZYDIS_FORMATTER_HOOK_PRINT_MNEMONIC,
            FuncFormatOperandReg(_) => ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_REG,
            FuncFormatOperandMem(_) => ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_MEM,
            FuncFormatOperandPtr(_) => ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_PTR,
            FuncFormatOperandImm(_) => ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_IMM,
            FuncPrintOperandsize(_) => ZYDIS_FORMATTER_HOOK_PRINT_OPERANDSIZE,
            FuncPrintSegment(_) => ZYDIS_FORMATTER_HOOK_PRINT_SEGMENT,
            FuncPrintDecorator(_) => ZYDIS_FORMATTER_HOOK_PRINT_DECORATOR,
            FuncPrintDisplacement(_) => ZYDIS_FORMATTER_HOOK_PRINT_DISPLACEMENT,
            FuncPrintImmediate(_) => ZYDIS_FORMATTER_HOOK_PRINT_IMMEDIATE,
            FuncPrintAddress(_) => ZYDIS_FORMATTER_HOOK_PRINT_ADDRESS,
        }
    }

    pub unsafe fn to_raw(&self) -> *const c_void {
        use self::Hook::*;
        match *self {
            FuncPre(x) => mem::transmute(x),
            FuncPost(x) => mem::transmute(x),
            FuncFormatInstruction(x) => mem::transmute(x),
            FuncPrintPrefixes(x) => mem::transmute(x),
            FuncPrintMnemonic(x) => mem::transmute(x),
            FuncFormatOperandReg(x) => mem::transmute(x),
            FuncFormatOperandMem(x) => mem::transmute(x),
            FuncFormatOperandPtr(x) => mem::transmute(x),
            FuncFormatOperandImm(x) => mem::transmute(x),
            FuncPrintOperandsize(x) => mem::transmute(x),
            FuncPrintSegment(x) => mem::transmute(x),
            FuncPrintDecorator(x) => mem::transmute(x),
            FuncPrintDisplacement(x) => mem::transmute(x),
            FuncPrintImmediate(x) => mem::transmute(x),
            FuncPrintAddress(x) => mem::transmute(x),
        }
    }

    pub unsafe fn from_raw(id: ZydisFormatterHookTypes, cb: *const c_void) -> Hook {
        use self::Hook::*;
        match id {
            ZYDIS_FORMATTER_HOOK_PRE => FuncPre(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_POST => FuncPost(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_FORMAT_INSTRUCTION => FuncFormatInstruction(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_PREFIXES => FuncPrintPrefixes(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_MNEMONIC => FuncPrintMnemonic(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_REG => FuncFormatOperandReg(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_MEM => FuncFormatOperandMem(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_PTR => FuncFormatOperandPtr(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_IMM => FuncFormatOperandImm(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_OPERANDSIZE => FuncPrintOperandsize(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_SEGMENT => FuncPrintSegment(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_DECORATOR => FuncPrintDecorator(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_DISPLACEMENT => FuncPrintDisplacement(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_IMMEDIATE => FuncPrintImmediate(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_ADDRESS => FuncPrintAddress(mem::transmute(cb)),
            _ => unreachable!(),
        }
    }
}

pub struct Formatter {
    formatter: ZydisFormatter,
}

impl Formatter {
    /// Creates a new formatter instance, accepting formatter flags.
    pub fn new_ex(
        style: ZydisFormatterStyles,
        flags: ZydisFormatterFlags,
        address_format: ZydisFormatterAddressFormats,
        displacement_format: ZydisFormatterDisplacementFormats,
        immediate_format: ZydisFormatterImmediateFormats,
    ) -> ZydisResult<Self> {
        unsafe {
            let mut formatter = mem::uninitialized();
            check!(
                ZydisFormatterInitEx(
                    &mut formatter,
                    style as _,
                    flags,
                    address_format as _,
                    displacement_format as _,
                    immediate_format as _,
                ),
                Formatter { formatter }
            )
        }
    }

    /// Creates a new formatter instance.
    pub fn new(style: ZydisFormatterStyles) -> ZydisResult<Self> {
        Self::new_ex(
            style,
            0,
            ZYDIS_FORMATTER_ADDR_DEFAULT,
            ZYDIS_FORMATTER_DISP_DEFAULT,
            ZYDIS_FORMATTER_IMM_DEFAULT,
        )
    }

    /// Formats the given instruction, returning a string.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut formatter = zydis::Formatter::new(
    ///     zydis::gen::ZYDIS_FORMATTER_STYLE_INTEL
    /// ).unwrap();
    /// let mut dec = zydis::Decoder::new(
    ///     zydis::gen::ZYDIS_MACHINE_MODE_LONG_64,
    ///     zydis::gen::ZYDIS_ADDRESS_WIDTH_64
    /// ).unwrap();
    ///
    /// static INT3: &'static [u8] = &[0xCCu8];
    /// let mut info = dec.decode(INT3, 0).unwrap().unwrap();
    /// let fmt = formatter.format_instruction(&mut info, 200).unwrap();
    /// assert_eq!(fmt, "int3 ");
    /// ```
    pub fn format_instruction(
        &self,
        instruction: &mut ZydisDecodedInstruction,
        size: usize,
    ) -> ZydisResult<String> {
        let mut buffer = vec![0u8; size];
        self.format_instruction_raw(instruction, &mut buffer)
            .map(|_| {
                unsafe { CStr::from_ptr(buffer.as_ptr() as _) }
                    .to_string_lossy()
                    .into()
            })
    }

    pub fn format_instruction_raw(
        &self,
        instruction: &mut ZydisDecodedInstruction,
        buffer: &mut [u8],
    ) -> ZydisResult<()> {
        unsafe {
            check!(
                ZydisFormatterFormatInstruction(
                    &self.formatter,
                    instruction,
                    buffer.as_ptr() as _,
                    buffer.len()
                ),
                ()
            )
        }
    }

    /// Sets a hook, allowing for customizations along the formatting process.
    pub fn set_hook(&mut self, hook: Hook) -> ZydisResult<Hook> {
        unsafe {
            let mut cb = hook.to_raw();
            let hook_id = hook.to_id();

            check!(
                ZydisFormatterSetHook(&mut self.formatter, hook_id as _, &mut cb),
                Hook::from_raw(hook_id, cb)
            )
        }
    }
}
