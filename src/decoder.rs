use crate::*;
use core::{fmt, hash, marker::PhantomData, mem, mem::MaybeUninit, ops, ptr};

/// Decodes raw instruction bytes into a machine-readable struct.
#[derive(Clone, Debug)]
pub struct Decoder(ffi::Decoder);

impl Decoder {
    /// Creates a new [`Decoder`] with custom machine mode and stack width.
    #[inline]
    pub fn new(machine_mode: MachineMode, stack_width: StackWidth) -> Result<Self> {
        unsafe {
            let mut decoder = MaybeUninit::uninit();
            let status = ffi::ZydisDecoderInit(decoder.as_mut_ptr(), machine_mode, stack_width);
            if status.is_error() {
                return Err(status);
            }
            Ok(Self(decoder.assume_init()))
        }
    }

    /// Creating a typical 32 bit decoder.
    ///
    /// Machine mode is `MachineMode::LONG_COMPAT_32` and stack width is
    /// `StackWidth::_32`.
    #[inline]
    pub fn new32() -> Self {
        Self::new(MachineMode::LONG_COMPAT_32, StackWidth::_32)
            .expect("init with valid mode combination cannot fail")
    }

    /// Creating a typical 64 bit decoder.
    ///
    /// Machine mode is `MachineMode::LONG_64` and stack width is
    /// `StackWidth::_64`.
    pub fn new64() -> Self {
        Self::new(MachineMode::LONG_64, StackWidth::_64)
            .expect("init with valid mode combination cannot fail")
    }

    /// Enables or disables decoder modes.
    #[inline]
    pub fn enable_mode(&mut self, mode: DecoderMode, value: bool) -> Result<&mut Self> {
        unsafe {
            ffi::ZydisDecoderEnableMode(&mut self.0, mode, value as _).as_result()?;
            Ok(self)
        }
    }

    /// Decodes the first instruction in the given buffer.
    ///
    /// # Examples
    /// ```
    /// # use zydis::*;
    /// static INT3: &[u8] = &[0xCC];
    /// let mut decoder = Decoder::new64();
    ///
    /// let insn = decoder.decode_first::<NoOperands>(INT3).unwrap().unwrap();
    /// assert_eq!(insn.mnemonic, Mnemonic::INT3);
    /// ```
    #[inline]
    pub fn decode_first<O: Operands>(&self, buffer: &[u8]) -> Result<Option<Instruction<O>>> {
        let mut uninit_ctx = MaybeUninit::<ffi::DecoderContext>::uninit();
        let mut uninit_insn = MaybeUninit::<ffi::DecodedInstruction>::uninit();

        unsafe {
            match ffi::ZydisDecoderDecodeInstruction(
                &self.0,
                uninit_ctx.as_mut_ptr(),
                buffer.as_ptr() as _,
                buffer.len(),
                uninit_insn.as_mut_ptr(),
            ) {
                Status::NoMoreData => return Ok(None),
                x if x.is_error() => return Err(x),
                _ => (),
            }

            let operands = O::decode(
                &self.0,
                uninit_ctx.assume_init_ref(),
                uninit_insn.assume_init_ref(),
            );

            Ok(Some(Instruction {
                info: uninit_insn.assume_init(),
                operands,
            }))
        }
    }

    /// Returns an iterator over all the instructions in the buffer.
    ///
    /// If you don't know the instruction pointer or simply want to track the
    /// current offset within the input buffer, pass `0` as `ip`.
    #[inline]
    pub fn decode_all<'this, 'buffer, O: Operands>(
        &'this self,
        buffer: &'buffer [u8],
        ip: u64,
    ) -> InstructionIter<'this, 'buffer, O> {
        InstructionIter {
            decoder: self,
            buffer,
            ip,
            _marker: PhantomData,
        }
    }
}

/// Iterator decoding instructions in a buffer.
///
/// Created via [`Decoder::decode_all`].
#[derive(Clone)]
pub struct InstructionIter<'decoder, 'buffer, O: Operands> {
    decoder: &'decoder Decoder,
    buffer: &'buffer [u8],
    ip: u64,
    _marker: PhantomData<*const O>,
}

impl<'decoder, 'buffer, O: Operands> Iterator for InstructionIter<'decoder, 'buffer, O> {
    type Item = Result<(u64, &'buffer [u8], Instruction<O>)>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.decoder.decode_first(self.buffer) {
            Ok(Some(insn)) => {
                let ip = self.ip;
                let (insn_bytes, new_buffer) = self.buffer.split_at(usize::from(insn.length));
                self.buffer = new_buffer;
                self.ip += u64::from(insn.length);
                Some(Ok((ip, insn_bytes, insn)))
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// Convenience alias for an instruction with full operand information.
#[cfg(feature = "full-decoder")]
pub type FullInstruction = Instruction<AllOperands>;

/// Basic information about an instruction.
///
/// Instruction information can be accessed via [`core::ops::Deref`]. Please
/// refer to [`ffi::DecodedInstruction`] for a list of available fields.
#[cfg_attr(
    feature = "full-decoder",
    doc = r##"
# Example

```rust
# use zydis::*;
let ins: Instruction<VisibleOperands> = Decoder::new64()
    .decode_first(b"\xEB\xFE")
    .unwrap()
    .unwrap();

assert_eq!(ins.mnemonic, Mnemonic::JMP); // `.mnemonic` accessed via Deref impl!
assert_eq!(ins.operands().len(), 1);

let ffi::DecodedOperandKind::Imm(imm) = &ins.operands()[0].kind else {
    unreachable!() 
};

assert_eq!(imm.value, -2i64 as u64);
```
"##
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Instruction<O: Operands> {
    info: ffi::DecodedInstruction,
    operands: O,
}

impl<O: Operands> ops::Deref for Instruction<O> {
    type Target = ffi::DecodedInstruction;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

/// Simple relative instruction formatting in Intel syntax.
///
/// For more control over formatting prefer using [`crate::Formatter`] directly.
/// This also isn't terribly efficient because it instantiates a new formatter
/// on every call.
#[cfg(feature = "formatter")]
impl<const N: usize> fmt::Display for Instruction<OperandArrayVec<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt = Formatter::intel();
        let mut buffer = [0u8; 256];
        let mut buffer = OutputBuffer::new(&mut buffer);
        fmt.format_ex(None, self, &mut buffer, None)
            .map_err(|_| fmt::Error)?;
        f.write_str(buffer.as_str().map_err(|_| fmt::Error)?)
    }
}

/// Minimal instruction formatting printing just the mnemonic.
impl fmt::Display for Instruction<NoOperands> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.mnemonic.fmt(f)
    }
}

#[cfg(feature = "full-decoder")]
impl<const N: usize> Instruction<OperandArrayVec<N>> {
    /// Drops the operands, turning this object into [`Instruction<NoOperands>`].
    pub fn drop_operands(self) -> Instruction<NoOperands> {
        Instruction {
            info: self.info,
            operands: NoOperands,
        }
    }
}

impl<O: Operands> Instruction<O> {
    /// Returns offsets and sizes of all logical instruction segments.
    #[inline]
    pub fn segments(&self) -> Result<ffi::InstructionSegments> {
        unsafe {
            let mut segments = MaybeUninit::uninit();
            ffi::ZydisGetInstructionSegments(&self.info, segments.as_mut_ptr()).as_result()?;
            Ok(segments.assume_init())
        }
    }

    /// Retrieve the operand array.
    ///
    /// If `O` is [`NoOperands`], this always returns an empty slice.
    #[inline]
    pub fn operands(&self) -> &[ffi::DecodedOperand] {
        self.operands.operands()
    }
}

/// Defines storage and decoding behavior for operands.
pub trait Operands {
    fn decode(
        decoder: &ffi::Decoder,
        ctx: &ffi::DecoderContext,
        insn: &ffi::DecodedInstruction,
    ) -> Self;

    fn operands(&self) -> &[ffi::DecodedOperand];
}

/// Don't decode or store any operands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NoOperands;

impl Operands for NoOperands {
    fn decode(_: &ffi::Decoder, _: &ffi::DecoderContext, _: &ffi::DecodedInstruction) -> Self {
        Self
    }

    fn operands(&self) -> &[ffi::DecodedOperand] {
        &[]
    }
}

/// Decode and store visible operands.
#[cfg(feature = "full-decoder")]
pub type VisibleOperands = OperandArrayVec<MAX_OPERAND_COUNT_VISIBLE>;

/// Decode and store all (both visible and implicit) operands.
#[cfg(feature = "full-decoder")]
pub type AllOperands = OperandArrayVec<MAX_OPERAND_COUNT>;

/// Decode and store operands in a static array buffer.
#[cfg(feature = "full-decoder")]
pub struct OperandArrayVec<const MAX_OPERANDS: usize> {
    operands: [MaybeUninit<ffi::DecodedOperand>; MAX_OPERANDS],
    num_initialized: usize,
}

#[cfg(feature = "full-decoder")]
impl<const MAX_OPERANDS: usize> Operands for OperandArrayVec<MAX_OPERANDS> {
    fn decode(
        decoder: &ffi::Decoder,
        ctx: &ffi::DecoderContext,
        insn: &ffi::DecodedInstruction,
    ) -> Self {
        let num_operands = match MAX_OPERANDS {
            MAX_OPERAND_COUNT => usize::from(insn.operand_count),
            MAX_OPERAND_COUNT_VISIBLE => usize::from(insn.operand_count_visible),
            _ => unreachable!(),
        };

        unsafe {
            let mut ops = OperandArrayVec {
                num_initialized: num_operands,
                operands: MaybeUninit::uninit().assume_init(),
            };

            ffi::ZydisDecoderDecodeOperands(
                decoder,
                ctx,
                insn,
                ops.operands.as_mut_ptr() as _,
                MAX_OPERANDS as u8,
            )
            .as_result()
            .expect("operand decoding should be infallible for valid arguments");

            ops
        }
    }

    fn operands(&self) -> &[ffi::DecodedOperand] {
        unsafe { mem::transmute(&self.operands[..self.num_initialized]) }
    }
}

#[cfg(feature = "full-decoder")]
impl<const MAX_OPERANDS: usize> PartialEq for OperandArrayVec<MAX_OPERANDS> {
    fn eq(&self, other: &Self) -> bool {
        self.operands().eq(other.operands())
    }
}

#[cfg(feature = "full-decoder")]
impl<const MAX_OPERANDS: usize> Eq for OperandArrayVec<MAX_OPERANDS> {}

#[cfg(feature = "full-decoder")]
impl<const MAX_OPERANDS: usize> hash::Hash for OperandArrayVec<MAX_OPERANDS> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.operands().hash(state);
    }
}

#[cfg(feature = "full-decoder")]
impl<const MAX_OPERANDS: usize> Clone for OperandArrayVec<MAX_OPERANDS> {
    fn clone(&self) -> Self {
        unsafe {
            let mut operands: [MaybeUninit<ffi::DecodedOperand>; MAX_OPERANDS] =
                MaybeUninit::uninit().assume_init();

            ptr::copy_nonoverlapping(
                self.operands.as_ptr(),
                operands.as_mut_ptr(),
                self.num_initialized,
            );

            Self {
                num_initialized: self.num_initialized,
                operands,
            }
        }
    }
}

#[cfg(feature = "full-decoder")]
impl<const MAX_OPERANDS: usize> fmt::Debug for OperandArrayVec<MAX_OPERANDS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("OperandArrayVec")
            .field(&self.operands())
            .finish()
    }
}
