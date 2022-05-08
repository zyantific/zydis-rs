use super::*;

pub type ZyanStringFlags = u8;

/// The string type used in zydis.
#[derive(Debug)]
#[repr(C)]
pub struct ZyanString {
    flags: ZyanStringFlags,
    vector: ZyanVector,
}

impl ZyanString {
    /// Create a new `ZyanString`, using the given `buffer` for storage.
    #[inline]
    pub fn new(buffer: &mut [u8]) -> Result<Self> {
        Self::new_ptr(buffer.as_mut_ptr(), buffer.len())
    }

    /// Create a new `ZyanString` from a given buffer and a capacity.
    #[inline]
    pub fn new_ptr(buffer: *mut u8, capacity: usize) -> Result<Self> {
        unsafe {
            let mut string = MaybeUninit::uninit();
            check!(ZyanStringInitCustomBuffer(
                string.as_mut_ptr(),
                buffer as *mut c_char,
                capacity
            ))?;
            Ok(string.assume_init())
        }
    }

    /// Appends the given string `s` to this buffer.
    ///
    /// Warning: The actual Rust `&str`ings are encoded in UTF-8 and aren't
    /// converted to any other encoding. They're simply copied, byte by
    /// byte, to the buffer. Therefore, the buffer should be interpreted as
    /// UTF-8 when later being printed.
    #[inline]
    pub fn append<S: AsRef<str> + ?Sized>(&mut self, s: &S) -> Result<()> {
        unsafe {
            let bytes = s.as_ref().as_bytes();
            let view = ZyanStringView::new(bytes)?;
            check!(ZyanStringAppend(self, &view))
        }
    }
}

impl fmt::Write for ZyanString {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.append(s).map_err(|_| fmt::Error)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ZyanStringView {
    string: ZyanString,
}

impl ZyanStringView {
    /// Creates a string view from the given `buffer`.
    #[inline]
    pub fn new(buffer: &[u8]) -> Result<Self> {
        unsafe {
            let mut view = MaybeUninit::uninit();
            check!(ZyanStringViewInsideBufferEx(
                view.as_mut_ptr(),
                buffer.as_ptr() as *const c_char,
                buffer.len()
            ))?;
            Ok(view.assume_init())
        }
    }
}

#[derive(Debug)]
#[repr(C)]
struct ZyanVector {
    allocator: *mut c_void,
    growth_factor: f32,
    shrink_threshold: f32,
    size: usize,
    capacity: usize,
    element_size: usize,
    destructor: *mut c_void,
    data: *mut c_void,
}

extern "C" {
    pub fn ZyanStringInitCustomBuffer(
        string: *mut ZyanString,
        buffer: *mut c_char,
        capacity: usize,
    ) -> Status;

    pub fn ZyanStringAppend(destination: *mut ZyanString, source: *const ZyanStringView) -> Status;

    pub fn ZyanStringDestroy(string: *mut ZyanString) -> Status;

    pub fn ZyanStringViewInsideBufferEx(
        view: *mut ZyanStringView,
        buffer: *const c_char,
        length: usize,
    ) -> Status;
}
