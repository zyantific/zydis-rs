//! This example demonstrates how to encode using the `insn64` macro.

use zydis::*;

/// example encoding a function that adds two numbers
#[derive(argh::FromArgs)]
struct Args {
    /// offset to apply
    #[argh(option, short = 'o', default = "0x123")]
    offset: i64,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    // Encode a simple `add` function with a stack-frame in Sys-V ABI.
    let mut buf = Vec::with_capacity(128);
    let mut add = |request: EncoderRequest| request.encode_extend(&mut buf);

    add(insn64!(PUSH RBP))?;
    add(insn64!(MOV RBP, RSP))?;
    add(insn64!(LEA RAX, qword ptr [RDI + RSI + (args.offset)]))?;
    add(insn64!(POP RBP))?;
    add(insn64!(RET))?;

    // Decode and print the program for demonstration purposes.
    for insn in Decoder::new64().decode_all::<VisibleOperands>(&buf, 0) {
        let (offs, bytes, insn) = insn?;
        let bytes: String = bytes.iter().map(|x| format!("{x:02x} ")).collect();
        println!("0x{:04X}: {:<24} {}", offs, bytes, insn);
    }

    Ok(())
}
