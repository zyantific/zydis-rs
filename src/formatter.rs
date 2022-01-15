//! Textual instruction formatting routines.

use core::{
    any::Any,
    fmt,
    mem::{self, MaybeUninit},
    ptr,
};

use std::{ffi::CStr, os::raw::c_void};

use super::{
    enums::*,
    ffi::*,
    status::{Result, Status},
};

#[derive(Clone)]
pub enum Hook {
    PreInstruction(FormatterFunc),
    PostInstruction(FormatterFunc),
    PreOperand(FormatterFunc),
    PostOperand(FormatterFunc),
    FormatInstruction(FormatterFunc),
    FormatOperandReg(FormatterFunc),
    FormatOperandMem(FormatterFunc),
    FormatOperandPtr(FormatterFunc),
    FormatOperandImm(FormatterFunc),
    PrintMnemonic(FormatterFunc),
    PrintRegister(FormatterRegisterFunc),
    PrintAddressAbs(FormatterFunc),
    PrintAddressRel(FormatterFunc),
    PrintDisp(FormatterFunc),
    PrintImm(FormatterFunc),
    PrintTypecast(FormatterFunc),
    PrintSegment(FormatterFunc),
    PrintPrefixes(FormatterFunc),
    PrintDecorator(FormatterDecoratorFunc),
}

impl Hook {
    #[rustfmt::skip]
    pub fn to_id(&self) -> FormatterFunction {
        use self::Hook::*;
        match self {
            PreInstruction(_)    => FormatterFunction::PRE_INSTRUCTION,
            PostInstruction(_)   => FormatterFunction::POST_INSTRUCTION,
            PreOperand(_)        => FormatterFunction::PRE_OPERAND,
            PostOperand(_)       => FormatterFunction::POST_OPERAND,
            FormatInstruction(_) => FormatterFunction::FORMAT_INSTRUCTION,
            FormatOperandReg(_)  => FormatterFunction::FORMAT_OPERAND_REG,
            FormatOperandMem(_)  => FormatterFunction::FORMAT_OPERAND_MEM,
            FormatOperandPtr(_)  => FormatterFunction::FORMAT_OPERAND_PTR,
            FormatOperandImm(_)  => FormatterFunction::FORMAT_OPERAND_IMM,
            PrintMnemonic(_)     => FormatterFunction::PRINT_MNEMONIC,
            PrintRegister(_)     => FormatterFunction::PRINT_REGISTER,
            PrintAddressAbs(_)   => FormatterFunction::PRINT_ADDRESS_ABS,
            PrintAddressRel(_)   => FormatterFunction::PRINT_ADDRESS_REL,
            PrintDisp(_)         => FormatterFunction::PRINT_DISP,
            PrintImm(_)          => FormatterFunction::PRINT_IMM,
            PrintTypecast(_)     => FormatterFunction::PRINT_TYPECAST,
            PrintSegment(_)      => FormatterFunction::PRINT_SEGMENT,
            PrintPrefixes(_)     => FormatterFunction::PRINT_PREFIXES,
            PrintDecorator(_)    => FormatterFunction::PRINT_DECORATOR,
        }
    }

    pub unsafe fn to_raw(&self) -> *const c_void {
        use self::Hook::*;
        // Note: do not remove the `*` at `*self`, Rust 1.26 will segfault
        // since we don't give explicit types for mem::transmute.
        match *self {
            PreInstruction(x) | PostInstruction(x) | PrintPrefixes(x) | FormatInstruction(x)
            | PrintMnemonic(x) | PreOperand(x) | PostOperand(x) | FormatOperandReg(x)
            | FormatOperandMem(x) | FormatOperandPtr(x) | FormatOperandImm(x)
            | PrintAddressAbs(x) | PrintAddressRel(x) | PrintDisp(x) | PrintImm(x)
            | PrintTypecast(x) | PrintSegment(x) => mem::transmute(x),

            PrintRegister(x) => mem::transmute(x),
            PrintDecorator(x) => mem::transmute(x),
        }
    }

    #[rustfmt::skip]
    pub unsafe fn from_raw(id: FormatterFunction, cb: *const c_void) -> Hook {
        use self::Hook::*;
        match id {
            FormatterFunction::PRE_INSTRUCTION    => PreInstruction(mem::transmute(cb)),
            FormatterFunction::POST_INSTRUCTION   => PostInstruction(mem::transmute(cb)),
            FormatterFunction::FORMAT_INSTRUCTION => FormatInstruction(mem::transmute(cb)),
            FormatterFunction::PRE_OPERAND        => PreOperand(mem::transmute(cb)),
            FormatterFunction::POST_OPERAND       => PostOperand(mem::transmute(cb)),
            FormatterFunction::FORMAT_OPERAND_REG => FormatOperandReg(mem::transmute(cb)),
            FormatterFunction::FORMAT_OPERAND_MEM => FormatOperandMem(mem::transmute(cb)),
            FormatterFunction::FORMAT_OPERAND_PTR => FormatOperandPtr(mem::transmute(cb)),
            FormatterFunction::FORMAT_OPERAND_IMM => FormatOperandImm(mem::transmute(cb)),
            FormatterFunction::PRINT_MNEMONIC     => PrintMnemonic(mem::transmute(cb)),
            FormatterFunction::PRINT_REGISTER     => PrintRegister(mem::transmute(cb)),
            FormatterFunction::PRINT_ADDRESS_ABS  => PrintAddressAbs(mem::transmute(cb)),
            FormatterFunction::PRINT_ADDRESS_REL  => PrintAddressRel(mem::transmute(cb)),
            FormatterFunction::PRINT_DISP         => PrintDisp(mem::transmute(cb)),
            FormatterFunction::PRINT_IMM          => PrintImm(mem::transmute(cb)),
            FormatterFunction::PRINT_TYPECAST     => PrintTypecast(mem::transmute(cb)),
            FormatterFunction::PRINT_SEGMENT      => PrintSegment(mem::transmute(cb)),
            FormatterFunction::PRINT_PREFIXES     => PrintPrefixes(mem::transmute(cb)),
            FormatterFunction::PRINT_DECORATOR    => PrintDecorator(mem::transmute(cb)),
        }
    }
}

#[rustfmt::skip]
pub type WrappedGeneralFunc = dyn Fn(
    &Formatter,
    &mut FormatterBuffer,
    &mut FormatterContext,
    Option<&mut dyn Any>
) -> Result<()>;

#[rustfmt::skip]
pub type WrappedRegisterFunc = dyn Fn(
    &Formatter,
    &mut FormatterBuffer,
    &mut FormatterContext,
    Register,
    Option<&mut dyn Any>
) -> Result<()>;

#[rustfmt::skip]
pub type WrappedDecoratorFunc = dyn Fn(
    &Formatter,
    &mut FormatterBuffer,
    &mut FormatterContext,
    Decorator,
    Option<&mut dyn Any>
) -> Result<()>;

macro_rules! wrapped_hook_setter{
    ($field_name:ident, $field_type:ty, $func_name:ident, $dispatch_func:ident, $constructor:expr)
        => {
        /// Sets the formatter hook to the provided value.
        ///
        /// This function accepts a wrapped version of the raw hook.
        /// It returns the previous set *raw* hook.
        #[inline]
        pub fn $func_name(&mut self, new_func: Box<$field_type>) -> Result<Hook> {
            self.$field_name = Some(new_func);
            unsafe { self.set_raw_hook($constructor(Some($dispatch_func))) }
        }
    };
}

unsafe fn get_user_data<'a>(user_data: *mut c_void) -> Option<&'a mut dyn Any> {
    if user_data.is_null() {
        None
    } else {
        Some(*(user_data as *mut &mut dyn Any))
    }
}

macro_rules! wrap_func {
    (general $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name(
            formatter: *const ZydisFormatter,
            buffer: *mut FormatterBuffer,
            ctx: *mut FormatterContext,
        ) -> Status {
            let formatter = &*(formatter as *const Formatter);
            let ctx = &mut *ctx;
            let usr = get_user_data(ctx.user_data);
            match formatter.$field_name.as_ref().unwrap()(formatter, &mut *buffer, ctx, usr) {
                Ok(_) => Status::Success,
                Err(e) => e,
            }
        }
    };
    (register $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name(
            formatter: *const ZydisFormatter,
            buffer: *mut FormatterBuffer,
            ctx: *mut FormatterContext,
            reg: Register,
        ) -> Status {
            let formatter = &*(formatter as *const Formatter);
            let ctx = &mut *ctx;
            let usr = get_user_data(ctx.user_data);
            match formatter.$field_name.as_ref().unwrap()(formatter, &mut *buffer, ctx, reg, usr) {
                Ok(_) => Status::Success,
                Err(e) => e,
            }
        }
    };
    (decorator $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name(
            formatter: *const ZydisFormatter,
            buffer: *mut FormatterBuffer,
            ctx: *mut FormatterContext,
            decorator: Decorator,
        ) -> Status {
            let formatter = &*(formatter as *const Formatter);
            let ctx = &mut *ctx;
            let usr = get_user_data(ctx.user_data);
            match formatter.$field_name.as_ref().unwrap()(
                formatter,
                &mut *buffer,
                ctx,
                decorator,
                usr,
            ) {
                Ok(_) => Status::Success,
                Err(e) => e,
            }
        }
    };
}

wrap_func!(general pre_instruction, dispatch_pre_instruction);
wrap_func!(general post_instruction, dispatch_post_instruction);
wrap_func!(general pre_operand, dispatch_pre_operand);
wrap_func!(general post_operand, dispatch_post_operand);
wrap_func!(general format_instruction, dispatch_format_instruction);
wrap_func!(general format_operand_reg, dispatch_format_operand_reg);
wrap_func!(general format_operand_mem, dispatch_format_operand_mem);
wrap_func!(general format_operand_ptr, dispatch_format_operand_ptr);
wrap_func!(general format_operand_imm, dispatch_format_operand_imm);
wrap_func!(general print_mnemonic, dispatch_print_mnemonic);
wrap_func!(general print_disp, dispatch_print_disp);
wrap_func!(general print_imm, dispatch_print_imm);
wrap_func!(general print_typecast, dispatch_print_typecast);
wrap_func!(general print_prefixes, dispatch_print_prefixes);
wrap_func!(general print_address_abs, dispatch_print_address_abs);
wrap_func!(general print_address_rel, dispatch_print_address_rel);
wrap_func!(register print_register, dispatch_print_register);
wrap_func!(decorator print_decorator, dispatch_print_decorator);

#[derive(Clone, Copy)]
pub enum FormatterProperty<'a> {
    ForceSize(bool),
    ForceSegment(bool),
    ForceRelativeBranches(bool),
    ForceRelativeRiprel(bool),
    PrintBranchSize(bool),
    DetailedPrefixes(bool),
    AddressBase(NumericBase),
    AddressSignedness(Signedness),
    AddressPaddingAbsolute(Padding),
    AddressPaddingRelative(Padding),
    DisplacementBase(NumericBase),
    DisplacementSignedness(Signedness),
    DisplacementPadding(Padding),
    ImmediateBase(NumericBase),
    ImmediateSignedness(Signedness),
    ImmediatePadding(Padding),
    UppercasePrefixes(bool),
    UppercaseMnemonic(bool),
    UppercaseRegisters(bool),
    UppercaseTypecasts(bool),
    UppercaseDecorators(bool),
    DecPrefix(Option<&'a CStr>),
    DecSuffix(Option<&'a CStr>),
    HexUppercase(bool),
    HexPrefix(Option<&'a CStr>),
    HexSuffix(Option<&'a CStr>),
}

fn user_data_to_c_void(x: &mut &mut dyn Any) -> *mut c_void {
    (x as *mut &mut dyn Any) as *mut c_void
}

fn ip_to_runtime_addr(ip: Option<u64>) -> u64 {
    match ip {
        None => (-1i64) as u64,
        Some(ip) => ip,
    }
}

/// A convenience typed when using the `format.*` or `tokenize.*` functions.
#[derive(Debug)]
pub struct OutputBuffer<'a> {
    buffer: &'a mut [u8],
}

impl<'a> OutputBuffer<'a> {
    /// Creates a new `OutputBuffer` using the given `buffer` for storage.
    #[inline]
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer }
    }

    /// Gets a string from this buffer.
    #[inline]
    pub fn as_str(&self) -> Result<&'a str> {
        unsafe { CStr::from_ptr(self.buffer.as_ptr() as _) }
            .to_str()
            .map_err(|_| Status::NotUTF8)
    }
}

impl fmt::Display for OutputBuffer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = self.as_str().map_err(|_| fmt::Error)?;
        write!(f, "{}", str)
    }
}

#[repr(C)]
// needed, since we cast a *const ZydisFormatter to a *const Formatter and the
// rust compiler could reorder the fields if this wasn't #[repr(C)].
pub struct Formatter {
    formatter: ZydisFormatter,

    pre_instruction: Option<Box<WrappedGeneralFunc>>,
    post_instruction: Option<Box<WrappedGeneralFunc>>,
    pre_operand: Option<Box<WrappedGeneralFunc>>,
    post_operand: Option<Box<WrappedGeneralFunc>>,
    format_instruction: Option<Box<WrappedGeneralFunc>>,
    format_operand_reg: Option<Box<WrappedGeneralFunc>>,
    format_operand_mem: Option<Box<WrappedGeneralFunc>>,
    format_operand_ptr: Option<Box<WrappedGeneralFunc>>,
    format_operand_imm: Option<Box<WrappedGeneralFunc>>,
    print_mnemonic: Option<Box<WrappedGeneralFunc>>,
    print_register: Option<Box<WrappedRegisterFunc>>,
    print_address_abs: Option<Box<WrappedGeneralFunc>>,
    print_address_rel: Option<Box<WrappedGeneralFunc>>,
    print_disp: Option<Box<WrappedGeneralFunc>>,
    print_imm: Option<Box<WrappedGeneralFunc>>,
    print_typecast: Option<Box<WrappedGeneralFunc>>,
    print_prefixes: Option<Box<WrappedGeneralFunc>>,
    print_decorator: Option<Box<WrappedDecoratorFunc>>,
}

impl Formatter {
    wrapped_hook_setter!(
        pre_instruction,
        WrappedGeneralFunc,
        set_pre_instruction,
        dispatch_pre_instruction,
        Hook::PreInstruction
    );

    wrapped_hook_setter!(
        post_instruction,
        WrappedGeneralFunc,
        set_post_instruction,
        dispatch_post_instruction,
        Hook::PostInstruction
    );

    wrapped_hook_setter!(
        pre_operand,
        WrappedGeneralFunc,
        set_pre_operand,
        dispatch_pre_operand,
        Hook::PreOperand
    );

    wrapped_hook_setter!(
        post_operand,
        WrappedGeneralFunc,
        set_post_operand,
        dispatch_post_operand,
        Hook::PostOperand
    );

    wrapped_hook_setter!(
        format_instruction,
        WrappedGeneralFunc,
        set_format_instruction,
        dispatch_format_instruction,
        Hook::FormatInstruction
    );

    wrapped_hook_setter!(
        format_operand_reg,
        WrappedGeneralFunc,
        set_format_operand_reg,
        dispatch_format_operand_reg,
        Hook::FormatOperandReg
    );

    wrapped_hook_setter!(
        format_operand_mem,
        WrappedGeneralFunc,
        set_format_operand_mem,
        dispatch_format_operand_mem,
        Hook::FormatOperandMem
    );

    wrapped_hook_setter!(
        format_operand_ptr,
        WrappedGeneralFunc,
        set_format_operand_ptr,
        dispatch_format_operand_ptr,
        Hook::FormatOperandPtr
    );

    wrapped_hook_setter!(
        format_operand_imm,
        WrappedGeneralFunc,
        set_format_operand_imm,
        dispatch_format_operand_imm,
        Hook::FormatOperandImm
    );

    wrapped_hook_setter!(
        print_mnemonic,
        WrappedGeneralFunc,
        set_print_mnemonic,
        dispatch_print_mnemonic,
        Hook::PrintMnemonic
    );

    wrapped_hook_setter!(
        print_register,
        WrappedRegisterFunc,
        set_print_register,
        dispatch_print_register,
        Hook::PrintRegister
    );

    wrapped_hook_setter!(
        print_address_abs,
        WrappedGeneralFunc,
        set_print_address_abs,
        dispatch_print_address_abs,
        Hook::PrintAddressAbs
    );

    wrapped_hook_setter!(
        print_address_rel,
        WrappedGeneralFunc,
        set_print_address_rel,
        dispatch_print_address_rel,
        Hook::PrintAddressRel
    );

    wrapped_hook_setter!(
        print_disp,
        WrappedGeneralFunc,
        set_print_disp,
        dispatch_print_disp,
        Hook::PrintDisp
    );

    wrapped_hook_setter!(
        print_imm,
        WrappedGeneralFunc,
        set_print_imm,
        dispatch_print_imm,
        Hook::PrintImm
    );

    wrapped_hook_setter!(
        print_typecast,
        WrappedGeneralFunc,
        set_print_typecast,
        dispatch_print_typecast,
        Hook::PrintTypecast
    );

    wrapped_hook_setter!(
        print_prefixes,
        WrappedGeneralFunc,
        set_print_prefixes,
        dispatch_print_prefixes,
        Hook::PrintPrefixes
    );

    wrapped_hook_setter!(
        print_decorator,
        WrappedDecoratorFunc,
        set_print_decorator,
        dispatch_print_decorator,
        Hook::PrintDecorator
    );

    /// Creates a new formatter instance.
    pub fn new(style: FormatterStyle) -> Result<Self> {
        unsafe {
            let mut formatter = MaybeUninit::uninit();
            check!(ZydisFormatterInit(formatter.as_mut_ptr(), style as _,), {
                Formatter {
                    formatter: formatter.assume_init(),
                    pre_instruction: None,
                    post_instruction: None,
                    pre_operand: None,
                    post_operand: None,
                    format_instruction: None,
                    format_operand_reg: None,
                    format_operand_mem: None,
                    format_operand_ptr: None,
                    format_operand_imm: None,
                    print_mnemonic: None,
                    print_register: None,
                    print_address_abs: None,
                    print_address_rel: None,
                    print_disp: None,
                    print_imm: None,
                    print_typecast: None,
                    print_prefixes: None,
                    print_decorator: None,
                }
            })
        }
    }

    /// Sets the given FormatterProperty on this formatter instance.
    #[rustfmt::skip]
    pub fn set_property(&mut self, prop: FormatterProperty<'_>) -> Result<()> {
        use FormatterProperty::*;
        let (property, value) = match prop {
            ForceSize(v)              => (ZydisFormatterProperty::FORCE_SIZE              , v as usize),
            ForceSegment(v)           => (ZydisFormatterProperty::FORCE_SEGMENT           , v as usize),
            ForceRelativeBranches(v)  => (ZydisFormatterProperty::FORCE_RELATIVE_BRANCHES , v as usize),
            ForceRelativeRiprel(v)    => (ZydisFormatterProperty::FORCE_RELATIVE_RIPREL   , v as usize),
            PrintBranchSize(v)        => (ZydisFormatterProperty::PRINT_BRANCH_SIZE       , v as usize),
            DetailedPrefixes(v)       => (ZydisFormatterProperty::DETAILED_PREFIXES       , v as usize),
            AddressBase(v)            => (ZydisFormatterProperty::ADDR_BASE               , v as usize),
            AddressSignedness(v)      => (ZydisFormatterProperty::ADDR_SIGNEDNESS         , v as usize),
            AddressPaddingAbsolute(v) => (ZydisFormatterProperty::ADDR_PADDING_ABSOLUTE   , v as usize),
            AddressPaddingRelative(v) => (ZydisFormatterProperty::ADDR_PADDING_RELATIVE   , v as usize),
            DisplacementBase(v)       => (ZydisFormatterProperty::DISP_BASE               , v as usize),
            DisplacementSignedness(v) => (ZydisFormatterProperty::DISP_SIGNEDNESS         , v as usize),
            DisplacementPadding(v)    => (ZydisFormatterProperty::DISP_PADDING            , v as usize),
            ImmediateBase(v)          => (ZydisFormatterProperty::IMM_BASE                , v as usize),
            ImmediateSignedness(v)    => (ZydisFormatterProperty::IMM_SIGNEDNESS          , v as usize),
            ImmediatePadding(v)       => (ZydisFormatterProperty::IMM_PADDING             , v as usize),
            UppercasePrefixes(v)      => (ZydisFormatterProperty::UPPERCASE_PREFIXES      , v as usize),
            UppercaseMnemonic(v)      => (ZydisFormatterProperty::UPPERCASE_MNEMONIC      , v as usize),
            UppercaseRegisters(v)     => (ZydisFormatterProperty::UPPERCASE_REGISTERS     , v as usize),
            UppercaseTypecasts(v)     => (ZydisFormatterProperty::UPPERCASE_TYPECASTS     , v as usize),
            UppercaseDecorators(v)    => (ZydisFormatterProperty::UPPERCASE_DECORATORS    , v as usize),
            DecPrefix(Some(v))        => (ZydisFormatterProperty::DEC_PREFIX              , v.as_ptr() as usize),
            DecPrefix(_)              => (ZydisFormatterProperty::DEC_PREFIX              , 0),
            DecSuffix(Some(v))        => (ZydisFormatterProperty::DEC_SUFFIX              , v.as_ptr() as usize),
            DecSuffix(_)              => (ZydisFormatterProperty::DEC_SUFFIX              , 0),
            HexUppercase(v)           => (ZydisFormatterProperty::HEX_UPPERCASE           , v as usize),
            HexPrefix(Some(v))        => (ZydisFormatterProperty::HEX_PREFIX              , v.as_ptr() as usize),
            HexPrefix(_)              => (ZydisFormatterProperty::HEX_PREFIX              , 0),
            HexSuffix(Some(v))        => (ZydisFormatterProperty::HEX_SUFFIX              , v.as_ptr() as usize),
            HexSuffix(_)              => (ZydisFormatterProperty::HEX_SUFFIX              , 0),
        };

        unsafe {
            check!(ZydisFormatterSetProperty(
                &mut self.formatter,
                property,
                value
            ))
        }
    }

    /// Formats the given `instruction`, using the given `buffer` for storage.
    ///
    /// The `ip` may be `None`, in which case relative address formatting is
    /// used. Otherwise absolute addresses are used.
    ///
    /// `user_data` may contain any data you wish to pass on to the
    /// Formatter hooks.
    ///
    /// # Examples
    /// ```
    /// use zydis::{AddressWidth, Decoder, Formatter, FormatterStyle, MachineMode, OutputBuffer};
    /// static INT3: &'static [u8] = &[0xCC];
    ///
    /// let mut buffer = [0u8; 200];
    /// let mut buffer = OutputBuffer::new(&mut buffer[..]);
    ///
    /// let formatter = Formatter::new(FormatterStyle::INTEL).unwrap();
    /// let dec = Decoder::new(MachineMode::LONG_64, AddressWidth::_64).unwrap();
    ///
    /// let insn = dec.decode(INT3).unwrap().unwrap();
    /// formatter
    ///     .format_instruction(&insn, &mut buffer, Some(0), None)
    ///     .unwrap();
    /// assert_eq!(buffer.as_str().unwrap(), "int3");
    /// ```
    #[inline]
    pub fn format_instruction(
        &self,
        instruction: &DecodedInstruction,
        buffer: &mut OutputBuffer,
        ip: Option<u64>,
        user_data: Option<&mut dyn Any>,
    ) -> Result<()> {
        unsafe {
            check!(match user_data {
                Some(mut x) => ZydisFormatterFormatInstructionEx(
                    &self.formatter,
                    instruction,
                    buffer.buffer.as_mut_ptr() as *mut _,
                    buffer.buffer.len(),
                    ip_to_runtime_addr(ip),
                    user_data_to_c_void(&mut x),
                ),
                None => ZydisFormatterFormatInstructionEx(
                    &self.formatter,
                    instruction,
                    buffer.buffer.as_mut_ptr() as *mut _,
                    buffer.buffer.len(),
                    ip_to_runtime_addr(ip),
                    ptr::null_mut(),
                ),
            })
        }
    }

    /// Formats just the given operand at `operand_index` from the given
    /// `instruction`, using `buffer` for storage.
    ///
    /// The `ip` may be `None`, in which case relative address formatting is
    /// used. Otherwise absolute addresses are used.
    ///
    /// `user_data` may contain any data you wish to pass on to the Formatter
    /// hooks.
    #[inline]
    pub fn format_operand(
        &self,
        instruction: &DecodedInstruction,
        operand_index: u8,
        buffer: &mut OutputBuffer,
        ip: Option<u64>,
        user_data: Option<&mut dyn Any>,
    ) -> Result<()> {
        unsafe {
            check!(match user_data {
                Some(mut x) => ZydisFormatterFormatOperandEx(
                    &self.formatter,
                    instruction,
                    operand_index,
                    buffer.buffer.as_mut_ptr() as *mut _,
                    buffer.buffer.len(),
                    ip_to_runtime_addr(ip),
                    user_data_to_c_void(&mut x),
                ),
                None => ZydisFormatterFormatOperandEx(
                    &self.formatter,
                    instruction,
                    operand_index,
                    buffer.buffer.as_mut_ptr() as *mut _,
                    buffer.buffer.len(),
                    ip_to_runtime_addr(ip),
                    ptr::null_mut(),
                ),
            })
        }
    }

    /// The recommended amount of memory to allocate is 256 bytes.
    #[inline]
    pub fn tokenize_instruction<'a>(
        &self,
        instruction: &DecodedInstruction,
        buffer: &'a mut [u8],
        ip: Option<u64>,
        user_data: Option<&mut dyn Any>,
    ) -> Result<&'a FormatterToken<'a>> {
        unsafe {
            let mut token = MaybeUninit::uninit();
            check!(
                match user_data {
                    Some(mut x) => ZydisFormatterTokenizeInstructionEx(
                        &self.formatter,
                        instruction,
                        buffer.as_mut_ptr() as *mut _,
                        buffer.len(),
                        ip_to_runtime_addr(ip),
                        token.as_mut_ptr(),
                        user_data_to_c_void(&mut x),
                    ),
                    None => ZydisFormatterTokenizeInstructionEx(
                        &self.formatter,
                        instruction,
                        buffer.as_mut_ptr() as *mut _,
                        buffer.len(),
                        ip_to_runtime_addr(ip),
                        token.as_mut_ptr(),
                        ptr::null_mut(),
                    ),
                },
                &*{ token.assume_init() }
            )
        }
    }

    /// Tokenizes the given operand at the `index` of the given `instruction`.
    /// See also the `tokens` example in the `examples` directory.
    ///
    /// # Examples
    /// ```
    /// use zydis::{AddressWidth, Decoder, Formatter, FormatterStyle, MachineMode, TOKEN_REGISTER};
    /// // push rcx
    /// static PUSH: &'static [u8] = &[0x51];
    ///
    /// let dec = Decoder::new(MachineMode::LONG_64, AddressWidth::_64).unwrap();
    /// let formatter = Formatter::new(FormatterStyle::INTEL).unwrap();
    ///
    /// let mut buffer = [0; 256];
    ///
    /// let insn = dec.decode(PUSH).unwrap().unwrap();
    /// let (ty, val) = formatter
    ///     .tokenize_operand(&insn, 0, &mut buffer[..], None, None)
    ///     .unwrap()
    ///     .get_value()
    ///     .unwrap();
    /// assert_eq!(ty, TOKEN_REGISTER);
    /// assert_eq!(val, "rcx");
    /// ```
    #[inline]
    pub fn tokenize_operand<'a>(
        &self,
        instruction: &DecodedInstruction,
        index: u8,
        buffer: &'a mut [u8],
        ip: Option<u64>,
        user_data: Option<&mut dyn Any>,
    ) -> Result<&'a FormatterToken<'a>> {
        unsafe {
            let mut token = MaybeUninit::uninit();
            check!(
                match user_data {
                    Some(mut x) => ZydisFormatterTokenizeOperandEx(
                        &self.formatter,
                        instruction,
                        index,
                        buffer.as_mut_ptr() as *mut _,
                        buffer.len(),
                        ip_to_runtime_addr(ip),
                        token.as_mut_ptr(),
                        user_data_to_c_void(&mut x),
                    ),
                    None => ZydisFormatterTokenizeOperandEx(
                        &self.formatter,
                        instruction,
                        index,
                        buffer.as_mut_ptr() as *mut _,
                        buffer.len(),
                        ip_to_runtime_addr(ip),
                        token.as_mut_ptr(),
                        ptr::null_mut(),
                    ),
                },
                &*{ token.assume_init() }
            )
        }
    }

    /// Sets a raw hook, allowing for customizations along the formatting
    /// process.
    ///
    /// This is the raw C style version of the formatter hook mechanism. No
    /// wrapping occurs, your callback will receive raw pointers. You might want
    /// to consider using any of the wrapped variants instead.
    ///
    /// To use the raw hooks set by this function when formatting, use the
    /// functinos in `zydis::ffi`. For example:
    ///
    /// ```
    /// use zydis::{
    ///     ffi::ZydisFormatterFormatInstructionEx, AddressWidth, Decoder, Formatter, FormatterStyle,
    ///     MachineMode, Status,
    /// };
    /// static INT3: &'static [u8] = &[0xCC];
    ///
    /// let mut buffer = [0u8; 200];
    ///
    /// let formatter = Formatter::new(FormatterStyle::INTEL).unwrap();
    /// let dec = Decoder::new(MachineMode::LONG_64, AddressWidth::_64).unwrap();
    ///
    /// let insn = dec.decode(INT3).unwrap().unwrap();
    /// unsafe {
    ///     let status = ZydisFormatterFormatInstructionEx(
    ///         &formatter as *const Formatter as *const _,
    ///         &insn,
    ///         buffer.as_mut_ptr() as *mut _,
    ///         200,                  // buffer size
    ///         0,                    // runtime address
    ///         std::ptr::null_mut(), // arbitrary user data passed directly to the hooks.
    ///     );
    ///     assert_eq!(status, Status::Success);
    /// }
    /// assert_eq!(&buffer[..4], b"int3");
    /// assert_eq!(buffer[4], 0);
    /// ```
    ///
    /// # Safety
    ///
    /// When using any of the wrapped functions here (i.e. `format_instruction`)
    /// the `user_data` parameters of the hook has the type `*mut &mut Any`.
    #[inline]
    pub unsafe fn set_raw_hook(&mut self, hook: Hook) -> Result<Hook> {
        let mut cb = hook.to_raw();
        let hook_id = hook.to_id();

        check!(
            ZydisFormatterSetHook(&mut self.formatter, hook_id as _, &mut cb),
            Hook::from_raw(hook_id, cb)
        )
    }
}
