# NOTE: this isn't a standalone script -- meant to be run within gdb!

# gdb's rust mode doesn't parse the `<` and `>` in the types correctly
gdb.execute('set language c++')


checked_types = [
    # decoder
    ('zydis::decoder::Decoder', 'ZydisDecoder'),
    ('zydis::ffi::decoder::AccessedFlags<zydis::enums::CpuFlag>', 'ZydisAccessedFlags'),
    ('zydis::ffi::decoder::AccessedFlags<zydis::enums::FpuFlag>', 'ZydisAccessedFlags'),
    ('zydis::ffi::decoder::AvxInfo', 'ZydisDecodedInstructionAvx'),
    ('zydis::ffi::decoder::MetaInfo', 'ZydisDecodedInstructionMeta'),
    ('zydis::ffi::decoder::RawInfo', 'ZydisDecodedInstructionRaw'),
    ('zydis::ffi::decoder::MemoryInfo', 'ZydisDecodedOperandMem'),
    ('zydis::ffi::decoder::PointerInfo', 'ZydisDecodedOperandPtr'),
    ('zydis::enums::generated::Register', 'ZydisDecodedOperandReg'),
    ('zydis::ffi::decoder::ImmediateInfo', 'ZydisDecodedOperandImm'),
    ('zydis::ffi::decoder::DecodedOperand', 'ZydisDecodedOperand'),
    ('zydis::ffi::decoder::DecoderContext', 'ZydisDecoderContext'),

    # encoder
    ('zydis::ffi::encoder::OperandRegister', '((ZydisEncoderOperand*)(0))->reg'),
    ('zydis::ffi::encoder::OperandPointer', '((ZydisEncoderOperand*)(0))->ptr'),
    ('zydis::ffi::encoder::OperandMemory', '((ZydisEncoderOperand*)(0))->mem'),
    ('zydis::ffi::encoder::EncoderOperand', 'ZydisEncoderOperand'),
    ('zydis::ffi::encoder::EncoderRequest', 'ZydisEncoderRequest'),

    # formatter
    ('zydis::ffi::formatter::Formatter', 'ZydisFormatter'),
    ('zydis::ffi::formatter::FormatterBuffer', 'ZydisFormatterBuffer'),

    # zycore
    ('zydis::ffi::zycore::ZyanVector', 'ZyanVector'),
    ('zydis::ffi::zycore::ZyanString', 'ZyanString'),

    # TODO: incomplete, add more ..
]


def sizeof(ty: str) -> int:
    return gdb.parse_and_eval(f'sizeof({ty})')


for bind_ty, c_ty in checked_types:
    bind_ty_sz = sizeof(bind_ty)
    c_ty_sz = sizeof(c_ty)
    
    assert bind_ty_sz == c_ty_sz, \
        f'binding type {bind_ty} is {bind_ty_sz} bytes, but expected {c_ty_sz}'

print("ALL STRUCTS OK")
