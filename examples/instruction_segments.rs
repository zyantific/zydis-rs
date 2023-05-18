use zydis::*;

#[rustfmt::skip]
static CODE: &'static [u8] = &[
    0x51, 0x8D, 0x45, 0xFF, 0x50, 0xFF, 0x75, 0x0C, 0xFF, 0x75, 0x08,
    0xFF, 0x15, 0xA0, 0xA5, 0x48, 0x76, 0x85, 0xC0, 0x0F, 0x88, 0xFC,
    0xDA, 0x02, 0x00,
];

fn main() -> Result<()> {
    let decoder = Decoder::new64()?;

    for item in decoder.decode_all::<VisibleOperands>(CODE, 0) {
        let (_, _, insn) = item?;
        for segment in &insn.segments()? {
            println!(
                "type: {:?} offset: {} size: {}",
                segment.ty, segment.offset, segment.size
            );
        }
        println!("----");
    }

    Ok(())
}
