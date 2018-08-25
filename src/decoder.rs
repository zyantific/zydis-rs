//! Binary instruction decoding.

use core::mem::uninitialized;

use gen::*;
use status::Result;

pub struct Decoder {
    decoder: ZydisDecoder,
}

impl Decoder {
    /// Creates a new `Decoder` with the given `machine_mode` and
    /// `address_width`.
    pub fn new(machine_mode: MachineMode, address_width: AddressWidth) -> Result<Decoder> {
        unsafe {
            let mut decoder = uninitialized();
            check!(
                ZydisDecoderInit(&mut decoder, machine_mode, address_width),
                Decoder { decoder }
            )
        }
    }

    /// Enables or disables (depending on the `value`) the given decoder `mode`.
    pub fn enable_mode(&mut self, mode: DecoderMode, value: bool) -> Result<()> {
        unsafe {
            check!(
                ZydisDecoderEnableMode(&mut self.decoder, mode, value as _),
                ()
            )
        }
    }

    /// Decodes a binary instruction to `ZydisDecodedInstruction`, taking
    /// additional flags.
    ///
    /// # Examples
    ///
    /// ```
    /// use zydis::gen::*;
    /// static INT3: &'static [u8] = &[0xCCu8];
    /// let decoder = zydis::Decoder::new(ZYDIS_MACHINE_MODE_LONG_64, ZYDIS_ADDRESS_WIDTH_64).unwrap();
    /// let info = decoder.decode(INT3).unwrap().unwrap();
    /// assert_eq!(info.mnemonic, ZYDIS_MNEMONIC_INT3);
    /// ```
    pub fn decode(&self, buffer: &[u8]) -> Result<Option<Instruction>> {
        unsafe {
            let mut insn: ZydisDecodedInstruction = uninitialized();
            check_option!(
                ZydisDecoderDecodeBuffer(
                    &self.decoder,
                    buffer.as_ptr() as _,
                    buffer.len(),
                    &mut insn
                ),
                insn
            )
        }
    }

    /// Returns an iterator over all the instructions in the buffer.
    pub fn instruction_iterator<'a, 'b>(
        &'a self,
        buffer: &'b [u8],
        ip: u64,
    ) -> InstructionIterator<'a, 'b> {
        InstructionIterator {
            decoder: self,
            buffer,
            ip,
        }
    }
}

pub struct InstructionIterator<'a, 'b> {
    decoder: &'a Decoder,
    buffer: &'b [u8],
    ip: u64,
}

impl<'a, 'b> Iterator for InstructionIterator<'a, 'b> {
    type Item = (Instruction, u64);

    fn next(&mut self) -> Option<Self::Item> {
        match self.decoder.decode(self.buffer) {
            Ok(Some(insn)) => {
                self.buffer = &self.buffer[insn.length as usize..];
                let item = Some((insn, self.ip));
                self.ip += u64::from(insn.length);
                item
            }
            _ => None,
        }
    }
}
