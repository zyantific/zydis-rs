//! Status code utilities.

use core::{fmt, result};

use std::error;

/// A convenience alias for a Result, holding either a value or a status.
pub type Result<T> = result::Result<T, Status>;

const ZYAN_MODULE_ZYCORE: usize = 0x1;
const ZYAN_MODULE_ZYDIS: usize = 0x2;
const ZYAN_MODULE_USER: usize = 0x3FF;

macro_rules! make_status {
    ($error:expr, $module:expr, $code:expr) => {
        ((($error & 1) << 31) | (($module & 0x7FF) << 20) | ($code & 0xFFFFF)) as isize
    };
}

#[rustfmt::skip]
#[derive(Copy, Clone, Eq, PartialEq)]
// TODO: Once stable
//#[non_exhaustive]
#[repr(C)]
pub enum Status {
    Success                = make_status!(0, ZYAN_MODULE_ZYCORE, 0x00),
    Failed                 = make_status!(1, ZYAN_MODULE_ZYCORE, 0x01),
    True                   = make_status!(0, ZYAN_MODULE_ZYCORE, 0x02),
    False                  = make_status!(0, ZYAN_MODULE_ZYCORE, 0x03),
    InvalidArgument        = make_status!(1, ZYAN_MODULE_ZYCORE, 0x04),
    InvalidOperation       = make_status!(1, ZYAN_MODULE_ZYCORE, 0x05),
    NotFound               = make_status!(1, ZYAN_MODULE_ZYCORE, 0x06),
    OutOfRange             = make_status!(1, ZYAN_MODULE_ZYCORE, 0x07),
    InsufficientBufferSize = make_status!(1, ZYAN_MODULE_ZYCORE, 0x08),
    NotEnoughMemory        = make_status!(1, ZYAN_MODULE_ZYCORE, 0x09),
    BadSystemcall          = make_status!(1, ZYAN_MODULE_ZYCORE, 0x0A),

    // Zydis
    NoMoreData             = make_status!(1, ZYAN_MODULE_ZYDIS, 0x00),
    DecodingError          = make_status!(1, ZYAN_MODULE_ZYDIS, 0x01),
    InstructionTooLong     = make_status!(1, ZYAN_MODULE_ZYDIS, 0x02),
    BadRegister            = make_status!(1, ZYAN_MODULE_ZYDIS, 0x03),
    IllegalLock            = make_status!(1, ZYAN_MODULE_ZYDIS, 0x04),
    IllegalLegacyPfx       = make_status!(1, ZYAN_MODULE_ZYDIS, 0x05),
    IllegalRex             = make_status!(1, ZYAN_MODULE_ZYDIS, 0x06),
    InvalidMap             = make_status!(1, ZYAN_MODULE_ZYDIS, 0x07),
    MalformedEvex          = make_status!(1, ZYAN_MODULE_ZYDIS, 0x08),
    MalformedMvex          = make_status!(1, ZYAN_MODULE_ZYDIS, 0x09),
    InvalidMask            = make_status!(1, ZYAN_MODULE_ZYDIS, 0x0A),

    // Formatter
    /// Returning this status code from some formatter callback will cause the
    /// formatter to omit the corresponding token.
    ///
    /// Valid callbacks are:
    /// - `HookPreOperand`
    /// - `HookPostOperand`
    /// - `HookFormatOperandReg`
    /// - `HookFormatOperandMem`
    /// - `HookFormatOperandPtr`
    /// - `HookFormatOperandImm`
    /// - `HookPrintMemsize`
    SkipToken = make_status!(0, ZYAN_MODULE_ZYDIS, 0x0B),

    /// Use this for custom errors that don't fit for any of the other errors.
    User = make_status!(1, ZYAN_MODULE_USER, 0x00),
    /// The given bytes were not UTF8 encoded.
    NotUTF8 = make_status!(1, ZYAN_MODULE_USER, 0x01),

    // TODO: For now ...
    // Don't use this, it is used so that you always have a `_` in all match patterns, because
    // otherwise we could hit undefined behaviour.
    //
    // 0x7FF... so that this can fit within one isize.
    #[doc(hidden)]
    __NoExhaustiveMatching__ = 0x7FFFFFFF,
}

impl Status {
    /// Returns the error code of this status.
    pub fn code(self) -> usize {
        (self as usize) & 0xFFFFF
    }

    /// Returns the module of this status.
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
            Status::BadSystemcall => "An error occured during a system call.",
            Status::NoMoreData => {
                "An attempt was made to read data from an input data-source that has no more data \
                 available."
            }
            Status::DecodingError => {
                "An general error occured while decoding the current instruction. The instruction \
                 might be undfined."
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
            _ => "unknown error",
        }
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

impl error::Error for Status {
    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }

    fn description(&self) -> &str {
        // Call method not defined in this trait.
        Self::description(*self)
    }
}

#[macro_export]
macro_rules! check {
    ($expression:expr) => {
        check!($expression, ())
    };
    ($expression:expr, $ok:expr) => {
        match $expression {
            x if !x.is_error() => Ok($ok),
            x => Err(x),
        }
    };
}

macro_rules! check_option {
    // This should only be used for the `ZydisDecoderDecodeBuffer` function.
    ($expression:expr, $ok:expr) => {
        match $expression {
            x if !x.is_error() => Ok(Some($ok)),
            $crate::status::Status::NoMoreData => Ok(None),
            x => Err(x),
        }
    };
}

macro_rules! check_string {
    ($expression:expr) => {{
        use std::ffi::CStr;

        match $expression {
            x if x.is_null() => None,
            x => Some(CStr::from_ptr(x).to_str().unwrap()),
        }
    }};
}
