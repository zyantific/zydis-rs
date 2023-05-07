use crate::{
    ffi, DecoderMode, MachineMode, Result, StackWidth, Status, MAX_OPERAND_COUNT,
    MAX_OPERAND_COUNT_VISIBLE,
};

use core::{mem::MaybeUninit, ops, ptr};

/// Decoder for X86/X86-64 instructions.
///
/// Decodes raw instruction bytes into a machine-processable structure.
#[derive(Clone, Debug)]
pub struct Decoder(ffi::Decoder);

impl Decoder {
    /// Creates a new [`Decoder`].
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

    /// Enables or disables (depending on the `value`) the given decoder `mode`:
    #[inline]
    pub fn enable_mode(&mut self, mode: DecoderMode, value: bool) -> Result {
        unsafe { check!(ffi::ZydisDecoderEnableMode(&mut self.0, mode, value as _)) }
    }

    /// Decodes the first instruction in the given buffer.
    ///
    /// # Examples
    /// ```
    /// use zydis::{Decoder, MachineMode, Mnemonic, StackWidth};
    /// static INT3: &'static [u8] = &[0xCC];
    /// let mut decoder = Decoder::new(MachineMode::LONG_64, StackWidth::_64).unwrap();
    ///
    /// let instruction = decoder.decode_first(INT3, 0).unwrap().unwrap();
    /// assert_eq!(instruction.mnemonic, Mnemonic::INT3);
    /// ```
    #[inline]
    pub fn decode_first<'this, 'buffer>(
        &'this self,
        buffer: &'buffer [u8],
        ip: u64, // TODO: Option?
    ) -> Result<Option<Instruction<'this, 'buffer>>> {
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

            Ok(Some(Instruction {
                decoder: self,
                ctx: uninit_ctx.assume_init(),
                span: buffer,
                ip,
                info: uninit_insn.assume_init(),
            }))
        }
    }

    /// Returns an iterator over all the instructions in the buffer.
    #[inline]
    pub fn decode_all<'this, 'buffer>(
        &'this self,
        buffer: &'buffer [u8],
        ip: u64, // TODO: Option?
    ) -> InstructionIter<'this, 'buffer> {
        InstructionIter {
            decoder: self,
            buffer,
            ip,
        }
    }
}

/// Iterator decoding instructions in a buffer.
///
/// Created via [`Decoder::decode_all`].
#[derive(Clone)]
pub struct InstructionIter<'decoder, 'buffer> {
    decoder: &'decoder Decoder,
    buffer: &'buffer [u8],
    ip: u64,
}

impl<'decoder, 'buffer> Iterator for InstructionIter<'decoder, 'buffer> {
    type Item = Result<Instruction<'decoder, 'buffer>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.decoder.decode_first(self.buffer, self.ip) {
            Ok(Some(insn)) => {
                self.buffer = &self.buffer[usize::from(insn.length)..];
                self.ip += u64::from(insn.length);
                Some(Ok(insn))
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
pub struct Instruction<'decoder, 'buffer> {
    decoder: &'decoder Decoder,
    ctx: ffi::DecoderContext,
    span: &'buffer [u8],
    ip: u64,
    info: ffi::DecodedInstruction,
}

impl Instruction<'_, '_> {
    /// Instruction pointer at which the current instruction resides.
    pub fn ip(&self) -> u64 {
        self.ip
    }

    /// Raw bytes of the instruction.
    pub fn bytes(&self) -> &[u8] {
        self.span
    }

    /// Decode the remaining info according to the given [`OperandDecoder`]
    /// type and create an instruction object
    pub fn into_owned<O: OperandDecoder>(self) -> OwnedInstruction<O> {
        OwnedInstruction {
            operands: O::decode(&self.decoder.0, &self.ctx, &self.info),
            info: self.info,
        }
    }

    /// Decodes the instruction's operands.
    pub fn decode_operands<O: OperandDecoder>(&self) -> O {
        O::decode(&self.decoder.0, &self.ctx, &self.info)
    }
}

impl ops::Deref for Instruction<'_, '_> {
    type Target = ffi::DecodedInstruction;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

/// Fully decoded and owned instruction, including operand information.
pub struct OwnedInstruction<O: OperandDecoder> {
    info: ffi::DecodedInstruction,
    operands: O,
}

impl<O: OperandDecoder> OwnedInstruction<O> {
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

impl<O: OperandDecoder> ops::Deref for OwnedInstruction<O> {
    type Target = ffi::DecodedInstruction;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

/// Defines storage and decoding behavior for operands.
pub trait OperandDecoder {
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

impl OperandDecoder for NoOperands {
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

impl<const MAX_OPERANDS: usize> OperandDecoder for OperandArrayVec<MAX_OPERANDS> {
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
