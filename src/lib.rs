//! The official Rust bindings for the Zyan Disassembler Engine.

#![deny(bare_trait_objects)]

extern crate core;

#[macro_use]
pub mod status;

pub mod decoder;
pub mod enums;
pub mod formatter;
pub mod gen;
pub mod mnemonic;
pub mod register;

pub use decoder::{Decoder, InstructionIterator};
pub use formatter::{
    user_data_to_c_void, Formatter, FormatterProperty, Hook, WrappedAddressFunc,
    WrappedDecoratorFunc, WrappedGeneralFunc, WrappedRegisterFunc,
};
pub use gen::{
    AddressFormat, AddressWidth, CPUFlag, CPUFlagAction, CPUFlags, DecoderMode, DecoratorType,
    DisplacementFormat, FormatterContext, FormatterStyle, ImmediateFormat, Instruction,
    MachineMode, Mnemonic, Operand, Register, RegisterClass, Status, ZyanString,
};
pub use mnemonic::MnemonicMethods;
pub use register::{RegisterClassExtensions, RegisterMethods};
pub use status::{status_description, Error, Result};
