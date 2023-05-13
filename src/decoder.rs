use crate::{
    ffi, DecoderMode, MachineMode, Result, StackWidth, Status, MAX_OPERAND_COUNT,
    MAX_OPERAND_COUNT_VISIBLE,
};

use core::{mem::MaybeUninit, ops, ptr};
use std::{fmt, marker::PhantomData};

/// Decoder for X86/X86-64 instructions.
///
/// Decodes raw instruction bytes into a machine-processable structure.
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
    pub fn new32() -> Result<Self> {
        Self::new(MachineMode::LONG_COMPAT_32, StackWidth::_32)
    }

    /// Creating a typical 64 bit decoder.
    ///
    /// Machine mode is `MachineMode::LONG_64` and stack width is
    /// `StackWidth::_64`.
    pub fn new64() -> Result<Self> {
        Self::new(MachineMode::LONG_64, StackWidth::_64)
    }

    /// Enables or disables decoder modes.
    #[inline]
    pub fn enable_mode(&mut self, mode: DecoderMode, value: bool) -> Result {
        unsafe { check!(ffi::ZydisDecoderEnableMode(&mut self.0, mode, value as _)) }
    }

    /// Decodes the first instruction in the given buffer.
    ///
    /// # Examples
    /// ```
    /// # use zydis::*;
    /// static INT3: &[u8] = &[0xCC];
    /// let mut decoder = Decoder::new64().unwrap();
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

/// Basic information about an instruction.
///
/// Instruction information can be accessed via [`Deref`].
#[derive(Debug, Clone)]
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
#[cfg(not(feature = "minimal"))]
impl<const N: usize> fmt::Display for Instruction<OperandArrayVec<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::{Formatter, FormatterStyle, OutputBuffer};
        let fmt = Formatter::new(FormatterStyle::INTEL).unwrap();
        let mut buffer = [0u8; 256];
        let mut buffer = OutputBuffer::new(&mut buffer);
        fmt.format_into_output_buf(None, self, &mut buffer)
            .map_err(|_| fmt::Error)?;
        f.write_str(buffer.as_str().map_err(|_| fmt::Error)?)
    }
}

/// Minimal instruction formatting printing just the mnemonic.
impl fmt::Display for Instruction<NoOperands> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.mnemonic.get_string().ok_or(fmt::Error)?)
    }
}

impl<const N: usize> Instruction<OperandArrayVec<N>> {
    /// Drops the operands, turning it into [`Instruction<NoOperands>`].
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
            check!(
                ffi::ZydisGetInstructionSegments(&self.info, segments.as_mut_ptr()),
                segments.assume_init()
            )
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
#[derive(Debug, Clone, Copy)]
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
pub type VisibleOperands = OperandArrayVec<MAX_OPERAND_COUNT_VISIBLE>;

/// Decode and store all (both visible and implicit) operands.
pub type AllOperands = OperandArrayVec<MAX_OPERAND_COUNT>;

/// Decode and store operands in a static array buffer.
pub struct OperandArrayVec<const MAX_OPERANDS: usize> {
    // TODO: use maybeuninit here
    operands: [ffi::DecodedOperand; MAX_OPERANDS],
    num_initialized: usize,
}

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

        let mut ops = MaybeUninit::<Self>::uninit();
        let ops_ptr = ops.as_mut_ptr();
        unsafe {
            ptr::write(ptr::addr_of_mut!((*ops_ptr).num_initialized), num_operands);
            let status = ffi::ZydisDecoderDecodeOperands(
                decoder,
                ctx,
                insn,
                ptr::addr_of_mut!((*ops_ptr).operands) as _,
                MAX_OPERANDS as u8,
            );
            assert!(
                !status.is_error(),
                "operand decoding should be infallible for valid arguments",
            );
            ops.assume_init()
        }
    }

    fn operands(&self) -> &[ffi::DecodedOperand] {
        &self.operands[..self.num_initialized]
    }
}
