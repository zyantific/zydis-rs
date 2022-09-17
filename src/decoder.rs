use crate::{
    ffi, DecoderMode, MachineMode, Result, StackWidth, Status, MAX_OPERAND_COUNT,
    MAX_OPERAND_COUNT_VISIBLE,
};

use core::{fmt, mem::MaybeUninit, ops, ptr, slice};

#[derive(Clone, Debug)]
pub struct Decoder(ffi::ZydisDecoder);

impl Decoder {
    /// Creates a new `Decoder`.
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
    pub fn enable_mode(&mut self, mode: DecoderMode, value: bool) -> Result<()> {
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
    /// let instruction = decoder.decode(INT3).unwrap().unwrap();
    /// assert_eq!(instruction.mnemonic, Mnemonic::INT3);
    /// ```
    #[inline]
    pub fn decode(&self, buffer: &[u8]) -> Result<Option<Instruction>> {
        let mut uninit_insn = MaybeUninit::<Instruction>::uninit();
        let insn_ptr = uninit_insn.as_mut_ptr();
        unsafe {
            return match ffi::ZydisDecoderDecodeInstruction(
                &self.0,
                ptr::addr_of_mut!((*insn_ptr).ctx),
                buffer.as_ptr() as _,
                buffer.len(),
                ptr::addr_of_mut!((*insn_ptr).insn),
            ) {
                Status::NoMoreData => Ok(None),
                x if x.is_error() => Err(x),
                _ => Ok(Some(uninit_insn.assume_init())),
            };
        }
    }

    /// Returns an iterator over all the instructions in the buffer.
    #[inline]
    pub fn decode_all<'decoder, 'buffer>(
        &'decoder self,
        buffer: &'buffer [u8],
        ip: u64,
    ) -> InstructionIter<'decoder, 'buffer> {
        InstructionIter {
            decoder: self,
            buffer,
            ip,
        }
    }
}

pub struct InstructionIter<'decoder, 'buffer> {
    decoder: &'decoder Decoder,
    buffer: &'buffer [u8],
    ip: u64,
}

impl<'decoder, 'buffer> InstructionIter<'decoder, 'buffer> {
    pub fn with_operands(self) -> InstructionAndOperandIter<'decoder, 'buffer> {
        InstructionAndOperandIter { inner: self }
    }
}

impl Iterator for InstructionIter<'_, '_> {
    type Item = Result<(u64, Instruction)>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.decoder.decode(self.buffer) {
            Ok(Some(insn)) => {
                self.buffer = &self.buffer[usize::from(insn.length)..];
                let insn_ip = self.ip;
                self.ip += u64::from(insn.length);
                Some(Ok((insn_ip, insn)))
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct InstructionAndOperandIter<'decoder, 'buffer> {
    inner: InstructionIter<'decoder, 'buffer>,
}

impl Iterator for InstructionAndOperandIter<'_, '_> {
    type Item = Result<(u64, Instruction, Operands<MAX_OPERAND_COUNT>)>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.map(|(ip, insn)| {
            let operands = insn.operands(self.inner.decoder);
            (ip, insn, operands)
        }))
    }
}

pub struct Instruction {
    insn: ffi::DecodedInstruction,
    ctx: ffi::ZydisDecoderContext,
}

impl Instruction {
    fn operands_internal<const N: usize>(
        &self,
        decoder: &Decoder,
        num_operands: usize,
    ) -> Operands<N> {
        let mut ops = MaybeUninit::<Operands<N>>::uninit();
        let ops_ptr = ops.as_mut_ptr();
        unsafe {
            ptr::write(ptr::addr_of_mut!((*ops_ptr).num_initialized), num_operands);
            let status = ffi::ZydisDecoderDecodeOperands(
                &decoder.0,
                &self.ctx,
                &self.insn,
                ptr::addr_of_mut!((*ops_ptr).operands) as _,
                N as u8,
            );
            assert!(
                !status.is_error(),
                "operand decoding should be infallible for valid arguments",
            );
            ops.assume_init()
        }
    }

    /// Decodes all operands, including implicit ones.
    #[inline]
    pub fn operands(&self, decoder: &Decoder) -> Operands<MAX_OPERAND_COUNT> {
        self.operands_internal::<MAX_OPERAND_COUNT>(decoder, usize::from(self.operand_count))
    }

    /// Decodes all visible operands.
    ///
    /// Visible operands are those that are printed by the formatter.
    #[inline]
    pub fn visible_operands(&self, decoder: &Decoder) -> Operands<MAX_OPERAND_COUNT_VISIBLE> {
        self.operands_internal::<MAX_OPERAND_COUNT_VISIBLE>(
            decoder,
            usize::from(self.operand_count),
        )
    }

    /// Returns offsets and sizes of all logical instruction segments.
    #[inline]
    pub fn segments(&self) -> Result<ffi::InstructionSegments> {
        unsafe {
            let mut segments = MaybeUninit::uninit();
            check!(
                ffi::ZydisGetInstructionSegments(&self.insn, segments.as_mut_ptr()),
                segments.assume_init()
            )
        }
    }
}

impl ops::Deref for Instruction {
    type Target = ffi::DecodedInstruction;

    fn deref(&self) -> &Self::Target {
        &self.insn
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

pub struct Operands<const MAX_OPERANDS: usize> {
    operands: [ffi::DecodedOperand; MAX_OPERANDS],
    num_initialized: usize,
}

impl<const N: usize> fmt::Debug for Operands<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<const N: usize> AsRef<[ffi::DecodedOperand]> for Operands<N> {
    fn as_ref(&self) -> &[ffi::DecodedOperand] {
        &self.operands[..self.num_initialized]
    }
}

impl<const N: usize> ops::Deref for Operands<N> {
    type Target = [ffi::DecodedOperand];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<const N: usize, I> ops::Index<I> for Operands<N>
where
    I: slice::SliceIndex<[ffi::DecodedOperand]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.as_ref().index(index)
    }
}
