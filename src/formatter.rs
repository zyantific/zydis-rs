//! Textual instruction formatting routines.

use gen::*;
use status::ZydisResult;
use std::mem;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::slice;


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
    FuncPrintDecorator(ZydisFormatterFormatDecoratorFunc),
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

pub struct Buffer {
    raw: *mut *mut c_char,
    buffer_length: usize,
}

impl Buffer {
    pub fn new(raw: *mut *mut c_char, buffer_length: usize) -> Self {
        Self { raw, buffer_length }
    }

    pub fn append(&mut self, s: &str) -> ZydisResult<()> {
        let bytes = s.as_bytes();
        if bytes.len() + 1 >= self.buffer_length {
            return Err(ZYDIS_STATUS_INSUFFICIENT_BUFFER_SIZE);
        }

        let slice =
            unsafe { slice::from_raw_parts_mut(*(self.raw) as *mut u8, self.buffer_length) };
        (&mut slice[..bytes.len()]).clone_from_slice(bytes);
        slice[bytes.len()] = '\0' as u8;
        unsafe { *self.raw = (*self.raw).offset(bytes.len() as _) };
        Ok(())
    }
}

pub type WrappedNotifyFunc = Fn(&Formatter, &ZydisDecodedInstruction)
    -> ZydisResult<()>;
pub type WrappedFormatFunc = Fn(&Formatter, &mut Buffer, &ZydisDecodedInstruction)
    -> ZydisResult<()>;
pub type WrappedFormatOperandFunc = Fn(
    &Formatter,
    &mut Buffer,
    &ZydisDecodedInstruction,
    &ZydisDecodedOperand,
) -> ZydisResult<()>;
pub type WrappedFormatAddressFunc = Fn(
    &Formatter,
    &mut Buffer,
    &ZydisDecodedInstruction,
    &ZydisDecodedOperand,
    u64,
) -> ZydisResult<()>;
pub type WrappedFormatDecoratorFunc = Fn(
    &Formatter,
    &mut Buffer,
    &ZydisDecodedInstruction,
    &ZydisDecodedOperand,
    ZydisDecoratorType,
) -> ZydisResult<()>;

macro_rules! wrapped_hook_setter{
    ($field_name:ident, $field_type:ty, $func_name:ident, $dispatch_func:ident, $constructor:expr)
        => {
        pub fn $func_name(&mut self, new_func: Box<$field_type>) -> ZydisResult<Hook> {
            self.$field_name = Some(new_func);
            self.set_raw_hook($constructor(Some($dispatch_func)))
        }
    }
}

macro_rules! dispatch_wrapped_func{
    (notify $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name(formatter: *const ZydisFormatter,
                                        instruction: *mut ZydisDecodedInstruction) -> ZydisStatus {
            let formatter = &*(formatter as *const Formatter);
            let r = match formatter.$field_name.as_ref().unwrap()(formatter, &*instruction) {
                Ok(_) => ZYDIS_STATUS_SUCCESS,
                Err(e) => e,
            };
            r as _
        }
    };
    (format $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name(formatter: *const ZydisFormatter, buffer: *mut *mut c_char,
                                        len: usize,
                                        instruction: *mut ZydisDecodedInstruction) -> ZydisStatus {
            let formatter = &*(formatter as *const Formatter);
            let r = match formatter.$field_name.as_ref().unwrap()(formatter,
                                                                  &mut Buffer::new(buffer, len),
                                                                  &*instruction) {
                Ok(_) => ZYDIS_STATUS_SUCCESS,
                Err(e) => e,
            };
            r as _
        }
    };
    (format_operand $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name(formatter: *const ZydisFormatter, buffer: *mut *mut c_char,
                                        len: usize,
                                        instruction: *mut ZydisDecodedInstruction,
                                        operand: *mut ZydisDecodedOperand) -> ZydisStatus {
            let formatter = &*(formatter as *const Formatter);
            let r = match formatter.$field_name.as_ref().unwrap()(formatter,
                                                                  &mut Buffer::new(buffer, len),
                                                                  &*instruction,
                                                                  &*operand) {
                Ok(_) => ZYDIS_STATUS_SUCCESS,
                Err(e) => e,
            };
            r as _
        }
    };
    (format_decorator $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name(formatter: *const ZydisFormatter, buffer: *mut *mut c_char,
                                        len: usize,
                                        instruction: *mut ZydisDecodedInstruction,
                                        operand: *mut ZydisDecodedOperand,
                                        decorator: ZydisDecoratorType) -> ZydisStatus {
            let formatter = &*(formatter as *const Formatter);
            let r = match formatter.$field_name.as_ref().unwrap()(formatter,
                                                                  &mut Buffer::new(buffer, len),
                                                                  &*instruction,
                                                                  &*operand,
                                                                  decorator) {
                Ok(_) => ZYDIS_STATUS_SUCCESS,
                Err(e) => e,
            };
            r as _
        }
    };
    (format_address $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name(formatter: *const ZydisFormatter, buffer: *mut *mut c_char,
                                        len: usize,
                                        instruction: *mut ZydisDecodedInstruction,
                                        operand: *mut ZydisDecodedOperand,
                                        address: u64) -> ZydisStatus {
            let formatter = &*(formatter as *const Formatter);
            let r = match formatter.$field_name.as_ref().unwrap()(formatter,
                                                                  &mut Buffer::new(buffer, len),
                                                                  &*instruction,
                                                                  &*operand,
                                                                  address) {
                Ok(_) => ZYDIS_STATUS_SUCCESS,
                Err(e) => e,
            };
            r as _
        }
    }
}

dispatch_wrapped_func!(notify pre, dispatch_pre);
dispatch_wrapped_func!(notify post, dispatch_post);
dispatch_wrapped_func!(format format_instruction, dispatch_format_instruction);
dispatch_wrapped_func!(format print_prefixes, dispatch_print_prefixes);
dispatch_wrapped_func!(format print_mnemonic, dispatch_print_mnemonic);
dispatch_wrapped_func!(format_operand format_operand_reg, dispatch_format_operand_reg);
dispatch_wrapped_func!(format_operand format_operand_mem, dispatch_format_operand_mem);
dispatch_wrapped_func!(format_operand format_operand_ptr, dispatch_format_operand_ptr);
dispatch_wrapped_func!(format_operand format_operand_imm, dispatch_format_operand_imm);
dispatch_wrapped_func!(format_operand print_operand_size, dispatch_print_operand_size);
dispatch_wrapped_func!(format_operand print_segment, dispatch_print_segment);
dispatch_wrapped_func!(format_decorator print_decorator, dispatch_print_decorator);
dispatch_wrapped_func!(format_address print_address, dispatch_print_address);
dispatch_wrapped_func!(format_operand print_displacement, dispatch_print_displacement);
dispatch_wrapped_func!(format_operand print_immediate, dispatch_print_immediate);

#[repr(C)]
// needed, since we cast a *const ZydisFormatter to a *const Formatter and the rust compiler
// could reorder the fields if this wasn't #[repr(C)].
pub struct Formatter {
    formatter: ZydisFormatter,
    pre: Option<Box<WrappedNotifyFunc>>,
    post: Option<Box<WrappedNotifyFunc>>,
    format_instruction: Option<Box<WrappedFormatFunc>>,
    print_prefixes: Option<Box<WrappedFormatFunc>>,
    print_mnemonic: Option<Box<WrappedFormatFunc>>,
    format_operand_reg: Option<Box<WrappedFormatOperandFunc>>,
    format_operand_mem: Option<Box<WrappedFormatOperandFunc>>,
    format_operand_ptr: Option<Box<WrappedFormatOperandFunc>>,
    format_operand_imm: Option<Box<WrappedFormatOperandFunc>>,
    print_operand_size: Option<Box<WrappedFormatOperandFunc>>,
    print_segment: Option<Box<WrappedFormatOperandFunc>>,
    print_decorator: Option<Box<WrappedFormatDecoratorFunc>>,
    print_address: Option<Box<WrappedFormatAddressFunc>>,
    print_displacement: Option<Box<WrappedFormatOperandFunc>>,
    print_immediate: Option<Box<WrappedFormatOperandFunc>>,
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
                Formatter {
                    formatter,
                    pre: None,
                    post: None,
                    format_instruction: None,
                    print_prefixes: None,
                    print_mnemonic: None,
                    format_operand_reg: None,
                    format_operand_mem: None,
                    format_operand_ptr: None,
                    format_operand_imm: None,
                    print_operand_size: None,
                    print_segment: None,
                    print_decorator: None,
                    print_address: None,
                    print_displacement: None,
                    print_immediate: None,
                }
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
    /// let formatter = zydis::Formatter::new(
    ///     zydis::gen::ZYDIS_FORMATTER_STYLE_INTEL
    /// ).unwrap();
    /// let dec = zydis::Decoder::new(
    ///     zydis::gen::ZYDIS_MACHINE_MODE_LONG_64,
    ///     zydis::gen::ZYDIS_ADDRESS_WIDTH_64
    /// ).unwrap();
    ///
    /// static INT3: &'static [u8] = &[0xCCu8];
    /// let mut info = dec.decode(INT3, 0).unwrap().unwrap();
    /// let fmt = formatter.format_instruction(&mut info, 200).unwrap();
    /// assert_eq!(fmt, "int3");
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
    pub fn set_raw_hook(&mut self, hook: Hook) -> ZydisResult<Hook> {
        unsafe {
            let mut cb = hook.to_raw();
            let hook_id = hook.to_id();

            check!(
                ZydisFormatterSetHook(&mut self.formatter, hook_id as _, &mut cb),
                Hook::from_raw(hook_id, cb)
            )
        }
    }

    wrapped_hook_setter!(pre, WrappedNotifyFunc, set_pre, dispatch_pre, Hook::FuncPre);
    wrapped_hook_setter!(
        post,
        WrappedNotifyFunc,
        set_post,
        dispatch_post,
        Hook::FuncPost
    );
    wrapped_hook_setter!(
        format_instruction,
        WrappedFormatFunc,
        set_format_instruction,
        dispatch_format_instruction,
        Hook::FuncFormatInstruction
    );
    wrapped_hook_setter!(
        print_prefixes,
        WrappedFormatFunc,
        set_print_prefixes,
        dispatch_print_prefixes,
        Hook::FuncPrintPrefixes
    );
    wrapped_hook_setter!(
        print_mnemonic,
        WrappedFormatFunc,
        set_print_mnemonic,
        dispatch_print_mnemonic,
        Hook::FuncPrintMnemonic
    );
    wrapped_hook_setter!(
        format_operand_reg,
        WrappedFormatOperandFunc,
        set_format_operand_reg,
        dispatch_format_operand_reg,
        Hook::FuncFormatOperandReg
    );
    wrapped_hook_setter!(
        format_operand_mem,
        WrappedFormatOperandFunc,
        set_format_operand_mem,
        dispatch_format_operand_mem,
        Hook::FuncFormatOperandMem
    );
    wrapped_hook_setter!(
        format_operand_ptr,
        WrappedFormatOperandFunc,
        set_format_operand_ptr,
        dispatch_format_operand_ptr,
        Hook::FuncFormatOperandPtr
    );
    wrapped_hook_setter!(
        format_operand_imm,
        WrappedFormatOperandFunc,
        set_format_operand_imm,
        dispatch_format_operand_imm,
        Hook::FuncFormatOperandImm
    );
    wrapped_hook_setter!(
        print_operand_size,
        WrappedFormatOperandFunc,
        set_print_operand_size,
        dispatch_print_operand_size,
        Hook::FuncPrintOperandsize
    );
    wrapped_hook_setter!(
        print_segment,
        WrappedFormatOperandFunc,
        set_print_segment,
        dispatch_print_segment,
        Hook::FuncPrintSegment
    );
    wrapped_hook_setter!(
        print_decorator,
        WrappedFormatDecoratorFunc,
        set_print_decorator,
        dispatch_print_decorator,
        Hook::FuncPrintDecorator
    );
    wrapped_hook_setter!(
        print_address,
        WrappedFormatAddressFunc,
        set_print_address,
        dispatch_print_address,
        Hook::FuncPrintAddress
    );
    wrapped_hook_setter!(
        print_displacement,
        WrappedFormatOperandFunc,
        set_print_displacement,
        dispatch_print_displacement,
        Hook::FuncPrintDisplacement
    );
    wrapped_hook_setter!(
        print_immediate,
        WrappedFormatOperandFunc,
        set_print_immediate,
        dispatch_print_immediate,
        Hook::FuncPrintImmediate
    );
}
