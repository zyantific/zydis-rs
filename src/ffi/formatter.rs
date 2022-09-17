use super::*;
use crate::ffi;

pub type FormatterFunc = Option<
    unsafe extern "C" fn(
        *const Formatter,
        *mut FormatterBuffer,
        *mut FormatterContext,
    ) -> Status,
>;

pub type FormatterDecoratorFunc = Option<
    unsafe extern "C" fn(
        *const Formatter,
        *mut FormatterBuffer,
        *mut FormatterContext,
        Decorator,
    ) -> Status,
>;

pub type FormatterRegisterFunc = Option<
    unsafe extern "C" fn(
        *const Formatter,
        *mut FormatterBuffer,
        *mut FormatterContext,
        Register,
    ) -> Status,
>;

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

#[derive(Debug)]
#[repr(C)]
pub struct Formatter {
    style: FormatterStyle,
    force_memory_size: bool,
    force_memory_segment: bool,
    force_memory_scale: bool,
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
    hex_force_leading_number: bool,
    number_format: [[ZydisFormatterStringData; NUMERIC_BASE_MAX_VALUE + 1]; 2],

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
    //       But how would we enforce constness then?
    /// The instruction being formatted.
    pub instruction: *const ffi::DecodedInstruction,
    pub operands: *const ffi::DecodedOperand,
    /// The current operand being formatted.
    pub operand: *const ffi::DecodedOperand,
    /// The runtime address of the instruction.
    ///
    /// If invalid, this is equal to u64::max_value()
    pub runtime_address: u64,
    /// A pointer to user-defined data.
    pub user_data: *mut c_void,
}

extern "C" {
    pub fn ZydisFormatterInit(formatter: *mut Formatter, style: FormatterStyle) -> Status;

    pub fn ZydisFormatterSetProperty(
        formatter: *mut Formatter,
        property: ZydisFormatterProperty,
        value: usize,
    ) -> Status;

    pub fn ZydisFormatterSetHook(
        formatter: *mut Formatter,
        hook: FormatterFunction,
        callback: *mut *const c_void,
    ) -> Status;

    pub fn ZydisFormatterFormatInstruction(
        formatter: *const Formatter,
        instruction: *const DecodedInstruction,
        operands: *const DecodedOperand,
        operand_count: u8,
        buffer: *mut c_char,
        buffer_length: usize,
        runtime_address: u64,
    ) -> Status;

    pub fn ZydisFormatterFormatInstructionEx(
        formatter: *const Formatter,
        instruction: *const DecodedInstruction,
        operands: *const DecodedOperand,
        operand_count: u8,
        buffer: *mut c_char,
        buffer_length: usize,
        runtime_address: u64,
        user_data: *mut c_void,
    ) -> Status;

    pub fn ZydisFormatterFormatOperand(
        formatter: *const Formatter,
        instruction: *const DecodedInstruction,
        operand: *const DecodedOperand,
        buffer: *mut c_char,
        buffer_length: usize,
        runtime_address: u64,
    ) -> Status;

    pub fn ZydisFormatterFormatOperandEx(
        formatter: *const Formatter,
        instruction: *const DecodedInstruction,
        operand: *const DecodedOperand,
        buffer: *mut c_char,
        buffer_length: usize,
        runtime_address: u64,
        user_data: *mut c_void,
    ) -> Status;

    pub fn ZydisFormatterTokenizeInstruction(
        formatter: *const Formatter,
        instruction: *const DecodedInstruction,
        operands: *const DecodedOperand,
        operand_count: u8,
        buffer: *mut c_void,
        length: usize,
        runtime_address: u64,
        token: *mut *const FormatterToken,
    ) -> Status;

    pub fn ZydisFormatterTokenizeInstructionEx(
        formatter: *const Formatter,
        instruction: *const DecodedInstruction,
        operands: *const DecodedOperand,
        operand_count: u8,
        buffer: *mut c_void,
        length: usize,
        runtime_address: u64,
        token: *mut *const FormatterToken,
        user_data: *mut c_void,
    ) -> Status;

    pub fn ZydisFormatterTokenizeOperand(
        formatter: *const Formatter,
        instruction: *const DecodedInstruction,
        operand: *const DecodedOperand,
        buffer: *mut c_void,
        length: usize,
        runtime_address: u64,
        token: *mut *const FormatterToken,
    ) -> Status;

    pub fn ZydisFormatterTokenizeOperandEx(
        formatter: *const Formatter,
        instruction: *const DecodedInstruction,
        operand: *const DecodedOperand,
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
}
