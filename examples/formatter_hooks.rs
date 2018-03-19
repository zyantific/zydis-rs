//! A completely stupid example for Zydis' formatter hook API.

extern crate zydis;
use zydis::gen::*;
use zydis::*;

use std::any::Any;
use std::ffi::CString;

#[cfg_attr(rustfmt, rustfmt_skip)]
static CODE: &'static [u8] = &[
    0x51, 0x8D, 0x45, 0xFF, 0x50, 0xFF, 0x75, 0x0C, 0xFF, 0x75, 0x08,
    0xFF, 0x15, 0xA0, 0xA5, 0x48, 0x76, 0x85, 0xC0, 0x0F, 0x88, 0xFC,
    0xDA, 0x02, 0x00u8,
];

fn print_mnemonic(
    _formatter: &Formatter,
    buffer: &mut ZydisString,
    _instruction: &ZydisDecodedInstruction,
    user_data: Option<&mut Any>,
) -> ZydisResult<()> {
    if let Some(user_data) = user_data {
        if let Some(s) = user_data.downcast_ref::<u64>() {
            buffer.append(&format!("my number: {}", s))?;
        }
    } else {
        buffer.append("abc")?;
    }

    Ok(())
}

fn real_main() -> ZydisResult<()> {
    let s = CString::new("h").unwrap();

    let mut formatter = Formatter::new(ZYDIS_FORMATTER_STYLE_INTEL)?
        // clear old prefix
        .set_property(FormatterProperty::HexPrefix(None))?
        // set h as suffix
        .set_property(FormatterProperty::HexSuffix(Some(s.as_c_str())))?;

    formatter.set_print_mnemonic(Box::new(print_mnemonic))?;

    let decoder = Decoder::new(ZYDIS_MACHINE_MODE_LONG_64, ZYDIS_ADDRESS_WIDTH_64)?;

    for (instruction, ip) in decoder.instruction_iterator(CODE, 0) {
        let insn = formatter.format_instruction(&instruction, 200, Some(&mut 1337u64))?;
        println!("0x{:016X} {}", ip, insn);
    }

    Ok(())
}

fn main() {
    real_main().unwrap();
}
