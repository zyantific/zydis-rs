//! A completely stupid example for Zydis' formatter hook API.

use std::{ffi::CString, fmt::Write, mem};
use zydis::{ffi::DecodedOperandKind, *};

#[rustfmt::skip]
static CODE: &[u8] = &[
    // cmpps xmm1, xmm4, 0x03
    0x0F, 0xC2, 0xCC, 0x03,
    // vcmppd xmm1, xmm2, xmm3, 0x17
    0xC5, 0xE9, 0xC2, 0xCB, 0x17,
    // vcmpps k2 {k7}, zmm2, dword ptr ds:[rax + rbx*4 + 0x100] {1to16}, 0x0F
    0x62, 0xF1, 0x6C, 0x5F, 0xC2, 0x54, 0x98, 0x40, 0x0F
];

static CONDITION_CODES: &[&str] = &[
    "eq", "lt", "le", "unord", "neq", "nlt", "nle", "ord", "eq_uq", "nge", "ngt", "false", "oq",
    "ge", "gt", "true", "eq_os", "lt_oq", "le_oq", "unord_s", "neq_us", "nlt_uq", "nle_uq",
    "ord_s", "eq_us", "nge_uq", "ngt_uq", "false_os", "neg_os", "ge_oq", "gt_oq", "true_us",
];

// Used with .map_err
fn user_err<T>(_: T) -> Status {
    Status::User
}

struct UserData {
    orig_print_mnemonic: Hook,
    orig_format_operand: Hook,
    omit_immediate: bool,
}

fn print_mnemonic(
    formatter: &Formatter<UserData>,
    buffer: &mut ffi::FormatterBuffer,
    ctx: &mut ffi::FormatterContext,
    user_data: Option<&mut UserData>,
) -> Result<()> {
    let instruction = unsafe { &*ctx.instruction };
    let operands =
        unsafe { core::slice::from_raw_parts(ctx.operands, instruction.operand_count as usize) };
    let user_data = user_data.unwrap();
    if let Hook::PrintMnemonic(orig_print_mnemonic) = user_data.orig_print_mnemonic {
        user_data.omit_immediate = true;

        let count = instruction.operand_count as usize;

        if count > 0 {
            if let DecodedOperandKind::Imm(imm) = &operands[count - 1].kind {
                let cc = imm.value as usize;
                match instruction.mnemonic {
                    Mnemonic::CMPPS if cc < 8 => {
                        buffer.append(TOKEN_MNEMONIC)?;
                        let string = buffer.get_string()?;
                        return write!(string, "cmp{}ps", CONDITION_CODES[cc]).map_err(user_err);
                    }
                    Mnemonic::CMPPD if cc < 8 => {
                        buffer.append(TOKEN_MNEMONIC)?;
                        let string = buffer.get_string()?;
                        return write!(string, "cmp{}pd", CONDITION_CODES[cc]).map_err(user_err);
                    }
                    Mnemonic::VCMPPS if cc < 0x20 => {
                        buffer.append(TOKEN_MNEMONIC)?;
                        let string = buffer.get_string()?;
                        return write!(string, "vcmp{}ps", CONDITION_CODES[cc]).map_err(user_err);
                    }
                    Mnemonic::VCMPPD if cc < 0x20 => {
                        buffer.append(TOKEN_MNEMONIC)?;
                        let string = buffer.get_string()?;
                        return write!(string, "vcmp{}pd", CONDITION_CODES[cc]).map_err(user_err);
                    }
                    _ => {}
                }
            }
        }

        user_data.omit_immediate = false;
        unsafe { orig_print_mnemonic(mem::transmute(formatter), buffer, ctx).into() }
    } else {
        Ok(())
    }
}

fn format_operand_imm(
    formatter: &Formatter<UserData>,
    buffer: &mut ffi::FormatterBuffer,
    ctx: &mut ffi::FormatterContext,
    user_data: Option<&mut UserData>,
) -> Result<()> {
    let user_data = user_data.unwrap();
    if let Hook::FormatOperandImm(orig_format_operand) = user_data.orig_format_operand {
        if user_data.omit_immediate {
            Err(Status::SkipToken)
        } else {
            unsafe { orig_format_operand(mem::transmute(formatter), buffer, ctx).as_result() }
        }
    } else {
        Ok(())
    }
}

fn main() -> Result<()> {
    let s = CString::new("h").unwrap();

    let mut formatter = Formatter::<UserData>::new_custom_userdata(FormatterStyle::INTEL);
    formatter.set_property(FormatterProperty::ForceSegment(true))?;
    formatter.set_property(FormatterProperty::ForceSize(true))?;

    // clear old prefix
    formatter.set_property(FormatterProperty::HexPrefix(None))?;
    // set h as suffix
    formatter.set_property(FormatterProperty::HexSuffix(Some(s.as_c_str())))?;

    let decoder = Decoder::new64();

    let mut buffer = [0u8; 200];
    let mut buffer = OutputBuffer::new(&mut buffer[..]);

    // First without hooks
    for item in decoder.decode_all::<VisibleOperands>(CODE, 0) {
        let (ip, _, insn) = item?;
        formatter.format_raw(&insn, insn.operands(), &mut buffer, Some(ip), None)?;
        println!("0x{:016X} {}", ip, buffer);
    }

    println!();

    // Now set the hooks
    let orig_print_mnemonic = formatter.set_print_mnemonic(Box::new(print_mnemonic))?;
    let orig_format_operand = formatter.set_format_operand_imm(Box::new(format_operand_imm))?;

    let mut user_data = UserData {
        orig_print_mnemonic,
        orig_format_operand,
        omit_immediate: false,
    };

    // And print it with hooks
    for item in decoder.decode_all::<VisibleOperands>(CODE, 0) {
        let (ip, _, insn) = item?;
        formatter.format_raw(
            &insn,
            insn.operands(),
            &mut buffer,
            Some(ip),
            Some(&mut user_data),
        )?;
        println!("0x{:016X} {}", ip, buffer);
    }

    Ok(())
}
