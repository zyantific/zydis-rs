//! Textual instruction formatting routines.

use alloc::{borrow::ToOwned, boxed::Box, string::String};
use core::{
    ffi::{c_void, CStr},
    fmt,
    mem::{self, MaybeUninit},
    ptr,
};

use crate::decoder::{Instruction, OperandArrayVec};

use super::{
    enums::*,
    ffi,
    status::{Result, Status},
};

#[derive(Clone)]
pub enum Hook {
    PreInstruction(ffi::FormatterFunc),
    PostInstruction(ffi::FormatterFunc),
    PreOperand(ffi::FormatterFunc),
    PostOperand(ffi::FormatterFunc),
    FormatInstruction(ffi::FormatterFunc),
    FormatOperandReg(ffi::FormatterFunc),
    FormatOperandMem(ffi::FormatterFunc),
    FormatOperandPtr(ffi::FormatterFunc),
    FormatOperandImm(ffi::FormatterFunc),
    PrintMnemonic(ffi::FormatterFunc),
    PrintRegister(ffi::FormatterRegisterFunc),
    PrintAddressAbs(ffi::FormatterFunc),
    PrintAddressRel(ffi::FormatterFunc),
    PrintDisp(ffi::FormatterFunc),
    PrintImm(ffi::FormatterFunc),
    PrintTypecast(ffi::FormatterFunc),
    PrintSegment(ffi::FormatterFunc),
    PrintPrefixes(ffi::FormatterFunc),
    PrintDecorator(ffi::FormatterDecoratorFunc),
}

impl Hook {
    pub fn to_id(&self) -> FormatterFunction {
        use self::Hook::*;
        match self {
            PreInstruction(_) => FormatterFunction::PRE_INSTRUCTION,
            PostInstruction(_) => FormatterFunction::POST_INSTRUCTION,
            PreOperand(_) => FormatterFunction::PRE_OPERAND,
            PostOperand(_) => FormatterFunction::POST_OPERAND,
            FormatInstruction(_) => FormatterFunction::FORMAT_INSTRUCTION,
            FormatOperandReg(_) => FormatterFunction::FORMAT_OPERAND_REG,
            FormatOperandMem(_) => FormatterFunction::FORMAT_OPERAND_MEM,
            FormatOperandPtr(_) => FormatterFunction::FORMAT_OPERAND_PTR,
            FormatOperandImm(_) => FormatterFunction::FORMAT_OPERAND_IMM,
            PrintMnemonic(_) => FormatterFunction::PRINT_MNEMONIC,
            PrintRegister(_) => FormatterFunction::PRINT_REGISTER,
            PrintAddressAbs(_) => FormatterFunction::PRINT_ADDRESS_ABS,
            PrintAddressRel(_) => FormatterFunction::PRINT_ADDRESS_REL,
            PrintDisp(_) => FormatterFunction::PRINT_DISP,
            PrintImm(_) => FormatterFunction::PRINT_IMM,
            PrintTypecast(_) => FormatterFunction::PRINT_TYPECAST,
            PrintSegment(_) => FormatterFunction::PRINT_SEGMENT,
            PrintPrefixes(_) => FormatterFunction::PRINT_PREFIXES,
            PrintDecorator(_) => FormatterFunction::PRINT_DECORATOR,
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
            | PrintTypecast(x) | PrintSegment(x) => x as *const c_void,

            PrintRegister(x) => x as *const c_void,
            PrintDecorator(x) => x as *const c_void,
        }
    }

    pub unsafe fn from_raw(id: FormatterFunction, cb: *const c_void) -> Hook {
        use self::Hook::*;
        match id {
            FormatterFunction::PRE_INSTRUCTION => PreInstruction(mem::transmute(cb)),
            FormatterFunction::POST_INSTRUCTION => PostInstruction(mem::transmute(cb)),
            FormatterFunction::FORMAT_INSTRUCTION => FormatInstruction(mem::transmute(cb)),
            FormatterFunction::PRE_OPERAND => PreOperand(mem::transmute(cb)),
            FormatterFunction::POST_OPERAND => PostOperand(mem::transmute(cb)),
            FormatterFunction::FORMAT_OPERAND_REG => FormatOperandReg(mem::transmute(cb)),
            FormatterFunction::FORMAT_OPERAND_MEM => FormatOperandMem(mem::transmute(cb)),
            FormatterFunction::FORMAT_OPERAND_PTR => FormatOperandPtr(mem::transmute(cb)),
            FormatterFunction::FORMAT_OPERAND_IMM => FormatOperandImm(mem::transmute(cb)),
            FormatterFunction::PRINT_MNEMONIC => PrintMnemonic(mem::transmute(cb)),
            FormatterFunction::PRINT_REGISTER => PrintRegister(mem::transmute(cb)),
            FormatterFunction::PRINT_ADDRESS_ABS => PrintAddressAbs(mem::transmute(cb)),
            FormatterFunction::PRINT_ADDRESS_REL => PrintAddressRel(mem::transmute(cb)),
            FormatterFunction::PRINT_DISP => PrintDisp(mem::transmute(cb)),
            FormatterFunction::PRINT_IMM => PrintImm(mem::transmute(cb)),
            FormatterFunction::PRINT_TYPECAST => PrintTypecast(mem::transmute(cb)),
            FormatterFunction::PRINT_SEGMENT => PrintSegment(mem::transmute(cb)),
            FormatterFunction::PRINT_PREFIXES => PrintPrefixes(mem::transmute(cb)),
            FormatterFunction::PRINT_DECORATOR => PrintDecorator(mem::transmute(cb)),
        }
    }
}

pub type WrappedGeneralFunc<UserData> = dyn Fn(
    &Formatter<UserData>,
    &mut ffi::FormatterBuffer,
    &mut ffi::FormatterContext,
    Option<&mut UserData>,
) -> Result<()>;

pub type WrappedRegisterFunc<UserData> = dyn Fn(
    &Formatter<UserData>,
    &mut ffi::FormatterBuffer,
    &mut ffi::FormatterContext,
    Register,
    Option<&mut UserData>,
) -> Result<()>;

pub type WrappedDecoratorFunc<UserData> = dyn Fn(
    &Formatter<UserData>,
    &mut ffi::FormatterBuffer,
    &mut ffi::FormatterContext,
    Decorator,
    Option<&mut UserData>,
) -> Result<()>;

macro_rules! wrapped_hook_setter {
    ($field_name:ident, $field_type:ty, $func_name:ident, $dispatch_func:path, $constructor:expr) => {
        /// Sets the formatter hook to the provided value.
        ///
        /// This function accepts a wrapped version of the raw hook.
        /// It returns the previous set *raw* hook.
        #[inline]
        pub fn $func_name(&mut self, new_func: Box<$field_type>) -> Result<Hook> {
            self.$field_name = Some(new_func);
            unsafe { self.set_raw_hook($constructor($dispatch_func)) }
        }
    };
}

unsafe fn get_user_data<'a, UserData>(user_data: *mut c_void) -> Option<&'a mut UserData> {
    if user_data.is_null() {
        None
    } else {
        Some(&mut *(user_data as *mut UserData))
    }
}

macro_rules! wrap_func {
    (general $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name<UserData>(
            formatter: *const ffi::Formatter,
            buffer: *mut ffi::FormatterBuffer,
            ctx: *mut ffi::FormatterContext,
        ) -> Status {
            let formatter = &*(formatter as *const Formatter<UserData>);
            match formatter.$field_name.as_ref().unwrap()(
                formatter,
                &mut *buffer,
                &mut *ctx,
                get_user_data((*ctx).user_data),
            ) {
                Ok(_) => Status::Success,
                Err(e) => e,
            }
        }
    };
    (register $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name<UserData>(
            formatter: *const ffi::Formatter,
            buffer: *mut ffi::FormatterBuffer,
            ctx: *mut ffi::FormatterContext,
            reg: Register,
        ) -> Status {
            let formatter = &*(formatter as *const Formatter<UserData>);
            match formatter.$field_name.as_ref().unwrap()(
                formatter,
                &mut *buffer,
                &mut *ctx,
                reg,
                get_user_data((*ctx).user_data),
            ) {
                Ok(_) => Status::Success,
                Err(e) => e,
            }
        }
    };
    (decorator $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name<UserData>(
            formatter: *const ffi::Formatter,
            buffer: *mut ffi::FormatterBuffer,
            ctx: *mut ffi::FormatterContext,
            decorator: Decorator,
        ) -> Status {
            let formatter = &*(formatter as *const Formatter<UserData>);
            match formatter.$field_name.as_ref().unwrap()(
                formatter,
                &mut *buffer,
                &mut *ctx,
                decorator,
                get_user_data((*ctx).user_data),
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

/// State of a formatter setting knob.
#[derive(Clone, Copy)]
pub enum FormatterProperty<'a> {
    ForceSize(bool),
    ForceSegment(bool),
    ForceScaleOne(bool),
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
    HexForceLeadingNumber(bool),
    HexPrefix(Option<&'a CStr>),
    HexSuffix(Option<&'a CStr>),
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

/// Formats decoded instructions to human-readable text.
#[repr(C)]
// needed, since we cast a *const ZydisFormatter to a *const Formatter and the
// rust compiler could reorder the fields if this wasn't #[repr(C)].
pub struct Formatter<UserData = ()> {
    formatter: ffi::Formatter,

    pre_instruction: Option<Box<WrappedGeneralFunc<UserData>>>,
    post_instruction: Option<Box<WrappedGeneralFunc<UserData>>>,
    pre_operand: Option<Box<WrappedGeneralFunc<UserData>>>,
    post_operand: Option<Box<WrappedGeneralFunc<UserData>>>,
    format_instruction: Option<Box<WrappedGeneralFunc<UserData>>>,
    format_operand_reg: Option<Box<WrappedGeneralFunc<UserData>>>,
    format_operand_mem: Option<Box<WrappedGeneralFunc<UserData>>>,
    format_operand_ptr: Option<Box<WrappedGeneralFunc<UserData>>>,
    format_operand_imm: Option<Box<WrappedGeneralFunc<UserData>>>,
    print_mnemonic: Option<Box<WrappedGeneralFunc<UserData>>>,
    print_register: Option<Box<WrappedRegisterFunc<UserData>>>,
    print_address_abs: Option<Box<WrappedGeneralFunc<UserData>>>,
    print_address_rel: Option<Box<WrappedGeneralFunc<UserData>>>,
    print_disp: Option<Box<WrappedGeneralFunc<UserData>>>,
    print_imm: Option<Box<WrappedGeneralFunc<UserData>>>,
    print_typecast: Option<Box<WrappedGeneralFunc<UserData>>>,
    print_prefixes: Option<Box<WrappedGeneralFunc<UserData>>>,
    print_decorator: Option<Box<WrappedDecoratorFunc<UserData>>>,
}

impl Formatter<()> {
    /// Creates a new formatter instance (no user-data).
    pub fn new(style: FormatterStyle) -> Self {
        Formatter::<()>::new_custom_userdata(style)
    }

    /// Creates a new formatter for Intel syntax.
    ///
    /// Convenience wrapper for `Self::new(FormatterStyle::INTEL)`.
    pub fn intel() -> Self {
        Self::new(FormatterStyle::INTEL)
    }

    /// Creates a new formatter for AT&T syntax.
    ///
    /// Convenience wrapper for `Self::new(FormatterStyle::ATT)`.
    pub fn att() -> Self {
        Self::new(FormatterStyle::ATT)
    }
}

impl<UserData> Formatter<UserData> {
    wrapped_hook_setter!(
        pre_instruction,
        WrappedGeneralFunc<UserData>,
        set_pre_instruction,
        dispatch_pre_instruction<UserData>,
        Hook::PreInstruction
    );

    wrapped_hook_setter!(
        post_instruction,
        WrappedGeneralFunc<UserData>,
        set_post_instruction,
        dispatch_post_instruction<UserData>,
        Hook::PostInstruction
    );

    wrapped_hook_setter!(
        pre_operand,
        WrappedGeneralFunc<UserData>,
        set_pre_operand,
        dispatch_pre_operand<UserData>,
        Hook::PreOperand
    );

    wrapped_hook_setter!(
        post_operand,
        WrappedGeneralFunc<UserData>,
        set_post_operand,
        dispatch_post_operand<UserData>,
        Hook::PostOperand
    );

    wrapped_hook_setter!(
        format_instruction,
        WrappedGeneralFunc<UserData>,
        set_format_instruction,
        dispatch_format_instruction<UserData>,
        Hook::FormatInstruction
    );

    wrapped_hook_setter!(
        format_operand_reg,
        WrappedGeneralFunc<UserData>,
        set_format_operand_reg,
        dispatch_format_operand_reg<UserData>,
        Hook::FormatOperandReg
    );

    wrapped_hook_setter!(
        format_operand_mem,
        WrappedGeneralFunc<UserData>,
        set_format_operand_mem,
        dispatch_format_operand_mem<UserData>,
        Hook::FormatOperandMem
    );

    wrapped_hook_setter!(
        format_operand_ptr,
        WrappedGeneralFunc<UserData>,
        set_format_operand_ptr,
        dispatch_format_operand_ptr<UserData>,
        Hook::FormatOperandPtr
    );

    wrapped_hook_setter!(
        format_operand_imm,
        WrappedGeneralFunc<UserData>,
        set_format_operand_imm,
        dispatch_format_operand_imm<UserData>,
        Hook::FormatOperandImm
    );

    wrapped_hook_setter!(
        print_mnemonic,
        WrappedGeneralFunc<UserData>,
        set_print_mnemonic,
        dispatch_print_mnemonic<UserData>,
        Hook::PrintMnemonic
    );

    wrapped_hook_setter!(
        print_register,
        WrappedRegisterFunc<UserData>,
        set_print_register,
        dispatch_print_register<UserData>,
        Hook::PrintRegister
    );

    wrapped_hook_setter!(
        print_address_abs,
        WrappedGeneralFunc<UserData>,
        set_print_address_abs,
        dispatch_print_address_abs<UserData>,
        Hook::PrintAddressAbs
    );

    wrapped_hook_setter!(
        print_address_rel,
        WrappedGeneralFunc<UserData>,
        set_print_address_rel,
        dispatch_print_address_rel<UserData>,
        Hook::PrintAddressRel
    );

    wrapped_hook_setter!(
        print_disp,
        WrappedGeneralFunc<UserData>,
        set_print_disp,
        dispatch_print_disp<UserData>,
        Hook::PrintDisp
    );

    wrapped_hook_setter!(
        print_imm,
        WrappedGeneralFunc<UserData>,
        set_print_imm,
        dispatch_print_imm<UserData>,
        Hook::PrintImm
    );

    wrapped_hook_setter!(
        print_typecast,
        WrappedGeneralFunc<UserData>,
        set_print_typecast,
        dispatch_print_typecast<UserData>,
        Hook::PrintTypecast
    );

    wrapped_hook_setter!(
        print_prefixes,
        WrappedGeneralFunc<UserData>,
        set_print_prefixes,
        dispatch_print_prefixes<UserData>,
        Hook::PrintPrefixes
    );

    wrapped_hook_setter!(
        print_decorator,
        WrappedDecoratorFunc<UserData>,
        set_print_decorator,
        dispatch_print_decorator<UserData>,
        Hook::PrintDecorator
    );

    pub fn raw(&self) -> &ffi::Formatter {
        &self.formatter
    }

    /// Creates a new formatter instance.
    pub fn new_custom_userdata(style: FormatterStyle) -> Self {
        unsafe {
            let mut formatter = MaybeUninit::uninit();

            ffi::ZydisFormatterInit(formatter.as_mut_ptr(), style as _)
                .as_result()
                .expect("init call with valid style cannot fail");

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
        }
    }

    /// Sets the given FormatterProperty on this formatter instance.
    pub fn set_property<'this, 'b: 'this>(
        &'this mut self,
        prop: FormatterProperty<'b>,
    ) -> Result<()> {
        use FormatterProperty::*;
        let (property, value) = match prop {
            ForceSize(v) => (ZydisFormatterProperty::FORCE_SIZE, v as usize),
            ForceSegment(v) => (ZydisFormatterProperty::FORCE_SEGMENT, v as usize),
            ForceScaleOne(v) => (ZydisFormatterProperty::FORCE_SCALE_ONE, v as usize),
            ForceRelativeBranches(v) => {
                (ZydisFormatterProperty::FORCE_RELATIVE_BRANCHES, v as usize)
            }
            ForceRelativeRiprel(v) => (ZydisFormatterProperty::FORCE_RELATIVE_RIPREL, v as usize),
            PrintBranchSize(v) => (ZydisFormatterProperty::PRINT_BRANCH_SIZE, v as usize),
            DetailedPrefixes(v) => (ZydisFormatterProperty::DETAILED_PREFIXES, v as usize),
            AddressBase(v) => (ZydisFormatterProperty::ADDR_BASE, v as usize),
            AddressSignedness(v) => (ZydisFormatterProperty::ADDR_SIGNEDNESS, v as usize),
            AddressPaddingAbsolute(v) => {
                (ZydisFormatterProperty::ADDR_PADDING_ABSOLUTE, v as usize)
            }
            AddressPaddingRelative(v) => {
                (ZydisFormatterProperty::ADDR_PADDING_RELATIVE, v as usize)
            }
            DisplacementBase(v) => (ZydisFormatterProperty::DISP_BASE, v as usize),
            DisplacementSignedness(v) => (ZydisFormatterProperty::DISP_SIGNEDNESS, v as usize),
            DisplacementPadding(v) => (ZydisFormatterProperty::DISP_PADDING, v as usize),
            ImmediateBase(v) => (ZydisFormatterProperty::IMM_BASE, v as usize),
            ImmediateSignedness(v) => (ZydisFormatterProperty::IMM_SIGNEDNESS, v as usize),
            ImmediatePadding(v) => (ZydisFormatterProperty::IMM_PADDING, v as usize),
            UppercasePrefixes(v) => (ZydisFormatterProperty::UPPERCASE_PREFIXES, v as usize),
            UppercaseMnemonic(v) => (ZydisFormatterProperty::UPPERCASE_MNEMONIC, v as usize),
            UppercaseRegisters(v) => (ZydisFormatterProperty::UPPERCASE_REGISTERS, v as usize),
            UppercaseTypecasts(v) => (ZydisFormatterProperty::UPPERCASE_TYPECASTS, v as usize),
            UppercaseDecorators(v) => (ZydisFormatterProperty::UPPERCASE_DECORATORS, v as usize),
            DecPrefix(Some(v)) => (ZydisFormatterProperty::DEC_PREFIX, v.as_ptr() as usize),
            DecPrefix(_) => (ZydisFormatterProperty::DEC_PREFIX, 0),
            DecSuffix(Some(v)) => (ZydisFormatterProperty::DEC_SUFFIX, v.as_ptr() as usize),
            DecSuffix(_) => (ZydisFormatterProperty::DEC_SUFFIX, 0),
            HexUppercase(v) => (ZydisFormatterProperty::HEX_UPPERCASE, v as usize),
            HexForceLeadingNumber(v) => {
                (ZydisFormatterProperty::HEX_FORCE_LEADING_NUMBER, v as usize)
            }
            HexPrefix(Some(v)) => (ZydisFormatterProperty::HEX_PREFIX, v.as_ptr() as usize),
            HexPrefix(_) => (ZydisFormatterProperty::HEX_PREFIX, 0),
            HexSuffix(Some(v)) => (ZydisFormatterProperty::HEX_SUFFIX, v.as_ptr() as usize),
            HexSuffix(_) => (ZydisFormatterProperty::HEX_SUFFIX, 0),
        };

        unsafe { ffi::ZydisFormatterSetProperty(&mut self.formatter, property, value).into() }
    }

    /// Format an instruction as a [`String`].
    ///  
    /// The `ip` may be `None`, in which case relative address formatting is
    /// used. Otherwise absolute addresses are used.
    pub fn format<const N: usize>(
        &self,
        ip: Option<u64>,
        insn: &Instruction<OperandArrayVec<N>>,
    ) -> Result<String> {
        let mut buffer = [0u8; 256];
        let mut buffer = OutputBuffer::new(&mut buffer);
        self.format_ex(ip, insn, &mut buffer, None)?;
        Ok(buffer.as_str()?.to_owned())
    }

    /// Format an instruction and append it to a [`fmt::Formatter`].
    ///
    /// The `ip` may be `None`, in which case relative address formatting is
    /// used. Otherwise absolute addresses are used.
    pub fn format_into<const N: usize>(
        &self,
        ip: Option<u64>,
        insn: &Instruction<OperandArrayVec<N>>,
        f: &mut fmt::Formatter<'_>,
    ) -> Result {
        let mut buffer = [0u8; 256];
        let mut buffer = OutputBuffer::new(&mut buffer);
        self.format_ex(ip, insn, &mut buffer, None)?;
        f.write_str(buffer.as_str()?)
            .map_err(|_| Status::FormatterError)
    }

    /// Format an instruction into an [`OutputBuffer`].
    ///
    /// The `ip` may be `None`, in which case relative address formatting is
    /// used. Otherwise absolute addresses are used.
    ///
    /// This variant is "rawer" than the other format function in that it
    /// accepts the raw FFI structs. It further allows the user to pass a
    /// custom `user_data` argument` that is passed to the formatter hooks.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zydis::*;
    /// static INT3: &'static [u8] = &[0xCC];
    ///
    /// let mut buffer = [0u8; 256];
    /// let mut buffer = OutputBuffer::new(&mut buffer[..]);
    ///
    /// let formatter = Formatter::intel();
    /// let dec = Decoder::new64();
    ///
    /// let insn = dec.decode_first::<VisibleOperands>(INT3).unwrap().unwrap();
    ///
    /// formatter
    ///     .format_ex(Some(0), &insn, &mut buffer, None)
    ///     .unwrap();
    ///
    /// assert_eq!(buffer.as_str().unwrap(), "int3");
    /// ```
    #[inline]
    pub fn format_ex<const N: usize>(
        &self,
        ip: Option<u64>,
        insn: &Instruction<OperandArrayVec<N>>,
        buffer: &mut OutputBuffer,
        user_data: Option<&mut UserData>,
    ) -> Result<()> {
        unsafe {
            ffi::ZydisFormatterFormatInstruction(
                &self.formatter,
                &**insn,
                insn.operands().as_ptr(),
                insn.operands().len() as u8,
                buffer.buffer.as_mut_ptr() as *mut _,
                buffer.buffer.len(),
                ip_to_runtime_addr(ip),
                user_data
                    .map(|x| x as *mut UserData as *mut c_void)
                    .unwrap_or(ptr::null_mut()),
            )
            .into()
        }
    }

    /// Formats just the given operand at `operand_index` from the given
    /// `instruction`, using `buffer` for storage.
    ///
    /// The `ip` may be `None`, in which case relative address formatting is
    /// used. Otherwise absolute addresses are used.
    ///
    /// `user_data` may contain any data to pass on to the formatter hooks.
    ///
    /// # Panics
    ///
    /// If `operand_index` is out of bounds.
    #[inline]
    pub fn format_operand<const N: usize>(
        &self,
        ip: Option<u64>,
        insn: &Instruction<OperandArrayVec<N>>,
        buffer: &mut OutputBuffer,
        operand_index: usize,
        user_data: Option<&mut UserData>,
    ) -> Result<()> {
        unsafe {
            ffi::ZydisFormatterFormatOperand(
                &self.formatter,
                &**insn,
                &insn.operands()[operand_index],
                buffer.buffer.as_mut_ptr() as *mut _,
                buffer.buffer.len(),
                ip_to_runtime_addr(ip),
                user_data
                    .map(|x| x as *mut UserData as *mut c_void)
                    .unwrap_or(ptr::null_mut()),
            )
            .into()
        }
    }

    /// Tokenize the given instruction.
    ///
    /// The recommended amount of memory to allocate is 256 bytes.
    #[inline]
    pub fn tokenize<'buffer, const N: usize>(
        &self,
        ip: Option<u64>,
        insn: &Instruction<OperandArrayVec<N>>,
        buffer: &'buffer mut [u8],
        user_data: Option<&mut UserData>,
    ) -> Result<&'buffer ffi::FormatterToken<'buffer>> {
        unsafe {
            let mut token = MaybeUninit::uninit();
            ffi::ZydisFormatterTokenizeInstruction(
                &self.formatter,
                &**insn,
                insn.operands().as_ptr(),
                insn.operands().len() as u8,
                buffer.as_mut_ptr() as *mut _,
                buffer.len(),
                ip_to_runtime_addr(ip),
                token.as_mut_ptr(),
                user_data
                    .map(|x| x as *mut UserData as *mut c_void)
                    .unwrap_or(ptr::null_mut()),
            )
            .as_result()?;
            Ok(&*{ token.assume_init() })
        }
    }

    /// Tokenizes the given operand at `operand_index`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zydis::*;
    /// static PUSH: &'static [u8] = &[0x51]; // push rcx
    ///
    /// let dec = Decoder::new64();
    /// let formatter = Formatter::intel();
    ///
    /// let mut buffer = [0; 256];
    /// let insn = dec.decode_first::<VisibleOperands>(PUSH).unwrap().unwrap();
    /// let (ty, val) = formatter
    ///     .tokenize_operand(None, &insn, &mut buffer[..], 0, None)
    ///     .unwrap()
    ///     .value()
    ///     .unwrap();
    ///
    /// assert_eq!(ty, TOKEN_REGISTER);
    /// assert_eq!(val, "rcx");
    /// ```
    ///
    /// # Panics
    ///
    /// If `operand_index` is out of bounds.
    #[inline]
    pub fn tokenize_operand<'buffer, const N: usize>(
        &self,
        ip: Option<u64>,
        insn: &Instruction<OperandArrayVec<N>>,
        buffer: &'buffer mut [u8],
        operand_index: usize,
        user_data: Option<&mut UserData>,
    ) -> Result<&'buffer ffi::FormatterToken<'buffer>> {
        unsafe {
            let mut token = MaybeUninit::uninit();
            ffi::ZydisFormatterTokenizeOperand(
                &self.formatter,
                &**insn,
                &insn.operands()[operand_index],
                buffer.as_mut_ptr() as *mut _,
                buffer.len(),
                ip_to_runtime_addr(ip),
                token.as_mut_ptr(),
                user_data
                    .map(|x| x as *mut UserData as *mut c_void)
                    .unwrap_or(ptr::null_mut()),
            )
            .as_result()?;
            Ok(&*{ token.assume_init() })
        }
    }

    /// Sets a raw hook, allowing for customizations along the formatting
    /// process.
    ///
    /// This is the raw C style version of the formatter hook mechanism. No
    /// wrapping occurs, your callback will receive raw pointers. You might want
    /// to consider using any of the wrapped variants instead.
    #[inline]
    pub unsafe fn set_raw_hook(&mut self, hook: Hook) -> Result<Hook> {
        let mut cb = hook.to_raw();
        let hook_id = hook.to_id();
        ffi::ZydisFormatterSetHook(&mut self.formatter, hook_id as _, &mut cb).as_result()?;
        Ok(Hook::from_raw(hook_id, cb))
    }
}
