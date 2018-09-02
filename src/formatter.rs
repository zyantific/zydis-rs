//! Textual instruction formatting routines.

use std::{
    any::Any,
    ffi::CStr,
    fmt,
    marker::PhantomData,
    mem,
    os::raw::{c_char, c_void},
    ptr,
};

use gen::*;
use status::Result;

#[derive(Clone)]
pub enum Hook {
    PreInstruction(ZydisFormatterFunc),
    PostInstruction(ZydisFormatterFunc),
    PreOperand(ZydisFormatterOperandFunc),
    PostOperand(ZydisFormatterOperandFunc),
    FormatInstruction(ZydisFormatterFunc),
    FormatOperandReg(ZydisFormatterOperandFunc),
    FormatOperandMem(ZydisFormatterOperandFunc),
    FormatOperandPtr(ZydisFormatterOperandFunc),
    FormatOperandImm(ZydisFormatterOperandFunc),
    PrintMnemonic(ZydisFormatterFunc),
    PrintRegister(ZydisFormatterRegisterFunc),
    PrintAddress(ZydisFormatterAddressFunc),
    PrintDisp(ZydisFormatterOperandFunc),
    PrintImm(ZydisFormatterOperandFunc),
    PrintMemsize(ZydisFormatterOperandFunc),
    PrintPrefixes(ZydisFormatterFunc),
    PrintDecorator(ZydisFormatterDecoratorFunc),
}

impl Hook {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn to_id(&self) -> ZydisFormatterHookTypes {
        use self::Hook::*;
        match self {
            PreInstruction(_)    => ZYDIS_FORMATTER_HOOK_PRE_INSTRUCTION,
            PostInstruction(_)   => ZYDIS_FORMATTER_HOOK_POST_INSTRUCTION,
            PreOperand(_)        => ZYDIS_FORMATTER_HOOK_PRE_OPERAND,
            PostOperand(_)       => ZYDIS_FORMATTER_HOOK_POST_OPERAND,
            FormatInstruction(_) => ZYDIS_FORMATTER_HOOK_FORMAT_INSTRUCTION,
            FormatOperandReg(_)  => ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_REG,
            FormatOperandMem(_)  => ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_MEM,
            FormatOperandPtr(_)  => ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_PTR,
            FormatOperandImm(_)  => ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_IMM,
            PrintMnemonic(_)     => ZYDIS_FORMATTER_HOOK_PRINT_MNEMONIC,
            PrintRegister(_)     => ZYDIS_FORMATTER_HOOK_PRINT_REGISTER,
            PrintAddress(_)      => ZYDIS_FORMATTER_HOOK_PRINT_ADDRESS,
            PrintDisp(_)         => ZYDIS_FORMATTER_HOOK_PRINT_DISP,
            PrintImm(_)          => ZYDIS_FORMATTER_HOOK_PRINT_IMM,
            PrintMemsize(_)      => ZYDIS_FORMATTER_HOOK_PRINT_MEMSIZE,
            PrintPrefixes(_)     => ZYDIS_FORMATTER_HOOK_PRINT_PREFIXES,
            PrintDecorator(_)    => ZYDIS_FORMATTER_HOOK_PRINT_DECORATOR,
        }
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub unsafe fn to_raw(&self) -> *const c_void {
        use self::Hook::*;
        // Note: do not remove the `*` at `*self`, Rust 1.26 will segfault
        // since we don't give explicit types for mem::transmute.
        match *self {
            PreInstruction(x) | PostInstruction(x) | PrintPrefixes(x) | FormatInstruction(x)
            | PrintMnemonic(x) =>
                mem::transmute(x),

            PreOperand(x) | PostOperand(x) | FormatOperandReg(x) | FormatOperandMem(x)
            | FormatOperandPtr(x) | FormatOperandImm(x) | PrintDisp(x) | PrintImm(x) =>
                mem::transmute(x),

            PrintRegister(x)  => mem::transmute(x),
            PrintAddress(x)   => mem::transmute(x),
            PrintMemsize(x)   => mem::transmute(x),
            PrintDecorator(x) => mem::transmute(x),
        }
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub unsafe fn from_raw(id: ZydisFormatterHookTypes, cb: *const c_void) -> Hook {
        use self::Hook::*;
        match id {
            ZYDIS_FORMATTER_HOOK_PRE_INSTRUCTION    => PreInstruction(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_POST_INSTRUCTION   => PostInstruction(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRE_OPERAND        => PreOperand(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_POST_OPERAND       => PostOperand(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_FORMAT_INSTRUCTION => FormatInstruction(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_REG => FormatOperandReg(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_MEM => FormatOperandMem(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_PTR => FormatOperandPtr(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_FORMAT_OPERAND_IMM => FormatOperandImm(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_MNEMONIC     => PrintMnemonic(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_REGISTER     => PrintRegister(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_ADDRESS      => PrintAddress(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_DISP         => PrintDisp(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_IMM          => PrintImm(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_MEMSIZE      => PrintMemsize(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_PREFIXES     => PrintPrefixes(mem::transmute(cb)),
            ZYDIS_FORMATTER_HOOK_PRINT_DECORATOR    => PrintDecorator(mem::transmute(cb)),
            _                                       => unreachable!(),
        }
    }
}

impl ZydisString {
    pub fn new(buffer: *mut c_char, capacity: usize) -> Self {
        Self {
            buffer,
            length: 0,
            capacity,
        }
    }

    /// Appends the given string `s` to this buffer.
    ///
    /// Warning: The actual Rust `&str`ings are encoded in UTF-8 and aren't converted to any
    /// other encoding. They're simply copied, byte by byte, to the buffer. Therefore, the
    /// buffer should be interpreted as UTF-8 when later being printed.
    pub fn append<S: AsRef<str> + ?Sized>(&mut self, s: &S) -> Result<()> {
        let bytes = s.as_ref().as_bytes();
        unsafe {
            check!(
                ZydisStringAppendStatic(
                    self,
                    &ZydisStaticString {
                        buffer: bytes.as_ptr() as _,
                        length: bytes.len() as _,
                    },
                    ZYDIS_LETTER_CASE_DEFAULT as _,
                ),
                ()
            )
        }
    }
}

impl fmt::Write for ZydisString {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.append(s).map_err(|_| fmt::Error)
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub type WrappedGeneralFunc = dyn Fn(
    &Formatter,
    &mut ZydisString,
    &ZydisDecodedInstruction,
    Option<&mut dyn Any>
) -> Result<()>;

pub type WrappedOperandFunc = dyn Fn(
    &Formatter,
    &mut ZydisString,
    &ZydisDecodedInstruction,
    &ZydisDecodedOperand,
    Option<&mut dyn Any>,
) -> Result<()>;

pub type WrappedRegisterFunc = dyn Fn(
    &Formatter,
    &mut ZydisString,
    &ZydisDecodedInstruction,
    &ZydisDecodedOperand,
    ZydisRegister,
    Option<&mut dyn Any>,
) -> Result<()>;

pub type WrappedAddressFunc = dyn Fn(
    &Formatter,
    &mut ZydisString,
    &ZydisDecodedInstruction,
    &ZydisDecodedOperand,
    u64,
    Option<&mut dyn Any>,
) -> Result<()>;

pub type WrappedDecoratorFunc = dyn Fn(
    &Formatter,
    &mut ZydisString,
    &ZydisDecodedInstruction,
    &ZydisDecodedOperand,
    ZydisDecoratorType,
    Option<&mut dyn Any>,
) -> Result<()>;

macro_rules! wrapped_hook_setter{
    ($field_name:ident, $field_type:ty, $func_name:ident, $dispatch_func:ident, $constructor:expr)
        => {
        /// Sets the formatter hook to the provided value.
        ///
        /// This function accepts a wrapped version of the raw hook.
        /// It returns the previous set *raw* hook.
        pub fn $func_name(&mut self, new_func: Box<$field_type>) -> Result<Hook> {
            self.$field_name = Some(new_func);
            self.set_raw_hook($constructor(Some($dispatch_func)))
        }
    };
}

macro_rules! get_user_data {
    ($user_data:expr) => {
        if $user_data.is_null() {
            None
        } else {
            Some(*($user_data as *mut &mut dyn Any))
        }
    };
}

macro_rules! wrap_func {
    (general $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name(
            formatter: *const ZydisFormatter,
            string: *mut ZydisString,
            instruction: *const ZydisDecodedInstruction,
            user_data: *mut c_void,
        ) -> ZydisStatus {
            let formatter = &*(formatter as *const Formatter);
            ZydisStatus::from(match formatter.$field_name.as_ref().unwrap()(
                formatter,
                &mut *string,
                &*instruction,
                get_user_data!(user_data),
            ) {
                Ok(_) => ZydisStatusCodes::ZYDIS_STATUS_SUCCESS,
                Err(e) => e.get_code(),
            })
        }
    };
    (operand $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name(
            formatter: *const ZydisFormatter,
            string: *mut ZydisString,
            instruction: *const ZydisDecodedInstruction,
            operand: *const ZydisDecodedOperand,
            user_data: *mut c_void,
        ) -> ZydisStatus {
            let formatter = &*(formatter as *const Formatter);
            ZydisStatus::from(match formatter.$field_name.as_ref().unwrap()(
                formatter,
                &mut *string,
                &*instruction,
                &*operand,
                get_user_data!(user_data),
            ) {
                Ok(_) => ZydisStatusCodes::ZYDIS_STATUS_SUCCESS,
                Err(e) => e.get_code(),
            })
        }
    };
    (register $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name(
            formatter: *const ZydisFormatter,
            string: *mut ZydisString,
            instruction: *const ZydisDecodedInstruction,
            operand: *const ZydisDecodedOperand,
            reg: ZydisRegister,
            user_data: *mut c_void,
        ) -> ZydisStatus {
            let formatter = &*(formatter as *const Formatter);
            ZydisStatus::from(match formatter.$field_name.as_ref().unwrap()(
                formatter,
                &mut *string,
                &*instruction,
                &*operand,
                reg,
                get_user_data!(user_data),
            ) {
                Ok(_) => ZydisStatusCodes::ZYDIS_STATUS_SUCCESS,
                Err(e) => e.get_code(),
            })
        }
    };
    (address $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name(
            formatter: *const ZydisFormatter,
            string: *mut ZydisString,
            instruction: *const ZydisDecodedInstruction,
            operand: *const ZydisDecodedOperand,
            address: u64,
            user_data: *mut c_void,
        ) -> ZydisStatus {
            let formatter = &*(formatter as *const Formatter);
            ZydisStatus::from(match formatter.$field_name.as_ref().unwrap()(
                formatter,
                &mut *string,
                &*instruction,
                &*operand,
                address,
                get_user_data!(user_data),
            ) {
                Ok(_) => ZydisStatusCodes::ZYDIS_STATUS_SUCCESS,
                Err(e) => e.get_code(),
            })
        }
    };
    (decorator $field_name:ident, $func_name:ident) => {
        unsafe extern "C" fn $func_name(
            formatter: *const ZydisFormatter,
            string: *mut ZydisString,
            instruction: *const ZydisDecodedInstruction,
            operand: *const ZydisDecodedOperand,
            decorator: ZydisDecoratorType,
            user_data: *mut c_void,
        ) -> ZydisStatus {
            let formatter = &*(formatter as *const Formatter);
            ZydisStatus::from(match formatter.$field_name.as_ref().unwrap()(
                formatter,
                &mut *string,
                &*instruction,
                &*operand,
                decorator,
                get_user_data!(user_data),
            ) {
                Ok(_) => ZydisStatusCodes::ZYDIS_STATUS_SUCCESS,
                Err(e) => e.get_code(),
            })
        }
    };
}

wrap_func!(general pre_instruction, dispatch_pre_instruction);
wrap_func!(general post_instruction, dispatch_post_instruction);
wrap_func!(operand pre_operand, dispatch_pre_operand);
wrap_func!(operand post_operand, dispatch_post_operand);
wrap_func!(general format_instruction, dispatch_format_instruction);
wrap_func!(operand format_operand_reg, dispatch_format_operand_reg);
wrap_func!(operand format_operand_mem, dispatch_format_operand_mem);
wrap_func!(operand format_operand_ptr, dispatch_format_operand_ptr);
wrap_func!(operand format_operand_imm, dispatch_format_operand_imm);
wrap_func!(general print_mnemonic, dispatch_print_mnemonic);
wrap_func!(register print_register, dispatch_print_register);
wrap_func!(address print_address, dispatch_print_address);
wrap_func!(operand print_disp, dispatch_print_disp);
wrap_func!(operand print_imm, dispatch_print_imm);
wrap_func!(operand print_memsize, dispatch_print_memsize);
wrap_func!(general print_prefixes, dispatch_print_prefixes);
wrap_func!(decorator print_decorator, dispatch_print_decorator);

#[derive(Clone)]
pub enum FormatterProperty<'a> {
    Uppercase(bool),
    ForceMemseg(bool),
    ForceMemsize(bool),
    AddressFormat(ZydisAddressFormat),
    DispFormat(ZydisDisplacementFormat),
    ImmFormat(ZydisImmediateFormat),
    HexUppercase(bool),
    HexPrefix(Option<&'a CStr>),
    HexSuffix(Option<&'a CStr>),
    HexPaddingAddr(u8),
    HexPaddingDisp(u8),
    HexPaddingImm(u8),
}

pub fn user_data_to_c_void(x: &mut &mut dyn Any) -> *mut c_void {
    (x as *mut &mut dyn Any) as *mut c_void
}

#[repr(C)]
// needed, since we cast a *const ZydisFormatter to a *const Formatter and the rust compiler
// could reorder the fields if this wasn't #[repr(C)].
pub struct Formatter<'a> {
    formatter: ZydisFormatter,

    pre_instruction: Option<Box<WrappedGeneralFunc>>,
    post_instruction: Option<Box<WrappedGeneralFunc>>,
    pre_operand: Option<Box<WrappedOperandFunc>>,
    post_operand: Option<Box<WrappedOperandFunc>>,
    format_instruction: Option<Box<WrappedGeneralFunc>>,
    format_operand_reg: Option<Box<WrappedOperandFunc>>,
    format_operand_mem: Option<Box<WrappedOperandFunc>>,
    format_operand_ptr: Option<Box<WrappedOperandFunc>>,
    format_operand_imm: Option<Box<WrappedOperandFunc>>,
    print_mnemonic: Option<Box<WrappedGeneralFunc>>,
    print_register: Option<Box<WrappedRegisterFunc>>,
    print_address: Option<Box<WrappedAddressFunc>>,
    print_disp: Option<Box<WrappedOperandFunc>>,
    print_imm: Option<Box<WrappedOperandFunc>>,
    print_memsize: Option<Box<WrappedOperandFunc>>,
    print_prefixes: Option<Box<WrappedGeneralFunc>>,
    print_decorator: Option<Box<WrappedDecoratorFunc>>,

    _p: PhantomData<&'a ()>,
}

impl<'a> Formatter<'a> {
    /// Creates a new formatter instance.
    pub fn new(style: ZydisFormatterStyles) -> Result<Self> {
        unsafe {
            let mut formatter = mem::uninitialized();
            check!(
                ZydisFormatterInit(&mut formatter, style as _,),
                Formatter {
                    formatter,
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
                    print_address: None,
                    print_disp: None,
                    print_imm: None,
                    print_memsize: None,
                    print_prefixes: None,
                    print_decorator: None,

                    _p: PhantomData,
                }
            )
        }
    }

    /// Sets the given FormatterProperty on this formatter instance.
    pub fn set_property(&mut self, prop: FormatterProperty<'a>) -> Result<()> {
        use FormatterProperty::*;
        let (property, value) = match prop {
            Uppercase(v) => (ZYDIS_FORMATTER_PROP_UPPERCASE, v as usize),
            ForceMemseg(v) => (ZYDIS_FORMATTER_PROP_FORCE_MEMSEG, v as usize),
            ForceMemsize(v) => (ZYDIS_FORMATTER_PROP_FORCE_MEMSIZE, v as usize),
            AddressFormat(v) => (ZYDIS_FORMATTER_PROP_ADDR_FORMAT, v as usize),
            DispFormat(v) => (ZYDIS_FORMATTER_PROP_DISP_FORMAT, v as usize),
            ImmFormat(v) => (ZYDIS_FORMATTER_PROP_IMM_FORMAT, v as usize),
            HexUppercase(v) => (ZYDIS_FORMATTER_PROP_HEX_UPPERCASE, v as usize),
            HexPrefix(Some(v)) => (ZYDIS_FORMATTER_PROP_HEX_PREFIX, v.as_ptr() as usize),
            HexPrefix(_) => (ZYDIS_FORMATTER_PROP_HEX_PREFIX, 0),
            HexSuffix(Some(v)) => (ZYDIS_FORMATTER_PROP_HEX_SUFFIX, v.as_ptr() as usize),
            HexSuffix(_) => (ZYDIS_FORMATTER_PROP_HEX_SUFFIX, 0),
            HexPaddingAddr(v) => (ZYDIS_FORMATTER_PROP_HEX_PADDING_ADDR, v as usize),
            HexPaddingDisp(v) => (ZYDIS_FORMATTER_PROP_HEX_PADDING_DISP, v as usize),
            HexPaddingImm(v) => (ZYDIS_FORMATTER_PROP_HEX_PADDING_IMM, v as usize),
        };

        unsafe {
            check!(
                ZydisFormatterSetProperty(&mut self.formatter, property as _, value),
                ()
            )
        }
    }

    /// Formats the given instruction, returning a string. `size` is the size
    /// allocated (in bytes) for the string that holds the result.
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
    /// let info = dec.decode(INT3, 0).unwrap().unwrap();
    /// let fmt = formatter.format_instruction(&info, 200, None).unwrap();
    /// assert_eq!(fmt, "int3");
    /// ```
    pub fn format_instruction(
        &self,
        instruction: &ZydisDecodedInstruction,
        size: usize,
        user_data: Option<&mut dyn Any>,
    ) -> Result<String> {
        let mut buffer = vec![0u8; size];
        self.format_instruction_raw(instruction, &mut buffer, user_data)
            .map(|_| {
                unsafe { CStr::from_ptr(buffer.as_ptr() as _) }
                    .to_string_lossy()
                    .into()
            })
    }

    /// Formats the given `instruction`, using the given `buffer` for the
    /// result.
    ///
    /// `user_data` may contain any data you wish to pass on to the
    /// Formatter hooks.
    pub fn format_instruction_raw(
        &self,
        instruction: &ZydisDecodedInstruction,
        buffer: &mut [u8],
        user_data: Option<&mut dyn Any>,
    ) -> Result<()> {
        unsafe {
            check!(
                ZydisFormatterFormatInstructionEx(
                    &self.formatter,
                    instruction,
                    buffer.as_ptr() as _,
                    buffer.len(),
                    match user_data {
                        None => ptr::null_mut(),
                        Some(mut x) => user_data_to_c_void(&mut x),
                    }
                ),
                ()
            )
        }
    }

    /// Sets a raw hook, allowing for customizations along the formatting process.
    ///
    /// This is the raw C style version of the formatter hook mechanism. No
    /// wrapping occurs, your callback will receive raw pointers. You might want
    /// to consider using any of the wrapped variants instead.
    ///
    /// Be careful when accessing the `user_data` parameter in the raw hooks.
    /// It's type is `*mut &mut Any`.
    pub fn set_raw_hook(&mut self, hook: Hook) -> Result<Hook> {
        unsafe {
            let mut cb = hook.to_raw();
            let hook_id = hook.to_id();

            check!(
                ZydisFormatterSetHook(&mut self.formatter, hook_id as _, &mut cb),
                Hook::from_raw(hook_id, cb)
            )
        }
    }

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
        WrappedOperandFunc,
        set_pre_operand,
        dispatch_pre_operand,
        Hook::PreOperand
    );
    wrapped_hook_setter!(
        post_operand,
        WrappedOperandFunc,
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
        WrappedOperandFunc,
        set_format_operand_reg,
        dispatch_format_operand_reg,
        Hook::FormatOperandReg
    );
    wrapped_hook_setter!(
        format_operand_mem,
        WrappedOperandFunc,
        set_format_operand_mem,
        dispatch_format_operand_mem,
        Hook::FormatOperandMem
    );
    wrapped_hook_setter!(
        format_operand_ptr,
        WrappedOperandFunc,
        set_format_operand_ptr,
        dispatch_format_operand_ptr,
        Hook::FormatOperandPtr
    );
    wrapped_hook_setter!(
        format_operand_imm,
        WrappedOperandFunc,
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
        print_address,
        WrappedAddressFunc,
        set_print_address,
        dispatch_print_address,
        Hook::PrintAddress
    );
    wrapped_hook_setter!(
        print_disp,
        WrappedOperandFunc,
        set_print_disp,
        dispatch_print_disp,
        Hook::PrintDisp
    );
    wrapped_hook_setter!(
        print_imm,
        WrappedOperandFunc,
        set_print_imm,
        dispatch_print_imm,
        Hook::PrintImm
    );
    wrapped_hook_setter!(
        print_memsize,
        WrappedOperandFunc,
        set_print_memsize,
        dispatch_print_memsize,
        Hook::PrintMemsize
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
}
