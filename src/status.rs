//! Status code utilities.

use core::{fmt, result};

/// A convenience alias for a Result, holding either a value or a status.
pub type Result<T = ()> = result::Result<T, Status>;

pub const ZYAN_MODULE_ZYCORE: usize = 0x1;
pub const ZYAN_MODULE_ZYDIS: usize = 0x2;
pub const ZYAN_MODULE_USER: usize = 0x3FF;
pub const ZYAN_MODULE_ZYDIS_RS: usize = ZYAN_MODULE_USER + 0x42;

macro_rules! make_status {
    ($error:expr, $module:expr, $code:expr) => {
        ((($error & 1) << 31) | (($module & 0x7FF) << 20) | ($code & 0xFFFFF)) as u32
    };
}

/// Status code indicating either success or failure.
#[repr(u32)]
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Status {
    Success = make_status!(0, ZYAN_MODULE_ZYCORE, 0x00),
    Failed = make_status!(1, ZYAN_MODULE_ZYCORE, 0x01),
    True = make_status!(0, ZYAN_MODULE_ZYCORE, 0x02),
    False = make_status!(0, ZYAN_MODULE_ZYCORE, 0x03),
    InvalidArgument = make_status!(1, ZYAN_MODULE_ZYCORE, 0x04),
    InvalidOperation = make_status!(1, ZYAN_MODULE_ZYCORE, 0x05),
    NotFound = make_status!(1, ZYAN_MODULE_ZYCORE, 0x06),
    OutOfRange = make_status!(1, ZYAN_MODULE_ZYCORE, 0x07),
    InsufficientBufferSize = make_status!(1, ZYAN_MODULE_ZYCORE, 0x08),
    NotEnoughMemory = make_status!(1, ZYAN_MODULE_ZYCORE, 0x09),
    BadSystemCall = make_status!(1, ZYAN_MODULE_ZYCORE, 0x0A),

    NoMoreData = make_status!(1, ZYAN_MODULE_ZYDIS, 0x00),
    DecodingError = make_status!(1, ZYAN_MODULE_ZYDIS, 0x01),
    InstructionTooLong = make_status!(1, ZYAN_MODULE_ZYDIS, 0x02),
    BadRegister = make_status!(1, ZYAN_MODULE_ZYDIS, 0x03),
    IllegalLock = make_status!(1, ZYAN_MODULE_ZYDIS, 0x04),
    IllegalLegacyPfx = make_status!(1, ZYAN_MODULE_ZYDIS, 0x05),
    IllegalRex = make_status!(1, ZYAN_MODULE_ZYDIS, 0x06),
    InvalidMap = make_status!(1, ZYAN_MODULE_ZYDIS, 0x07),
    MalformedEvex = make_status!(1, ZYAN_MODULE_ZYDIS, 0x08),
    MalformedMvex = make_status!(1, ZYAN_MODULE_ZYDIS, 0x09),
    InvalidMask = make_status!(1, ZYAN_MODULE_ZYDIS, 0x0A),
    SkipToken = make_status!(0, ZYAN_MODULE_ZYDIS, 0x0B),
    ImpossibleInstruction = make_status!(1, ZYAN_MODULE_ZYDIS, 0x0C),

    /// Generic user-defined error (e.g. for use in formatter hooks).
    User = make_status!(1, ZYAN_MODULE_ZYDIS_RS, 0x00),
    /// String isn't UTF8 encoded.
    NotUTF8 = make_status!(1, ZYAN_MODULE_ZYDIS_RS, 0x01),
    /// Rust formatter returned an error.
    FormatterError = make_status!(1, ZYAN_MODULE_ZYDIS_RS, 0x02),
}

impl Status {
    /// Returns the error code of this status.
    pub fn code(self) -> usize {
        (self as usize) & 0xFFFFF
    }

    /// Returns the module / ID space of this status code.
    ///
    /// Search doc for "ZYAN_MODULE" for the corresponding constants. This is
    /// doesn't return an enum because user-defined functions (e.g. formatter
    /// hooks) can return arbitrary values.
    pub fn module(self) -> usize {
        (self as usize >> 20) & 0x7FF
    }

    /// Whether this status code is an error.
    pub fn is_error(self) -> bool {
        (self as usize >> 31) == 1
    }

    /// Returns a human readable description of this status code.
    pub fn description(self) -> &'static str {
        match self {
            Status::Success => "no error",
            Status::Failed => "A operation failed.",
            Status::InvalidArgument => "An invalid parameter was passed to a function.",
            Status::InvalidOperation => "An attempt was made to perform an invalid operation.",
            Status::InsufficientBufferSize => {
                "A buffer passed to a function was too small to complete the requested operation."
            }
            Status::OutOfRange => "An index was out of bounds.",
            Status::NotFound => "The requested entity was not found.",
            Status::NotEnoughMemory => "Insufficient memory to perform the operation.",
            Status::BadSystemCall => "An error occurred during a system call.",
            Status::NoMoreData => {
                "An attempt was made to read data from an input data-source that has no more data \
                 available."
            }
            Status::DecodingError => {
                "An general error occured while decoding the current instruction. The instruction \
                 might be undefined."
            }
            Status::InstructionTooLong => {
                "The instruction exceeded the maximum length of 15 bytes."
            }
            Status::BadRegister => "The instruction encoded an invalid register.",
            Status::IllegalLock => {
                "A lock-prefix (F0) was found while decoding an instruction that does not support \
                 locking."
            }
            Status::IllegalLegacyPfx => {
                "A legacy-prefix (F2, F3, 66) was found while decoding a XOP/VEX/EVEX/MVEX \
                 instruction."
            }
            Status::IllegalRex => {
                "A rex-prefix was found while decoding a XOP/VEX/EVEX/MVEX instruction."
            }
            Status::InvalidMap => {
                "An invalid opcode-map value was found while decoding a XOP/VEX/EVEX/MVEX-prefix."
            }
            Status::MalformedEvex => "An error occured while decoding the EVEX-prefix.",
            Status::MalformedMvex => "An error occured while decoding the MVEX-prefix.",
            Status::InvalidMask => {
                "An invalid write-mask was specified for an EVEX/MVEX instruction."
            }
            Status::True | Status::False => "true/false not an error",
            Status::SkipToken => "skip this token",
            Status::User => "user error",
            Status::NotUTF8 => "invalid utf8 data was passed to rust",
            Status::ImpossibleInstruction => "requested impossible instruction",
            _ => "unknown error",
        }
    }

    /// Turns the status into a result.
    pub fn as_result(self) -> Result {
        if self.is_error() {
            Err(self)
        } else {
            Ok(())
        }
    }
}

impl From<Status> for Result {
    fn from(x: Status) -> Self {
        x.as_result()
    }
}

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Status {
    fn description(&self) -> &str {
        // Call method not defined in this trait.
        Self::description(*self)
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

macro_rules! check_string {
    ($expression:expr) => {{
        use core::ffi::CStr;

        match $expression {
            x if x.is_null() => None,
            x => Some(CStr::from_ptr(x).to_str().unwrap()),
        }
    }};
}
