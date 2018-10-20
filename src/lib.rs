//! The official Rust bindings for the Zyan Disassembler Engine.

#![deny(bare_trait_objects)]

extern crate core;

#[macro_use]
extern crate bitflags;

#[cfg(feature = "serialization")]
extern crate serde;

#[cfg(feature = "serialization")]
#[macro_use]
extern crate serde_derive;

#[macro_use]
pub mod status;

pub mod ffi;

pub mod enums;
pub mod formatter;

pub use enums::{
    AddressWidth, BranchType, BroadcastMode, CPUFlag, CPUFlagAction, ConversionMode, DecoderMode,
    Decorator, ElementType, ExceptionClass, Feature, FormatterStyle, ISAExt, ISASet,
    InstructionAttributes, InstructionCategory, InstructionEncoding, InstructionSegment,
    MachineMode, MaskMode, MemoryOperandType, Mnemonic, NumericBase, OpcodeMap, OperandAction,
    OperandEncoding, OperandType, OperandVisibility, Padding, PrefixType, Register, RegisterClass,
    RoundingMode, Signedness, SwizzleMode, Token, OPERAND_ACTION_MASK_READ,
    OPERAND_ACTION_MASK_WRITE, TOKEN_ADDRESS_ABS, TOKEN_ADDRESS_REL, TOKEN_DECORATOR,
    TOKEN_DELIMITER, TOKEN_DISPLACEMENT, TOKEN_IMMEDIATE, TOKEN_INVALID, TOKEN_MNEMONIC,
    TOKEN_PARENTHESIS_CLOSE, TOKEN_PARENTHESIS_OPEN, TOKEN_PREFIX, TOKEN_REGISTER, TOKEN_SYMBOL,
    TOKEN_TYPECAST, TOKEN_USER, TOKEN_WHITESPACE,
};
pub use ffi::{
    get_version, DecodedInstruction, DecodedOperand, Decoder, FormatterBuffer, FormatterContext,
    FormatterToken, InstructionIterator,
};
pub use formatter::{
    user_data_to_c_void, Formatter, FormatterProperty, Hook, OutputBuffer, WrappedDecoratorFunc,
    WrappedGeneralFunc, WrappedRegisterFunc,
};
pub use status::{Result, Status};
