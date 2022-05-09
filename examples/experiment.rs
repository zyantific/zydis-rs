use zydis::{Decoder, MachineMode, StackWidth};

fn main() {
    let mut decoder = Decoder::new(MachineMode::LONG_64, StackWidth::_64).unwrap();
    for x in 0u8..0xFF {
        let b = [
            x, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        match decoder.decode(&b) {
            Ok(Some((insn, operands))) => {
                let segments = insn.segments().unwrap();
                segments
            }
            Ok(None) => unreachable!(),
            Err(e) => (),
        }
    }
}
