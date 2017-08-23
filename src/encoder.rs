//! Binary instruction encoding.

/*
use gen::*;
use status::ZydisResult;
use std::os::raw::c_void;
use std::mem;

pub fn decoded_instruction_to_request(
    instruction: &ZydisDecodedInstruction
) -> ZydisResult<ZydisEncoderRequest> {
    unsafe {
        let mut req = mem::uninitialized();
        let status = ZydisEncoderDecodedInstructionToRequest(
            instruction, &mut req
        );
        match status { ZYDIS_STATUS_SUCCESS => Ok(req), _ => Err(status) }
    }
}


/// Encodes the given instruction info into byte-code, using a given buffer.
pub fn encode_instruction_into(
    buffer: &mut [u8], req: &mut ZydisEncoderRequest
) -> ZydisResult<usize> {
    unsafe {
        let mut insn_len = buffer.len();
        let status = ZydisEncoderEncodeInstruction(
            buffer.as_mut_ptr() as *mut c_void,
            &mut insn_len,
            req
        );
        match status { ZYDIS_STATUS_SUCCESS => Ok(insn_len), _ => Err(status) }
    }
}

/// Encodes the given instruction info into byte-code, returning a `Vec`.
///
/// # Examples
///
/// Rewriting the destination register of a `mov` instruction:
///
/// ```
/// let mut formatter = zydis::formatter::Formatter::new(
///     zydis::gen::ZYDIS_FORMATTER_STYLE_INTEL
/// ).unwrap();
/// let mut decoder = zydis::decoder::Decoder::new(
///     zydis::gen::ZYDIS_MACHINE_MODE_LONG_64,
///     zydis::gen::ZYDIS_ADDRESS_WIDTH_64
/// ).unwrap();
///
/// static MOV: &'static [u8] = b"\x48\xC7\xC0\x37\x13\x00\x00";
///
/// // Decode and format current instruction.
/// let mut info = decoder.decode(MOV, 0).unwrap();
/// let fmt = formatter.format_instruction(&mut info).unwrap();
/// assert_eq!(fmt, "mov rax, 0x1337");
///
/// // Transform and assemble / encode a patched one.
/// let mut req = zydis::encoder::decoded_instruction_to_request(&info).unwrap();
/// req.operands[0].reg = zydis::gen::ZYDIS_REGISTER_RCX as zydis::gen::ZydisRegister;
/// unsafe { *req.operands[1].imm.u.as_mut() += 1; }
/// let new_insn = zydis::encoder::encode_instruction(&mut req).unwrap();
/// assert_eq!(new_insn, b"\x48\xC7\xC1\x38\x13\x00\x00");
/// 
/// // Decode and format the new instruction.
/// let mut new_info = decoder.decode(&new_insn, 0).unwrap();
/// let new_fmt = formatter.format_instruction(&mut new_info).unwrap();
/// assert_eq!(new_fmt, "mov rcx, 0x1338");
/// ```
pub fn encode_instruction(
    req: &mut ZydisEncoderRequest
) -> ZydisResult<Vec<u8>> {
    let mut insn_buf = vec![0; ZYDIS_MAX_INSTRUCTION_LENGTH as usize];
    let insn_len = encode_instruction_into(&mut insn_buf, req)?;
    insn_buf.resize(insn_len, 0);
    Ok(insn_buf)
}
*/