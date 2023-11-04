use crate::{ffi, *};
use core::{
    mem::{self, ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
};

/// Workaround for missing `const fn` in `core::mem::zeroed`.
///
/// Concept borrowed from `const_zero` crate.
macro_rules! zeroed {
    ($ty:ty) => {{
        union TypeOrArray {
            raw: [u8; mem::size_of::<$ty>()],
            struc: mem::ManuallyDrop<$ty>,
        }
        ManuallyDrop::<$ty>::into_inner(
            TypeOrArray {
                raw: [0; mem::size_of::<$ty>()],
            }
            .struc,
        )
    }};
}

/// Describes an instruction to be encoded.
///
/// ## Encoding new instructions from scratch  
///
/// ### Using the [`insn32`] / [`insn64`] macros
///
/// ```rust
/// # use zydis::*;
/// # fn calc_some_disp() -> i64 { 123 }
/// # fn calc_some_imm() -> i64 { 321 }
/// let int3 = insn64!(INT3).encode();
/// assert_eq!(int3.unwrap(), b"\xCC");
///
/// let mov = insn32!(MOV EDX, 0x1234).encode();
/// assert_eq!(mov.unwrap(), b"\xBA\x34\x12\x00\x00");
///
/// let dyn_imm: i64 = calc_some_imm();
/// insn64!(MOV qword ptr [RAX + (calc_some_disp() + 99)], (dyn_imm))
///     .encode()
///     .unwrap();
/// ```
///
/// Please refer to the documentation on [`insn64`] for more details.
///
/// ### Manually populating a request
///
/// ```rust
/// # use zydis::*;
/// // int3
/// let int3 = EncoderRequest::new64(Mnemonic::INT3).encode();
/// assert_eq!(int3.unwrap(), b"\xCC");
///
/// // mov edx, 0x1234
/// let mov = EncoderRequest::new32(Mnemonic::MOV)
///     .add_operand(Register::EDX)
///     .add_operand(0x1234)
///     .encode();
/// assert_eq!(mov.unwrap(), b"\xBA\x34\x12\x00\x00");
///
/// // cmp dword ptr [rax + 123], 42
/// let cmp = EncoderRequest::new64(Mnemonic::CMP)
///     .add_operand(mem!(dword ptr [RAX + 123]))
///     .add_operand(42)
///     .encode();
/// assert_eq!(cmp.unwrap(), b"\x83\x78\x7B\x2A");
///
/// // jmp short $-2
/// let forever = EncoderRequest::new64(Mnemonic::JMP)
///     .add_operand(-2)
///     .encode();
/// assert_eq!(forever.unwrap(), b"\xEB\xFE");
///
/// // jmp long $-5
/// let forever_long = EncoderRequest::new64(Mnemonic::JMP)
///     .set_branch_width(BranchWidth::_32) // default: auto
///     .add_operand(-5)
///     .encode();
/// assert_eq!(forever_long.unwrap(), b"\xE9\xFB\xFF\xFF\xFF")
/// ```
///
/// ### Helper methods on [`Mnemonic`]
///
/// There are also two helper functions [`Mnemonic::build32`] and
/// [`Mnemonic::build64`] that allow for instantiating encoder requests in a
/// slightly more compact fashion:
///
/// ```rust
/// # use zydis::*;
/// let int3 = Mnemonic::INT3.build64().encode();
/// assert_eq!(int3.unwrap(), b"\xCC");
/// ```
///
/// ## Changing existing instructions
///
/// Previously decoded instructions can be converted into an encoder request.
/// The encoder request can then be mutated in any way you wish before
/// re-encoding it again.
///
/// ```
/// # use zydis::*;
/// let decoder = Decoder::new64();
/// let add = b"\x83\x05\x45\x23\x01\x00\x11";
///
/// // Decode the instruction.
/// let decoded: FullInstruction = decoder.decode_first(add).unwrap().unwrap();
/// assert_eq!(decoded.to_string(), "add dword ptr [rip+0x12345], 0x11");
///
/// // Convert it into an encoder request, change some stuff and re-encode it.
/// let reencoded = EncoderRequest::from(decoded)
///     .set_mnemonic(Mnemonic::SUB)
///     .replace_operand(1, 0x22)
///     .encode().unwrap();
/// assert_eq!(reencoded, b"\x83\x2D\x45\x23\x01\x00\x22");
///
/// // Decode & format it again for demonstration purposes.
/// let redec: FullInstruction = decoder.decode_first(&reencoded).unwrap().unwrap();
/// assert_eq!(redec.to_string(), "sub dword ptr [rip+0x12345], 0x22");
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EncoderRequest(ffi::EncoderRequest);

impl Deref for EncoderRequest {
    type Target = ffi::EncoderRequest;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EncoderRequest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl EncoderRequest {
    /// Create a new [`MachineMode::LONG_COMPAT_32`] request.
    pub const fn new32(mnemonic: Mnemonic) -> Self {
        Self::new(MachineMode::LONG_COMPAT_32, mnemonic)
    }

    /// Create a new [`MachineMode::LONG_64`] request.
    pub const fn new64(mnemonic: Mnemonic) -> Self {
        Self::new(MachineMode::LONG_64, mnemonic)
    }

    /// Create a new encoder request from scratch.
    pub const fn new(machine_mode: MachineMode, mnemonic: Mnemonic) -> Self {
        let mut request = unsafe { zeroed!(ffi::EncoderRequest) };
        request.machine_mode = machine_mode;
        request.mnemonic = mnemonic;
        Self(request)
    }

    /// Sets the mnemonic.
    pub const fn set_mnemonic(mut self, mnemonic: Mnemonic) -> Self {
        self.0.mnemonic = mnemonic;
        self
    }

    /// Sets the prefixes.
    ///
    /// Prefixes are simply represented using the corresponding instruction
    /// attributes. So e.g. if you wish to add a `GS` segment prefix, specify
    /// [`InstructionAttributes::HAS_SEGMENT_CS`].
    ///
    /// See [`ENCODABLE_PREFIXES`] for a list of all encodable prefixes
    /// (click the "source" button).
    pub const fn set_prefixes(mut self, prefixes: InstructionAttributes) -> Self {
        self.0.prefixes = prefixes;
        self
    }

    /// Sets the branch type.
    ///
    /// Required for branching instructions only. The default of
    /// [`BranchType::NONE`] lets the encoder pick size-optimal branch type
    /// automatically (`short` and `near` are prioritized over `far`).
    pub const fn set_branch_type(mut self, branch_type: BranchType) -> Self {
        self.0.branch_type = branch_type;
        self
    }

    /// Sets the branch width.
    ///
    /// Specifies physical size for relative immediate operands. Use [`BranchWidth::NONE`] to
    /// let encoder pick size-optimal branch width automatically. For segment:offset `far` branches
    /// this field applies to physical size of the offset part. For branching instructions without
    /// relative operands this field affects effective operand size attribute.
    pub const fn set_branch_width(mut self, branch_width: BranchWidth) -> Self {
        self.0.branch_width = branch_width;
        self
    }

    /// Sets the address size hint.
    ///
    /// Optional address size hint used to resolve ambiguities for some instructions. Generally
    /// encoder deduces address size from the [`EncoderOperand`] structures that represent
    /// explicit and implicit operands. This hint resolves conflicts when instruction's hidden
    /// operands scale with address size attribute.
    pub const fn set_address_size_hint(mut self, address_size_hint: AddressSizeHint) -> Self {
        self.0.address_size_hint = address_size_hint;
        self
    }

    /// Sets the operand size hint.
    ///
    /// Optional operand size hint used to resolve ambiguities for some instructions. Generally
    /// encoder deduces operand size from the [`EncoderOperand`] structures that represent
    /// explicit and implicit operands. This hint resolves conflicts when instruction's hidden
    /// operands scale with operand size attribute.
    pub const fn set_operand_size_hint(mut self, operand_size_hint: OperandSizeHint) -> Self {
        self.0.operand_size_hint = operand_size_hint;
        self
    }

    /// Gets a slice of the operands.
    pub const fn operands(&self) -> &[EncoderOperand] {
        unsafe {
            core::slice::from_raw_parts(
                self.0.operands.as_ptr() as *const EncoderOperand,
                self.0.operand_count as usize,
            )
        }
    }

    /// Gets a mutable slice of the operands.
    pub fn operands_mut(&mut self) -> &mut [EncoderOperand] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.0.operands.as_mut_ptr() as *mut EncoderOperand,
                self.0.operand_count as usize,
            )
        }
    }

    /// Adds an operand to the request.
    ///
    /// # Panics
    ///
    /// If the operand count exceeds [`ENCODER_MAX_OPERANDS`].
    pub fn add_operand(mut self, op: impl Into<EncoderOperand>) -> Self {
        assert!(
            self.0.operand_count < ENCODER_MAX_OPERANDS as _,
            "too many operands"
        );
        self.0.operands[self.0.operand_count as usize] = op.into().0;
        self.0.operand_count += 1;
        self
    }

    /// Clears the operand list.
    pub const fn clear_operands(mut self) -> Self {
        self.0.operand_count = 0;
        self
    }

    /// Replaces the operand at the given index.
    ///
    /// # Panics
    ///
    /// If the index was not previously populated.
    pub fn replace_operand(mut self, idx: usize, new: impl Into<EncoderOperand>) -> Self {
        assert!(
            idx < self.0.operand_count as _,
            "operand index out of bounds"
        );
        self.0.operands[idx] = new.into().0;
        self
    }

    /// Encodes the instruction into the given buffer.
    pub fn encode_into(&self, buf: &mut [u8]) -> Result<usize> {
        unsafe {
            let mut length = buf.len();
            ffi::ZydisEncoderEncodeInstruction(&self.0, buf.as_ptr() as _, &mut length)
                .as_result()?;
            Ok(length)
        }
    }

    /// Appends the encoded instruction to the given buffer.
    ///
    /// On failure the output buffer remains untouched.
    pub fn encode_extend(&self, buf: &mut impl Extend<u8>) -> Result<usize> {
        let mut tmp = [0; MAX_INSTRUCTION_LENGTH];
        let length = self.encode_into(&mut tmp)?;
        buf.extend(tmp.into_iter().take(length));
        Ok(length)
    }

    /// Encodes the instruction into a new buffer.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut out = vec![0; MAX_INSTRUCTION_LENGTH];
        let length = self.encode_into(&mut out[..])?;
        out.resize(length, 0);
        Ok(out)
    }
}

/// Converts a decoded instruction into an encoder request.
impl<const N: usize> From<Instruction<OperandArrayVec<N>>> for EncoderRequest {
    fn from(instr: Instruction<OperandArrayVec<N>>) -> Self {
        unsafe {
            let ops = instr.visible_operands();
            let mut request = MaybeUninit::uninit();
            ffi::ZydisEncoderDecodedInstructionToEncoderRequest(
                &*instr,
                ops.as_ptr(),
                ops.len() as _,
                request.as_mut_ptr(),
            )
            .as_result()
            .expect(
                "our rust wrapper for instructions is immutable and unchanged decoded \
                 instructions should always be convertible",
            );
            Self(request.assume_init())
        }
    }
}

/// Describes an operand in an [`EncoderRequest`].
///
/// You'll likely not want to construct these explicitly in most cases
/// and instead rely on the [`From`] implementations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct EncoderOperand(ffi::EncoderOperand);

impl EncoderOperand {
    #[doc(hidden)] // needed in `mem!` macro
    pub const ZERO_MEM: ffi::OperandMemory = unsafe { zeroed!(ffi::OperandMemory) };
    const ZERO_PTR: ffi::OperandPointer = unsafe { zeroed!(ffi::OperandPointer) };
    const ZERO_REG: ffi::OperandRegister = unsafe { zeroed!(ffi::OperandRegister) };

    /// Creates a new register operand.
    pub const fn reg(reg: Register) -> Self {
        Self::reg_is4(reg, false)
    }

    /// Creates a new register operand and specify `is4`.
    ///
    /// > Is this 4th operand (`VEX`/`XOP`). Despite its name, `is4` encoding can sometimes be
    /// > applied to 3rd operand instead of 4th. This field is used to resolve such ambiguities.
    /// > For all other operands it should be set to `ZYAN_FALSE`.
    pub const fn reg_is4(reg: Register, is4: bool) -> Self {
        Self(ffi::EncoderOperand {
            ty: OperandType::REGISTER,
            reg: ffi::OperandRegister {
                value: reg as _,
                is4,
            },
            mem: Self::ZERO_MEM,
            ptr: Self::ZERO_PTR,
            imm: 0,
        })
    }

    /// Creates a new `[disp]` memory operand.
    ///
    /// Note that only very few instructions actually accept a full 64-bit
    /// displacement. You'll typically only be able to use the lower 32 bits
    /// or encoding will fail.
    pub const fn mem_abs(size_bytes: u16, abs_disp: u64) -> Self {
        Self::mem_custom(ffi::OperandMemory {
            displacement: abs_disp as i64,
            size: size_bytes,
            ..Self::ZERO_MEM
        })
    }

    /// Creates a new `[reg + disp]` memory operand.
    pub const fn mem_base_disp(size_bytes: u16, base: Register, disp: i32) -> Self {
        Self::mem_custom(ffi::OperandMemory {
            base,
            displacement: disp as _,
            size: size_bytes,
            ..Self::ZERO_MEM
        })
    }

    /// Creates a new `[scale * index]` memory operand.
    ///
    /// Scale can only be 1, 2, 4, or 8.
    pub const fn mem_index_scale(size_bytes: u16, index: Register, scale: u8) -> Self {
        Self::mem_custom(ffi::OperandMemory {
            index,
            scale,
            size: size_bytes,
            ..Self::ZERO_MEM
        })
    }

    /// Creates a custom new memory operand.
    pub const fn mem_custom(mem: ffi::OperandMemory) -> Self {
        Self(ffi::EncoderOperand {
            ty: OperandType::MEMORY,
            reg: Self::ZERO_REG,
            mem,
            ptr: Self::ZERO_PTR,
            imm: 0,
        })
    }

    /// Creates a new pointer operand.
    pub const fn ptr(segment: u16, offset: u32) -> Self {
        Self(ffi::EncoderOperand {
            ty: OperandType::POINTER,
            reg: Self::ZERO_REG,
            mem: Self::ZERO_MEM,
            ptr: ffi::OperandPointer { segment, offset },
            imm: 0,
        })
    }

    /// Creates a new immediate operand (unsigned).
    pub const fn imm(imm: u64) -> Self {
        Self(ffi::EncoderOperand {
            ty: OperandType::IMMEDIATE,
            reg: Self::ZERO_REG,
            mem: Self::ZERO_MEM,
            ptr: Self::ZERO_PTR,
            imm,
        })
    }

    /// Creates a new immediate operand (signed).
    pub const fn imm_signed(imm: i64) -> Self {
        Self::imm(imm as _)
    }
}

impl Deref for EncoderOperand {
    type Target = ffi::EncoderOperand;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EncoderOperand {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Register> for EncoderOperand {
    fn from(reg: Register) -> Self {
        Self::reg(reg)
    }
}

macro_rules! impl_imm_from_primitive {
    ( $($prim:ident as $_as:ident),* ) => {$(
        impl From<$prim> for EncoderOperand {
            fn from(imm: $prim) -> Self {
                Self::imm(imm as $_as as u64)
            }
        }
    )*};
}

#[rustfmt::skip]
impl_imm_from_primitive![
    u64 as u64,
    u32 as u64,
    u16 as u64,
    u8  as u64,
    
    i64 as i64,
    i32 as i64,
    i16 as i64,
    i8  as i64
];

#[doc(hidden)]
pub mod mem_macro_plumbing {
    pub enum DispOrBase {
        Disp(i64),
        Base(crate::Register),
    }

    impl From<i64> for DispOrBase {
        fn from(disp: i64) -> DispOrBase {
            DispOrBase::Disp(disp)
        }
    }

    impl From<crate::Register> for DispOrBase {
        fn from(base: crate::Register) -> DispOrBase {
            DispOrBase::Base(base)
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! mem_impl {
    (@size byte) => { 8/8 };
    (@size word) => { 16/8 };
    (@size dword) => { 32/8 };
    (@size fword) => { 48/8 };
    (@size qword) => { 64/8 };
    (@size tbyte) => { 80/8 };
    (@size xmmword) => { 128/8 };
    (@size ymmword) => { 256/8 };
    (@size zmmword) => { 512/8 };
    (@size $x:tt) => { compile_error!(concat!("bad operand size: ", stringify!($x))) };

    (@base_or_disp $x:ident $disp:literal) => {
        $x.displacement = $disp;
    };
    (@base_or_disp $x:ident $base:ident $($tail:tt)*) => {
        $x.base = $crate::Register::$base;
        $crate::mem_impl!(@index_or_disp_or_scale $x $($tail)*);
    };
    (@base_or_disp $x:ident ($base_or_disp:expr) $($tail:tt)*) => {
        // This one is tricky because the expression can either eval to a register
        // or and integer. There is no way of telling at macro expansion time, so
        // we have to match dynamically.
        let y: $crate::mem_macro_plumbing::DispOrBase = $base_or_disp.into();
        match y {
            $crate::mem_macro_plumbing::DispOrBase::Disp(disp) => {
                $x.displacement = disp;
            }
            $crate::mem_macro_plumbing::DispOrBase::Base(base) => {
                $x.base = base;
            }
        }

        $crate::mem_impl!(@index_or_disp_or_scale $x $($tail)*);
    };

    (@index_or_disp_or_scale $x:ident) => {};
    (@index_or_disp_or_scale $x:ident + $disp:literal) => {
        $x.displacement = $disp;
    };
    (@index_or_disp_or_scale $x:ident + ($disp:expr)) => {
        $x.displacement = $disp;
    };
    (@index_or_disp_or_scale $x:ident + $index:ident $($tail:tt)*) => {
        $x.index = $crate::Register::$index;
        $crate::mem_impl!(@scale_or_disp $x $($tail)*);
    };
    (@index_or_disp_or_scale $x:ident + ($index:expr) $($tail:tt)*) => {
        $x.index = $index;
        $crate::mem_impl!(@scale_or_disp $x $($tail)*);
    };
    (@index_or_disp_or_scale $x:ident * $scale:literal $($tail:tt)*) => {
        $x.index = $x.base;
        $x.base = $crate::Register::NONE;
        $x.scale = $scale;
        $crate::mem_impl!(@scale_or_disp $x $($tail)*);
    };
    (@index_or_disp_or_scale $x:ident * ($scale:expr) $($tail:tt)*) => {
        $x.index = $x.base;
        $x.base = $crate::Register::NONE;
        $x.scale = $scale;
        $crate::mem_impl!(@scale_or_disp $x $($tail)*);
    };

    (@scale_or_disp $x:ident) => {};
    (@scale_or_disp $x:ident + $disp:literal) => {
        $x.displacement = $disp;
    };
    (@scale_or_disp $x:ident + ($disp:expr)) => {
        $x.displacement = $disp;
    };
    (@scale_or_disp $x:ident * $scale:literal $($tail:tt)*) => {
        $x.scale = $scale;
        $crate::mem_impl!(@disp $x $($tail)*);
    };
    (@scale_or_disp $x:ident * ($scale:expr) $($tail:tt)*) => {
        $x.scale = $scale;
        $crate::mem_impl!(@disp $x $($tail)*);
    };

    (@disp $x:ident) => {};
    (@disp $x:ident + $disp:literal) => {
        $x.displacement = $disp;
    };
    (@disp $x:ident + ($disp:expr)) => {
        $x.displacement = $disp;
    };
}

/// Macro for creating memory operands.
///
/// The general form is: `somesize ptr [base + index * scale + disp]`
///
/// All but one of `base`, `index`, `scale` and `disp` can be omitted,
/// but the order must be kept. For example you can't start with the
/// `disp` and then add an index expression after that.
///
/// All identifiers are parsed as register names (`Register::$ident`).
/// If you want to insert variables or expressions into the macro
/// invocation, you must wrap them in parenthesis.
///
/// # Example
///
/// ```rust
/// # use zydis::*;
/// // Literal operands. All `ident`s are assumed to be register names.
/// mem!(qword ptr [0x1234]);
/// mem!(dword ptr [RAX + 0x1234]);
/// mem!(dword ptr [RSI * 8]);
/// mem!(dword ptr [RDX + RSI * 8]);
/// mem!(dword ptr [RDX + RSI * 8 + 0x1234]);
///
/// // Operands with dynamic expressions must use parenthesis!
/// let my_dyn_disp = 0x1234 + 837434;
/// let my_dyn_reg = Register::RBX;
/// mem!(qword ptr [(my_dyn_disp)]);
/// mem!(qword ptr [(my_dyn_reg)]);
/// mem!(qword ptr [(my_dyn_reg) * (2 + 2)]);
/// mem!(qword ptr [(my_dyn_reg) * 4 + (my_dyn_disp)]);
/// mem!(qword ptr [RAX * (4 * 2) + 0x1234]);
/// ```
#[macro_export]
macro_rules! mem {
    ($size:tt ptr [ $($base_index_scale_disp:tt)* ]) => {{
        let mut x = $crate::EncoderOperand::ZERO_MEM.clone();
        x.size = $crate::mem_impl!(@size $size);
        $crate::mem_impl!(@base_or_disp x $($base_index_scale_disp)*);
        $crate::EncoderOperand::mem_custom(x)
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! insn_munch_operands {
    ($r:ident) => {};

    // Immediate operands.
    ($r:ident $lit:literal $(, $($tail:tt)*)?) => {
        $r = $r.add_operand($lit);
        $crate::insn_munch_operands!($r $($($tail)*)*);
    };

    // Register operands.
    ($r:ident $reg:ident $(, $($tail:tt)*)?) => {
        $r = $r.add_operand($crate::Register::$reg);
        $crate::insn_munch_operands!($r $($($tail)*)*);
    };

    // Memory operands.
    ($r:ident $size:tt ptr [$($mem:tt)*] $(, $($tail:tt)*)?) => {
        $r = $r.add_operand($crate::mem!($size ptr [$($mem)*]));
        $crate::insn_munch_operands!($r $($($tail)*)*);
    };

    // TODO: pointer operands for far jumps etc

    // Arbitrary expressions that eval to something `impl Into<EncoderOperand>`.
    ($r:ident ($e:expr) $(, $($tail:tt)*)?) => {
        $r = $r.add_operand($e);
        $crate::insn_munch_operands!($r $($($tail)*)*);
    };
}

/// Macro for conveniently creating encoder requests (64-bit variant).
///
/// The mnemonic is automatically qualified by prefixing `Mnemonic::$mnemonic`.
/// All identifiers within the operands are automatically qualified as
/// `Register::$reg`.
///
/// If you wish to insert variables or expressions, you need to wrap them
/// into parenthesis. The parenthesized expression must eval to something
/// that conforms to `impl Into<EncoderOperand>`.
///
/// Produces an [`EncoderRequest`] instance.
///
/// ```rust
/// # use zydis::*;
/// insn64!(MOV RAX, 1234).encode().unwrap();
/// insn64!(VPADDD YMM1, YMM2, YMM3).encode().unwrap();
/// insn64!(CMP CL, byte ptr [RIP + 10]).encode().unwrap();
///
/// // Variables and expressions must be wrapped in parenthesis.
/// let some_reg = Register::RSI;
/// let some_imm = 0x1234;
/// insn64!(PUSH (some_reg)).encode().unwrap();
/// insn64!(PUSH (some_imm + 123)).encode().unwrap();
/// insn64!(MOV RSI, (Register::RDI)).encode().unwrap();
/// ```
#[macro_export]
macro_rules! insn64 {
    ($mnemonic:ident $($operands:tt)*) => {{
        let mut r = EncoderRequest::new64($crate::Mnemonic::$mnemonic);
        $crate::insn_munch_operands!(r $($operands)*);
        r
    }}
}

/// Macro for conveniently creating encoder requests (32-bit variant).
///
/// See [`insn64`] for more details: this macro works exactly the same.
#[macro_export]
macro_rules! insn32 {
    ($mnemonic:ident $($operands:tt)*) => {{
        let mut r = EncoderRequest::new32($crate::Mnemonic::$mnemonic);
        $crate::insn_munch_operands!(r $($operands)*);
        r
    }}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insn_macro() {
        assert_eq!(
            insn32!(MOV RAX, 0x1234),
            EncoderRequest::new32(Mnemonic::MOV)
                .add_operand(Register::RAX)
                .add_operand(0x1234),
        );

        assert_eq!(
            insn64!(KADDW K2, K3, K6),
            EncoderRequest::new64(Mnemonic::KADDW)
                .add_operand(Register::K2)
                .add_operand(Register::K3)
                .add_operand(Register::K6),
        );

        assert_eq!(
            insn64!(ADD dword ptr [RAX + 234], 0x1234),
            EncoderRequest::new64(Mnemonic::ADD)
                .add_operand(EncoderOperand::mem_base_disp(4, Register::RAX, 234))
                .add_operand(0x1234),
        );
    }

    #[test]
    fn mem_macro() {
        type EO = EncoderOperand;
        type R = Register;

        assert_eq!(mem!(dword ptr [0x1337]), EO::mem_abs(4, 0x1337));
        assert_eq!(mem!(qword ptr [RAX]), EO::mem_base_disp(8, R::RAX, 0));
        assert_eq!(
            mem!(qword ptr [RDX + 0x1234]),
            EO::mem_base_disp(8, R::RDX, 0x1234)
        );
        assert_eq!(
            mem!(qword ptr [RAX + RDX]),
            EO::mem_custom(ffi::OperandMemory {
                size: 8,
                base: R::RAX,
                index: R::RDX,
                ..EncoderOperand::ZERO_MEM
            })
        );
        assert_eq!(
            mem!(qword ptr [RAX + RDX * 2]),
            EO::mem_custom(ffi::OperandMemory {
                size: 8,
                base: R::RAX,
                index: R::RDX,
                scale: 2,
                ..EncoderOperand::ZERO_MEM
            })
        );
        assert_eq!(
            mem!(qword ptr [RAX + RDX * 2 + 0x8282]),
            EO::mem_custom(ffi::OperandMemory {
                size: 8,
                base: R::RAX,
                index: R::RDX,
                scale: 2,
                displacement: 0x8282,
            })
        );
        assert_eq!(mem!(qword ptr [RAX * 4]), EO::mem_index_scale(8, R::RAX, 4));
        assert_eq!(
            mem!(qword ptr [RAX * 4 + 0x234]),
            EO::mem_custom(ffi::OperandMemory {
                size: 8,
                index: R::RAX,
                scale: 4,
                displacement: 0x234,
                ..EncoderOperand::ZERO_MEM
            })
        );
        assert_eq!(
            mem!(dword ptr [(Register::RDI)]),
            EO::mem_base_disp(4, R::RDI, 0)
        );
        assert_eq!(mem!(dword ptr [(0x1337 + 8)]), EO::mem_abs(4, 0x1337 + 8));
        assert_eq!(
            mem!(qword ptr [RAX * (2 + 2) + 0x234]),
            EO::mem_custom(ffi::OperandMemory {
                size: 8,
                index: R::RAX,
                scale: 4,
                displacement: 0x234,
                ..EncoderOperand::ZERO_MEM
            })
        );
        assert_eq!(
            mem!(qword ptr [RAX * 2 + (0x234 + 22)]),
            EO::mem_custom(ffi::OperandMemory {
                size: 8,
                index: R::RAX,
                scale: 2,
                displacement: 0x234 + 22,
                ..EncoderOperand::ZERO_MEM
            })
        );
        assert_eq!(
            mem!(dword ptr [RAX + (Register::RCX) * 8 + 888]),
            EO::mem_custom(ffi::OperandMemory {
                size: 4,
                base: R::RAX,
                index: R::RCX,
                scale: 8,
                displacement: 888
            })
        );
        assert_eq!(
            mem!(xmmword ptr [ RAX + (Register::RDX) * (1 + 1) + (0x123 + 0x33) ]),
            EO::mem_custom(ffi::OperandMemory {
                size: 128 / 8,
                base: R::RAX,
                index: R::RDX,
                scale: 2,
                displacement: 0x123 + 0x33,
            })
        );
        assert_eq!(
            mem!(byte ptr [ (Register::RSI) + (Register::RDX) * (1 + 1) + (0x123 + 0x33) ]),
            EO::mem_custom(ffi::OperandMemory {
                size: 1,
                base: R::RSI,
                index: R::RDX,
                scale: 2,
                displacement: 0x123 + 0x33,
            })
        );
    }

    #[test]
    fn encode_int3() {
        let req = EncoderRequest::new64(Mnemonic::INT3);
        let enc = req.encode().unwrap();
        assert_eq!(enc, vec![0xCC]);
    }

    #[test]
    fn encode_mov() {
        let mov = EncoderRequest::new64(Mnemonic::MOV)
            .add_operand(Register::RAX)
            .add_operand(0x1337u64)
            .encode()
            .unwrap();

        assert_eq!(mov, b"\x48\xC7\xC0\x37\x13\x00\x00");
    }

    #[test]
    fn reencode() {
        let cmp = b"\x48\x81\x78\x7B\x41\x01\x00\x00";
        let dec = Decoder::new64();

        let insn = dec.decode_first::<VisibleOperands>(cmp).unwrap().unwrap();
        assert_eq!(insn.to_string(), "cmp qword ptr [rax+0x7B], 0x141");

        let enc = EncoderRequest::from(insn)
            .set_prefixes(InstructionAttributes::HAS_SEGMENT_FS)
            .replace_operand(0, mem!(qword ptr [RDX + 0xB7]))
            .replace_operand(1, 0x1337u64)
            .encode()
            .unwrap();
        assert_eq!(enc, b"\x64\x48\x81\xBA\xB7\x00\x00\x00\x37\x13\x00\x00");

        let redec = dec.decode_first::<VisibleOperands>(&enc).unwrap().unwrap();
        assert_eq!(redec.to_string(), "cmp qword ptr fs:[rdx+0xB7], 0x1337");
    }
}
