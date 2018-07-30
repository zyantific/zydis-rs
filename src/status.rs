//! Status code utilities.

use std::{error, fmt, result};

use gen::*;

// A Zydis result, holding either a result or a failure code.
pub type Result<T> = result::Result<T, ZydisError>;

/// A type that implements std::error::Error and thus is useable with the failure crate.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ZydisError {
    x: ZydisStatusCodes,
}

impl fmt::Display for ZydisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", status_description(self.x))
    }
}

impl ZydisError {
    pub fn new(x: ZydisStatusCodes) -> ZydisError {
        Self { x }
    }

    pub fn get_code(&self) -> ZydisStatusCodes {
        self.x
    }
}

impl From<ZydisStatusCodes> for ZydisError {
    fn from(x: ZydisStatusCodes) -> Self {
        Self { x }
    }
}

impl error::Error for ZydisError {
    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }

    fn description(&self) -> &str {
        status_description(self.x)
    }
}

pub fn status_description(status: ZydisStatus) -> &'static str {
    match status {
        x if x == ZYDIS_STATUS_SUCCESS => "no error",
        x if x == ZYDIS_STATUS_INVALID_PARAMETER => "An invalid parameter was passed to a function.",
        x if x == ZYDIS_STATUS_INVALID_OPERATION => "An attempt was made to perform an invalid operation.",
        x if x == ZYDIS_STATUS_INSUFFICIENT_BUFFER_SIZE => "A buffer passed to a function was too small to complete the requested operation.",
        x if x == ZYDIS_STATUS_NO_MORE_DATA => "An attempt was made to read data from an input data-source that has no more data available.",
        x if x == ZYDIS_STATUS_DECODING_ERROR => "An general error occured while decoding the current instruction. The instruction might be undfined.",
        x if x == ZYDIS_STATUS_INSTRUCTION_TOO_LONG => "The instruction exceeded the maximum length of 15 bytes.",
        x if x == ZYDIS_STATUS_BAD_REGISTER => "The instruction encoded an invalid register.",
        x if x == ZYDIS_STATUS_ILLEGAL_LOCK => "A lock-prefix (F0) was found while decoding an instruction that does not support locking.",
        x if x == ZYDIS_STATUS_ILLEGAL_LEGACY_PFX => "A legacy-prefix (F2, F3, 66) was found while decoding a XOP/VEX/EVEX/MVEX instruction.",
        x if x == ZYDIS_STATUS_ILLEGAL_REX => "A rex-prefix was found while decoding a XOP/VEX/EVEX/MVEX instruction.",
        x if x == ZYDIS_STATUS_INVALID_MAP => "An invalid opcode-map value was found while decoding a XOP/VEX/EVEX/MVEX-prefix.",
        x if x == ZYDIS_STATUS_MALFORMED_EVEX => "An error occured while decoding the EVEX-prefix.",
        x if x == ZYDIS_STATUS_MALFORMED_MVEX => "An error occured while decoding the MVEX-prefix.",
        x if x == ZYDIS_STATUS_INVALID_MASK => "An invalid write-mask was specified for an EVEX/MVEX instruction.",
        _ => "unknown/user defined error"
    }
}

#[macro_export]
macro_rules! check {
    ($expression:expr, $ok:expr) => {
        match $expression as ZydisStatusCodes {
            x if x == ZYDIS_STATUS_SUCCESS => Ok($ok),
            e => Err(ZydisError::from(e)),
        }
    };
}

macro_rules! check_option {
    // This should only be used for the `ZydisDecoderDecodeBuffer` function.
    ($expression:expr, $ok:expr) => {
        match $expression as ZydisStatusCodes {
            x if x == ZYDIS_STATUS_SUCCESS => Ok(Some($ok)),
            x if x == ZYDIS_STATUS_NO_MORE_DATA => Ok(None),
            e => Err(ZydisError::from(e)),
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
