//! The official Rust bindings for the Zyan Disassembler Engine.

#![deny(bare_trait_objects)]

extern crate core;

#[macro_use]
extern crate bitflags;

#[macro_use]
pub mod status;

pub mod ffi;

pub mod enums;
pub mod formatter;

pub use enums::{
    AddressWidth, BroadcastMode, CPUFlag, CPUFlagAction, ConversionMode, DecoderMode, Decorator,
    ElementType, ExceptionClass, Feature, FormatterStyle, ISAExt, ISASet, InstructionAttributes,
    InstructionCategory, InstructionEncoding, InstructionSegment, MachineMode, MaskMode,
    MemoryOperandType, Mnemonic, NumericBase, OpcodeMap, OperandAction, OperandEncoding,
    OperandType, OperandVisibility, Padding, PrefixType, Register, RegisterClass, RoundingMode,
    Signedness, SwizzleMode, OPERAND_ACTION_MASK_READ, OPERAND_ACTION_MASK_WRITE,
};
pub use ffi::{DecodedInstruction, DecodedOperand, Decoder, FormatterContext, InstructionIterator};
pub use formatter::{
    user_data_to_c_void, Formatter, FormatterProperty, Hook, WrappedAddressFunc,
    WrappedDecoratorFunc, WrappedGeneralFunc, WrappedRegisterFunc,
};
pub use status::{Result, Status};
