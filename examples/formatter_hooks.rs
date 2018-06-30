//! A completely stupid example for Zydis' formatter hook API.

#[macro_use]
extern crate zydis;

use zydis::gen::*;
use zydis::*;

use std::any::Any;
use std::ffi::CString;
use std::fmt::Write;
use std::mem;
use std::ptr;

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

fn print_mnemonic(
    formatter: &Formatter,
    buffer: &mut ZydisString,
    instruction: &ZydisDecodedInstruction,
    user_data: Option<&mut Any>,
) -> ZydisResult<()> {
    match user_data {
        Some(x) => match x.downcast_mut::<UserData>() {
            Some(&mut UserData {
                ref mut omit_immediate,
                orig_print_mnemonic: Hook::PrintMnemonic(Some(orig_print_mnemonic)),
                ..
            }) => {
                *omit_immediate = true;

                if instruction.operands[(instruction.operandCount - 1) as usize].type_
                    == ZYDIS_OPERAND_TYPE_IMMEDIATE as u8
                {
                    let condition_code = unsafe {
                        instruction.operands[instruction.operandCount as usize]
                            .imm
                            .value
                            .u as usize
                    };

                    if instruction.mnemonic == ZYDIS_MNEMONIC_CMPPS as u16 && condition_code < 8 {
                        write!(buffer, "cmp{}ps", CONDITION_CODES[condition_code]).unwrap();
                        return Ok(());
                    } else if instruction.mnemonic == ZYDIS_MNEMONIC_CMPPD as u16
                        && condition_code < 8
                    {
                        write!(buffer, "cmp{}pd", CONDITION_CODES[condition_code]).unwrap();
                        return Ok(());
                    } else if instruction.mnemonic == ZYDIS_MNEMONIC_VCMPPS as u16
                        && condition_code < 0x20
                    {
                        write!(buffer, "vcmp{}ps", CONDITION_CODES[condition_code]).unwrap();
                        return Ok(());
                    } else if instruction.mnemonic == ZYDIS_MNEMONIC_VCMPPD as u16
                        && condition_code < 0x20
                    {
                        write!(buffer, "vcmp{}pd", CONDITION_CODES[condition_code]).unwrap();
                        return Ok(());
                    }
                }

                *omit_immediate = false;
                unsafe {
                    check!(
                        orig_print_mnemonic(
                            mem::transmute(formatter),
                            buffer,
                            instruction,
                            ptr::null_mut(),
                        ),
                        ()
                    )
                }
            }
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}

fn format_operand_imm(
    formatter: &Formatter,
    buffer: &mut ZydisString,
    instruction: &ZydisDecodedInstruction,
    operand: &ZydisDecodedOperand,
    user_data: Option<&mut Any>,
) -> ZydisResult<()> {
    match user_data {
        Some(mut x) => match x.downcast_ref::<UserData>() {
            Some(&UserData {
                omit_immediate,
                orig_format_operand: Hook::FormatOperandImm(Some(orig_format_operand)),
                ..
            }) => {
                if omit_immediate {
                    Err(ZYDIS_STATUS_SKIP_OPERAND)
                } else {
                    unsafe {
                        check!(
                            orig_format_operand(
                                mem::transmute(formatter),
                                buffer,
                                instruction,
                                operand,
                                user_data_to_c_void(&mut x),
                            ),
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

fn main() -> ZydisResult<()> {
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
        let insn = formatter.format_instruction(&instruction, 200, Some(&mut user_data))?;
        println!("0x{:016X} {}", ip, insn);
    }

    Ok(())
}
