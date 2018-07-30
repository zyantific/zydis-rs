//! Binary instruction decoding.

use std::mem::uninitialized;

use gen::*;
use status::{Error, Result};

pub struct Decoder {
    decoder: ZydisDecoder,
}

impl Decoder {
    /// Creates a new `Decoder` with the given `machine_mode` and `address_width`.
    pub fn new(
        machine_mode: ZydisMachineModes,
        address_width: ZydisAddressWidths,
    ) -> Result<Decoder> {
        unsafe {
            let mut decoder = uninitialized();
            check!(
                ZydisDecoderInit(&mut decoder, machine_mode as _, address_width as _,),
                Decoder { decoder }
            )
        }
    }

    /// Enables or disables (depending on the `value`) the given decoder `mode`.
    pub fn enable_mode(&mut self, mode: ZydisDecoderModes, value: bool) -> Result<()> {
        unsafe {
            check!(
                ZydisDecoderEnableMode(&mut self.decoder, mode as _, value as _),
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
    /// let decoder = zydis::Decoder::new(
    ///     ZYDIS_MACHINE_MODE_LONG_64,
    ///     ZYDIS_ADDRESS_WIDTH_64
    /// ).unwrap();
    /// let info = decoder.decode(INT3, 0x00400000).unwrap().unwrap();
    /// assert_eq!(info.mnemonic as ZydisMnemonics, ZYDIS_MNEMONIC_INT3);
    /// ```
    pub fn decode(
        &self,
        buffer: &[u8],
        instruction_pointer: u64,
    ) -> Result<Option<ZydisDecodedInstruction>> {
        unsafe {
            let mut info: ZydisDecodedInstruction = uninitialized();
            check_option!(
                ZydisDecoderDecodeBuffer(
                    &self.decoder,
                    buffer.as_ptr() as _,
                    buffer.len(),
                    instruction_pointer,
                    &mut info
                ),
                info
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
    type Item = (ZydisDecodedInstruction, u64);

    fn next(&mut self) -> Option<Self::Item> {
        match self.decoder.decode(self.buffer, self.ip) {
            Ok(Some(insn)) => {
                self.buffer = &self.buffer[insn.length as usize..];
                let item = Some((insn, self.ip));
                self.ip += insn.length as u64;
                item
            }
            _ => None,
        }
    }
}
