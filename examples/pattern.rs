//! Sample generating a simple pattern from instructions, masking out things
//! that commonly change during recompilation (displacements and immediates
//! of branch instructions).

extern crate zydis;

use zydis::*;

#[rustfmt::skip]
static CODE: &'static [u8] = &[
    0x51, 0x8D, 0x45, 0xFF, 0x50, 0xFF, 0x75, 0x0C, 0xFF, 0x75, 0x08,
    0xFF, 0x15, 0xA0, 0xA5, 0x48, 0x76, 0x85, 0xC0, 0x0F, 0x88, 0xFC,
    0xDA, 0x02, 0x00u8,
];

fn main() -> Result<()> {
    let decoder = Decoder::new(MachineMode::LONG_64, AddressWidth::_64)?;

    for (insn, ip) in decoder.instruction_iterator(CODE, 0) {
        // Max. instruction length for X86 is 15 -- a 16 byte mask does the job.
        let mut mask = 0u16;

        // Walk operands.
        for op_idx in 0..insn.operand_count as usize {
            let op = &insn.operands[op_idx];

            // Obtain offsets for relevant operands, skip others.
            let (dyn_offs, dyn_len) = match op.ty {
                OperandType::MEMORY if op.mem.disp.has_displacement => {
                    (insn.raw.disp_offset, insn.raw.disp_size)
                }
                OperandType::IMMEDIATE if op.imm.is_relative => {
                    (insn.raw.imm[op_idx].offset, insn.raw.imm[op_idx].size)
                }
                _ => continue,
            };

            // Apply offsets to our bitmask.
            for i in dyn_offs..dyn_offs + dyn_len / 8 {
                mask |= 1 << i;
            }
        }

        // Print pattern.
        let len = insn.length as usize;
        let ip = ip as usize;
        for (i, byte) in (&CODE[ip..ip + len]).iter().enumerate() {
            if mask & (1 << i) != 0 {
                print!("??");
            } else {
                print!("{:02X}", byte);
            }
        }

        println!();
    }

    Ok(())
}
