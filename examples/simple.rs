extern crate zydis;

use zydis::*;

#[rustfmt::skip]
static CODE: &'static [u8] = &[
    0x51, 0x8D, 0x45, 0xFF, 0x50, 0xFF, 0x75, 0x0C, 0xFF, 0x75, 0x08,
    0xFF, 0x15, 0xA0, 0xA5, 0x48, 0x76, 0x85, 0xC0, 0x0F, 0x88, 0xFC,
    0xDA, 0x02, 0x00,
];

fn main() -> Result<()> {
    let formatter = Formatter::new(FormatterStyle::Intel)?;
    let decoder = Decoder::new(MachineMode::Long64, AddressWidth::_64)?;

    // Our actual buffer.
    let mut buffer = [0u8; 200];
    // A wrapped version of the buffer allowing nicer access.
    let mut buffer = OutputBuffer::new(&mut buffer[..]);

    // TODO: There should be a way to omit the address if the user wants relative
    // addresses anyway.

    // 0x1337 is the base address for our code.
    for (instruction, ip) in decoder.instruction_iterator(CODE, 0x1337) {
        // We use Some(ip) here since we want absolute addressing based on the given
        // `ip`. If we would want to have relative addressing, we would use
        // `None` instead.
        formatter.format_instruction(&instruction, &mut buffer, Some(ip), None)?;
        println!("absolute: 0x{:016X} {}", ip, buffer);

        // Show relative format as well
        formatter.format_instruction(&instruction, &mut buffer, None, None)?;
        println!("relative:                    {}", buffer);
    }

    Ok(())
}
