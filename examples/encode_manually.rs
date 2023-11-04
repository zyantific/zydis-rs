//! This example demonstrates how to manually populate and use encoder requests.

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
    EncoderRequest::new64(Mnemonic::PUSH)
        .add_operand(Register::RBP)
        .encode_extend(&mut buf)?;
    EncoderRequest::new64(Mnemonic::MOV)
        .add_operand(Register::RBP)
        .add_operand(Register::RSP)
        .encode_extend(&mut buf)?;
    EncoderRequest::new64(Mnemonic::LEA)
        .add_operand(Register::RAX)
        .add_operand(mem!(qword ptr [RDI + RSI + (args.offset)]))
        .encode_extend(&mut buf)?;
    EncoderRequest::new64(Mnemonic::POP)
        .add_operand(Register::RBP)
        .encode_extend(&mut buf)?;
    EncoderRequest::new64(Mnemonic::RET).encode_extend(&mut buf)?;

    // Decode and print the program for demonstration purposes.
    for insn in Decoder::new64().decode_all::<VisibleOperands>(&buf, 0) {
        let (offs, bytes, insn) = insn?;
        let bytes: String = bytes.iter().map(|x| format!("{x:02x} ")).collect();
        println!("0x{:04X}: {:<24} {}", offs, bytes, insn);
    }

    Ok(())
}
