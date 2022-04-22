use core::{fmt, marker::PhantomData, mem::MaybeUninit, slice};

// TODO: use libc maybe, or wait for this to move into core?
use std::{
    ffi::CStr,
    os::raw::{c_char, c_void},
};

use crate::{
    enums::*,
    ffi::*,
    status::{Result, Status},
};

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct OperandMemory {
    pub base: Register,
    pub index: Register,
    pub scale: u8,
    pub displacement: i64,
    pub size: u16,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct OperandRegister {
    pub value: Register,
    pub is4: bool,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct OperandPointer {
    pub segment: u16,
    pub offset: u32
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct Operand {
    pub ty: OperandType,
    pub reg: OperandRegister,
    pub mem: OperandMemory,
    pub ptr: OperandPointer,
    /// This can be either a i64 or u64, but raw unions are kind of unergonomic to use in Rust.
    pub imm: u64,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct EvexFeatures {
    pub broadcast: BroadcastMode,
    pub rounding: RoundingMode,
    pub sae: bool,
    pub zeroing_mask: bool,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct MvexFeatures {
    pub broadcast: BroadcastMode,
    pub conversion: ConversionMode,
    pub rounding: RoundingMode,
    pub swizzle: SwizzleMode,
    pub sae: bool,
    pub eviction_hint: bool,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct Request {
    pub machine_mode: MachineMode,
    pub allowed_encodings: EncodableEncoding,
    pub mnemonic: Mnemonic,
    pub prefixes: InstructionAttributes,
    pub branch_type: BranchType,
    pub branch_width: BranchWidth,
    pub address_size_hint: AddressSizeHint,
    pub operand_size_hint: OperandSizeHint,
    pub operand_count: u8,
    // ZYDIS_ENCODER_MAX_OPERANDS,
    pub operands: [Operand; 4],
    pub evex: EvexFeatures,
    pub mvex: MvexFeatures,
}

extern "C" {
    pub fn ZydisEncoderEncodeInstruction(
        request: *const Request,
        buffer: *mut c_void,
        length: *mut usize,
    ) -> Status;

    pub fn ZydisEncoderDecodedInstructionToEncoderRequest(
        instruction: *const DecodedInstruction,
        operands: *const DecodedOperand,
        operand_count: u8,
        request: *mut Request,
    ) -> Status;
}
