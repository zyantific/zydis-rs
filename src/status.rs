//! Status code utilities.

use gen::ZydisStatus;

// A Zydis result, holding either a result or a failure code.
pub type ZydisResult<T> = Result<T, ZydisStatus>;

#[macro_export]
macro_rules! check {
    ($expression:expr, $ok:expr) => {
        match $expression as _ {
            ZYDIS_STATUS_SUCCESS => Ok($ok),
            e => Err(e),
        }
    };
}

macro_rules! check_option {
    // This should only be used for the `ZydisDecoderDecodeBuffer` function.
    ($expression:expr, $ok:expr) => {
        match $expression as _ {
            ZYDIS_STATUS_SUCCESS => Ok(Some($ok)),
            ZYDIS_STATUS_NO_MORE_DATA => Ok(None),
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
