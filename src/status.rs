//! Status code utilities.

use gen::ZydisStatusCode;


// A Zydis result, holding either a result or a failure code.
pub type ZydisResult<T> = Result<T, ZydisStatusCode>;