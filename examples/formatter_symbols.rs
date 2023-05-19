use zydis::{
    ffi, Decoder, Formatter, FormatterProperty, FormatterStyle, Hook, OutputBuffer,
    Result as ZydisResult, Status, VisibleOperands, TOKEN_SYMBOL,
};

use std::fmt::Write;

#[rustfmt::skip]
const CODE: &[u8] = &[
    0x48, 0x8B, 0x05, 0x39, 0x00, 0x13, 0x00, // mov rax, qword ptr ds:[<SomeModule.SomeData>]
    0x50,                                     // push rax
    0xFF, 0x15, 0xF2, 0x10, 0x00, 0x00,       // call qword ptr ds:[<SomeModule.SomeFunction>]
    0x85, 0xC0,                               // test eax, eax
    0x0F, 0x84, 0x00, 0x00, 0x00, 0x00,       // jz 0x007FFFFFFF400016
    0xE9, 0xE5, 0x0F, 0x00, 0x00              // jmp <SomeModule.EntryPoint>
];

const SYMBOL_TABLE: &[(u64, &str)] = &[
    (0x007FFFFFFF401000, "SomeModule.EntryPoint"),
    (0x007FFFFFFF530040, "SomeModule.SomeData"),
    (0x007FFFFFFF401100, "SomeModule.SomeFunction"),
];

fn print_address(
    formatter: &Formatter<ffi::FormatterFunc>,
    buffer: &mut ffi::FormatterBuffer,
    context: &mut ffi::FormatterContext,
    user_data: Option<&mut ffi::FormatterFunc>,
) -> ZydisResult<()> {
    let addr = unsafe {
        (*context.instruction).calc_absolute_address(context.runtime_address, &*context.operand)
    }?;

    match SYMBOL_TABLE.iter().find(|&&(x, _)| x == addr) {
        Some((_, symbol)) => {
            buffer.append(TOKEN_SYMBOL)?;
            write!(buffer.get_string()?, "<{}>", symbol).map_err(|_| Status::User)
        }
        None => unsafe {
            let orig_fn = user_data.unwrap();
            (orig_fn)(formatter.raw(), buffer, context).as_result()
        },
    }
}

fn main() -> ZydisResult<()> {
    let decoder = Decoder::new64()?;

    let mut formatter = Formatter::<ffi::FormatterFunc>::new_custom_userdata(FormatterStyle::INTEL);
    formatter.set_property(FormatterProperty::ForceSegment(true))?;
    formatter.set_property(FormatterProperty::ForceSize(true))?;

    let mut orig_print_address = if let Hook::PrintAddressAbs(x) =
        formatter.set_print_address_abs(Box::new(print_address))?
    {
        x
    } else {
        unreachable!();
    };

    let runtime_address = 0x007FFFFFFF400000;

    let mut buffer = [0u8; 200];
    let mut buffer = OutputBuffer::new(&mut buffer[..]);

    for item in decoder.decode_all::<VisibleOperands>(CODE, runtime_address) {
        let (ip, _, insn) = item?;

        formatter.format_raw(
            &insn,
            insn.operands(),
            &mut buffer,
            Some(ip),
            Some(&mut orig_print_address),
        )?;

        println!("0x{:016X} {}", ip, buffer);
    }

    Ok(())
}
