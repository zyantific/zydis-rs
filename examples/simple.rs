extern crate zydis;

use zydis::*;

#[rustfmt::skip]
static CODE: &'static [u8] = &[
    0x51, 0x8D, 0x45, 0xFF, 0x50, 0xFF, 0x75, 0x0C, 0xFF, 0x75, 0x08,
    0xFF, 0x15, 0xA0, 0xA5, 0x48, 0x76, 0x85, 0xC0, 0x0F, 0x88, 0xFC,
    0xDA, 0x02, 0x00,
];

fn main() -> Result<()> {
    let fmt = Formatter::<()>::new(FormatterStyle::INTEL)?;
    let decoder = Decoder::new(MachineMode::LONG_64, StackWidth::_64)?;

    // 0x1337 is the base address for our code.
    for item in decoder.decode_all::<VisibleOperands>(CODE, 0x1337) {
        let (ip, _raw_bytes, insn) = item?;

        // We use `Some(ip)` if we want absolute addressing based on the given
        // instruction pointer, or `None` for relative addressing.
        println!("absolute: 0x{:016X} {}", ip, fmt.format(Some(ip), &insn)?);
        println!("relative:                    {}", fmt.format(None, &insn)?);
    }

    Ok(())
}
