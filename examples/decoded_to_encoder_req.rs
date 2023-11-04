//! Example (and tool) that decodes and instruction and
//! prints the corresponding encoder request.

use zydis::*;

struct Mode(MachineMode, StackWidth);

impl Default for Mode {
    fn default() -> Self {
        Mode(MachineMode::LONG_64, StackWidth::_64)
    }
}

impl std::str::FromStr for Mode {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "real" => Self(MachineMode::REAL_16, StackWidth::_16),
            "16" => Self(MachineMode::LONG_COMPAT_16, StackWidth::_16),
            "32" => Self(MachineMode::LONG_COMPAT_32, StackWidth::_32),
            "64" => Self(MachineMode::LONG_64, StackWidth::_64),
            _ => return Err("unsupported machine mode"),
        })
    }
}

#[repr(transparent)]
struct InsnByte(u8);

impl std::str::FromStr for InsnByte {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        u8::from_str_radix(s, 16).map(InsnByte)
    }
}

/// Decode an instruction and transform it into an encoder request.
#[derive(argh::FromArgs)]
struct Args {
    /// machine mode. 16, 32 or 64. Default is 64.
    #[argh(option, short = 'm', default = "Mode::default()")]
    mode: Mode,

    /// instruction bytes
    #[argh(positional)]
    bytes: Vec<InsnByte>,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let dec = Decoder::new(args.mode.0, args.mode.1)?;
    let bytes: Vec<_> = args.bytes.into_iter().map(|x| x.0).collect();
    let insn = dec
        .decode_first::<VisibleOperands>(&bytes)?
        .ok_or(Status::NoMoreData)?;
    let req: EncoderRequest = insn.into();
    assert!(req.encode().is_ok());
    println!("{:#?}", req);
    Ok(())
}
