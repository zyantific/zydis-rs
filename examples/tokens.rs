use zydis::*;

#[rustfmt::skip]
static CODE: &[u8] = &[
    0x51, 0x8D, 0x45, 0xFF, 0x50, 0xFF, 0x75, 0x0C, 0xFF, 0x75, 0x08,
    0xFF, 0x15, 0xA0, 0xA5, 0x48, 0x76, 0x85, 0xC0, 0x0F, 0x88, 0xFC,
    0xDA, 0x02, 0x00,
];

fn main() -> Result<()> {
    let decoder = Decoder::new64()?;
    let formatter = Formatter::intel();

    let mut buffer = [0u8; 256];

    for insn in decoder.decode_all::<VisibleOperands>(CODE, 0) {
        let (ip, _, insn) = insn?;

        for (ty, val) in
            formatter.tokenize_raw(&insn, insn.operands(), &mut buffer[..], Some(ip), None)?
        {
            println!("token type: {}, value: {}", ty, val);
        }
        println!("----");
    }

    Ok(())
}
