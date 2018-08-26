//! A completely stupid example for Zydis' formatter hook API.

#![deny(bare_trait_objects)]

#[macro_use]
extern crate zydis;

use std::{any::Any, ffi::CString, fmt::Write, mem};

use zydis::{
    gen::{
        ZYDIS_ADDRESS_WIDTH_64, ZYDIS_FORMATTER_STYLE_INTEL, ZYDIS_MACHINE_MODE_LONG_64,
        ZYDIS_MNEMONIC_CMPPD, ZYDIS_MNEMONIC_CMPPS, ZYDIS_MNEMONIC_VCMPPD, ZYDIS_MNEMONIC_VCMPPS,
        ZYDIS_OPERAND_TYPE_IMMEDIATE,
    },
    *,
};

#[cfg_attr(rustfmt, rustfmt_skip)]
static CODE: &'static [u8] = &[
    // cmpps xmm1, xmm4, 0x03
    0x0F, 0xC2, 0xCC, 0x03,
    // vcmppd xmm1, xmm2, xmm3, 0x17
    0xC5, 0xE9, 0xC2, 0xCB, 0x17,
    // vcmpps k2 {k7}, zmm2, dword ptr ds:[rax + rbx*4 + 0x100] {1to16}, 0x0F
    0x62, 0xF1, 0x6C, 0x5F, 0xC2, 0x54, 0x98, 0x40, 0x0F
];

static CONDITION_CODES: &'static [&'static str] = &[
    "eq", "lt", "le", "unord", "neq", "nlt", "nle", "ord", "eq_uq", "nge", "ngt", "false", "oq",
    "ge", "gt", "true", "eq_os", "lt_oq", "le_oq", "unord_s", "neq_us", "nlt_uq", "nle_uq",
    "ord_s", "eq_us", "nge_uq", "ngt_uq", "false_os", "neg_os", "ge_oq", "gt_oq", "true_us",
];

struct UserData {
    orig_print_mnemonic: Hook,
    orig_format_operand: Hook,
    omit_immediate: bool,
}

fn user_err<T>(_: T) -> Error {
    Status::User.into()
}

fn print_mnemonic(
    formatter: &Formatter,
    buffer: &mut ZyanString,
    ctx: &mut FormatterContext,
    user_data: Option<&mut dyn Any>,
) -> Result<()> {
    let instruction = unsafe { &*ctx.instruction };
    match user_data.and_then(|x| x.downcast_mut::<UserData>()) {
        Some(&mut UserData {
            ref mut omit_immediate,
            orig_print_mnemonic: Hook::PrintMnemonic(Some(orig_print_mnemonic)),
            ..
        }) => {
            *omit_immediate = true;

            let count = instruction.operand_count as usize;

            if count > 0 && instruction.operands[count - 1].type_ == ZYDIS_OPERAND_TYPE_IMMEDIATE {
                let cc = unsafe { instruction.operands[count].imm.value.u as usize };

                match instruction.mnemonic as u32 {
                    ZYDIS_MNEMONIC_CMPPS if cc < 8 => {
                        return write!(buffer, "cmp{}ps", CONDITION_CODES[cc]).map_err(user_err)
                    }
                    ZYDIS_MNEMONIC_CMPPD if cc < 8 => {
                        return write!(buffer, "cmp{}pd", CONDITION_CODES[cc]).map_err(user_err)
                    }
                    ZYDIS_MNEMONIC_VCMPPS if cc < 0x20 => {
                        return write!(buffer, "vcmp{}ps", CONDITION_CODES[cc]).map_err(user_err)
                    }
                    ZYDIS_MNEMONIC_VCMPPD if cc < 0x20 => {
                        return write!(buffer, "vcmp{}pd", CONDITION_CODES[cc]).map_err(user_err)
                    }
                    _ => {}
                }
            }

            *omit_immediate = false;
            unsafe {
                check!(
                    orig_print_mnemonic(mem::transmute(formatter), buffer, ctx,),
                    ()
                )
            }
        }
        _ => Ok(()),
    }
}

fn format_operand_imm(
    formatter: &Formatter,
    buffer: &mut ZyanString,
    ctx: &mut FormatterContext,
    user_data: Option<&mut dyn Any>,
) -> Result<()> {
    match user_data {
        Some(x) => match x.downcast_ref::<UserData>() {
            Some(&UserData {
                omit_immediate,
                orig_format_operand: Hook::FormatOperandImm(Some(orig_format_operand)),
                ..
            }) => {
                if omit_immediate {
                    Err(Status::SkipToken.into())
                } else {
                    unsafe {
                        check!(
                            orig_format_operand(mem::transmute(formatter), buffer, ctx,),
                            ()
                        )
                    }
                }
            }
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}

fn main() -> Result<()> {
    let s = CString::new("h").unwrap();

    let mut formatter = Formatter::new(ZYDIS_FORMATTER_STYLE_INTEL)?;
    // clear old prefix
    formatter.set_property(FormatterProperty::HexPrefix(None))?;
    // set h as suffix
    formatter.set_property(FormatterProperty::HexSuffix(Some(s.as_c_str())))?;

    let orig_print_mnemonic = formatter.set_print_mnemonic(Box::new(print_mnemonic))?;
    let orig_format_operand = formatter.set_format_operand_imm(Box::new(format_operand_imm))?;

    let mut user_data = UserData {
        orig_print_mnemonic,
        orig_format_operand,
        omit_immediate: false,
    };

    let decoder = Decoder::new(ZYDIS_MACHINE_MODE_LONG_64, ZYDIS_ADDRESS_WIDTH_64)?;

    for (instruction, ip) in decoder.instruction_iterator(CODE, 0) {
        let insn = formatter.format_instruction(&instruction, 200, ip, Some(&mut user_data))?;
        println!("0x{:016X} {}", ip, insn);
    }

    Ok(())
}
