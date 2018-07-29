//! Status code utilities.

use gen::*;

// A Zydis result, holding either a result or a failure code.
pub type ZydisResult<T> = Result<T, ZydisStatus>;

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
        match $expression as _ {
            x if x == ZYDIS_STATUS_SUCCESS => Ok($ok),
            e => Err(e),
        }
    };
}

macro_rules! check_option {
    // This should only be used for the `ZydisDecoderDecodeBuffer` function.
    ($expression:expr, $ok:expr) => {
        match $expression as _ {
            x if x == ZYDIS_STATUS_SUCCESS => Ok(Some($ok)),
            x if x == ZYDIS_STATUS_NO_MORE_DATA => Ok(None),
            e => Err(e),
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
