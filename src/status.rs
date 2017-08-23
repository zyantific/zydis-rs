//! Status code utilities.

use gen::ZydisStatusCode;

// A Zydis result, holding either a result or a failure code.
pub type ZydisResult<T> = Result<T, ZydisStatusCode>;

#[macro_export]
macro_rules! check {
    ($expression:expr, $ok:expr) => {
        match $expression as _ {
            ZYDIS_STATUS_SUCCESS => Ok($ok),
            e => Err(e),
        }
    };
    // This should only be used for the `ZydisDecoderDecodeBuffer` function.
    (@option $expression:expr, $ok:expr) => {
        match $expression as _ {
            ZYDIS_STATUS_SUCCESS => Ok(Some($ok)),
            ZYDIS_STATUS_NO_MORE_DATA => Ok(None),
            e => Err(e),
        }
    };
    (@string $expression:expr) => { {
            use std::ptr::null;
            match $expression {
                x if x == null() => None,
                x => Some(CStr::from_ptr(x).to_str().unwrap())
            }
        }
    }
}
