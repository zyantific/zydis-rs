//! Status code utilities.

use core::{fmt, result};

use std::error;

use gen::*;

/// A Result, holding either a value or a failure code.
pub type Result<T> = result::Result<T, Error>;

/// A type that implements std::error::Error and thus is useable with the
/// failure crate.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Error {
    x: Status,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", status_description(self.x))
    }
}

impl Error {
    pub fn new(x: Status) -> Error {
        Self { x }
    }

    pub fn get_code(self) -> Status {
        self.x
    }
}

impl From<ZyanStatus> for Error {
    fn from(x: ZyanStatus) -> Self {
        Self { x: x.into() }
    }
}

impl From<Status> for Error {
    fn from(x: Status) -> Self {
        Self { x }
    }
}

#[cfg(not(no_std))]
impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }

    fn description(&self) -> &str {
        status_description(self.x)
    }
}

pub fn status_description(status: Status) -> &'static str {
    match status {
        Status::Success => "no error",
        Status::Failed => "A operation failed.",
        Status::InvalidArgument => "An invalid parameter was passed to a function.",
        Status::InvalidOperation => "An attempt was made to perform an invalid operation.",
        Status::InsufficientBufferSize => {
            "A buffer passed to a function was too small to complete the requested operation."
        }
        Status::OutOfBounds => "An index was out of bounds.",
        Status::NotFound => "The requested entity was not found.",
        Status::OutOfMemory => "Insufficient memory to perform the operation.",
        Status::BadSystemcall => "An error occured during a system call.",
        Status::NoMoreData => {
            "An attempt was made to read data from an input data-source that has no more data \
             available."
        }
        Status::DecodingError => {
            "An general error occured while decoding the current instruction. The instruction \
             might be undfined."
        }
        Status::InstructionTooLong => "The instruction exceeded the maximum length of 15 bytes.",
        Status::BadRegister => "The instruction encoded an invalid register.",
        Status::IllegalLock => {
            "A lock-prefix (F0) was found while decoding an instruction that does not support \
             locking."
        }
        Status::IllegalLegacyPfx => {
            "A legacy-prefix (F2, F3, 66) was found while decoding a XOP/VEX/EVEX/MVEX instruction."
        }
        Status::IllegalRex => {
            "A rex-prefix was found while decoding a XOP/VEX/EVEX/MVEX instruction."
        }
        Status::InvalidMap => {
            "An invalid opcode-map value was found while decoding a XOP/VEX/EVEX/MVEX-prefix."
        }
        Status::MalformedEvex => "An error occured while decoding the EVEX-prefix.",
        Status::MalformedMvex => "An error occured while decoding the MVEX-prefix.",
        Status::InvalidMask => "An invalid write-mask was specified for an EVEX/MVEX instruction.",
        Status::True | Status::False => "true/false not an error",
        Status::SkipToken => "skip this token",
        Status::User => "user error",
        _ => "unknown error",
    }
}

#[macro_export]
macro_rules! check {
    ($expression:expr) => {
        check!($expression, ())
    };
    ($expression:expr, $ok:expr) => {
        match $expression.into() {
            $crate::gen::Status::Success => Ok($ok),
            e => Err($crate::status::Error::from(e)),
        }
    };
}

macro_rules! check_option {
    // This should only be used for the `ZydisDecoderDecodeBuffer` function.
    ($expression:expr, $ok:expr) => {
        match $expression.into() {
            $crate::gen::Status::Success => Ok(Some($ok)),
            $crate::gen::Status::NoMoreData => Ok(None),
            e => Err($crate::status::Error::from(e)),
        }
    };
}

macro_rules! check_string {
    ($expression:expr) => {{
        match $expression {
            x if x.is_null() => None,
            x => Some(CStr::from_ptr(x).to_str().unwrap()),
        }
    }};
}
