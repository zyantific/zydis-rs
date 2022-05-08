//! The official Rust bindings for the Zyan Disassembler Engine.

#![deny(bare_trait_objects)]

#[macro_use]
pub mod status;
pub mod enums;
pub mod ffi;

#[cfg(not(feature = "minimal"))]
pub mod formatter;

pub use enums::{
    BranchType, BroadcastMode, ConversionMode, DecoderMode, Decorator, ElementType, ExceptionClass,
    Feature, FormatterStyle, ISAExt, ISASet, InstructionAttributes, InstructionCategory,
    InstructionEncoding, InstructionSegment, MachineMode, MaskMode, MemoryOperandType, Mnemonic,
    NumericBase, OpcodeMap, OperandAction, OperandEncoding, OperandType, OperandVisibility,
    Padding, PrefixType, Register, RegisterClass, RoundingMode, Signedness, StackWidth,
    SwizzleMode, Token, MAX_INSTRUCTION_LENGTH, MAX_INSTRUCTION_SEGMENT_COUNT, MAX_OPERAND_COUNT,
    MAX_OPERAND_COUNT_VISIBLE, TOKEN_ADDRESS_ABS, TOKEN_ADDRESS_REL, TOKEN_DECORATOR,
    TOKEN_DELIMITER, TOKEN_DISPLACEMENT, TOKEN_IMMEDIATE, TOKEN_INVALID, TOKEN_MNEMONIC,
    TOKEN_PARENTHESIS_CLOSE, TOKEN_PARENTHESIS_OPEN, TOKEN_PREFIX, TOKEN_REGISTER, TOKEN_SYMBOL,
    TOKEN_TYPECAST, TOKEN_USER, TOKEN_WHITESPACE,
};
#[allow(deprecated)]
pub use ffi::{
    get_version, AddressWidth, DecodedInstruction, DecodedOperand, Decoder, FormatterBuffer,
    FormatterContext, FormatterToken, InstructionIterator,
};
#[cfg(not(feature = "minimal"))]
pub use formatter::{
    Formatter, FormatterProperty, Hook, OutputBuffer, WrappedDecoratorFunc, WrappedGeneralFunc,
    WrappedRegisterFunc,
};
pub use status::{Result, Status};
