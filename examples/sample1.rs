extern crate zydis;
use zydis::gen::*;
use zydis::*;


static CODE: &'static [u8] = &[
    0x51, 0x8D, 0x45, 0xFF, 0x50, 0xFF, 0x75, 0x0C, 0xFF, 0x75, 
    0x08, 0xFF, 0x15, 0xA0, 0xA5, 0x48, 0x76, 0x85, 0xC0, 0x0F, 
    0x88, 0xFC, 0xDA, 0x02, 0x00u8
];


fn main() {
    let mut formatter = Formatter::new(
        ZYDIS_FORMATTER_STYLE_INTEL
    ).unwrap();
    let mut decoder = Decoder::new(
        ZYDIS_MACHINE_MODE_LONG_64,
        ZYDIS_ADDRESS_WIDTH_64
    ).unwrap();
    
    let mut insn_ptr = 0u64;
    while let Ok(mut info) = decoder.decode(
        &CODE[insn_ptr as usize..], insn_ptr
    ) {
        let insn = formatter.format_instruction(&mut info);
        println!("0x{:016X} {}", insn_ptr, insn.unwrap());
        insn_ptr += info.length as u64;
    }
}