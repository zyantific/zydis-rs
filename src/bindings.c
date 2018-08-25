// This makes it so much nicer to use these from Rust.
enum Status {
  Success = ZYAN_STATUS_SUCCESS,
  Failed = ZYAN_STATUS_FAILED,
  // ...
  True = ZYAN_STATUS_TRUE,
  False = ZYAN_STATUS_FALSE,
  InvalidArgument = ZYAN_STATUS_INVALID_ARGUMENT,
  InvalidOperation = ZYAN_STATUS_INVALID_OPERATION,
  NotFound = ZYAN_STATUS_NOT_FOUND,
  OutOfBounds = ZYAN_STATUS_OUT_OF_RANGE,
  InsufficientBufferSize = ZYAN_STATUS_INSUFFICIENT_BUFFER_SIZE,
  OutOfMemory = ZYAN_STATUS_NOT_ENOUGH_MEMORY,
  BadSystemcall = ZYAN_STATUS_BAD_SYSTEMCALL,

  // Zydis specific
  NoMoreData = ZYDIS_STATUS_NO_MORE_DATA,
  DecodingError = ZYDIS_STATUS_DECODING_ERROR,
  InstructionTooLong = ZYDIS_STATUS_INSTRUCTION_TOO_LONG,
  BadRegister = ZYDIS_STATUS_BAD_REGISTER,
  IllegalLock = ZYDIS_STATUS_ILLEGAL_LOCK,
  IllegalLegacyPfx = ZYDIS_STATUS_ILLEGAL_LEGACY_PFX,
  IllegalRex = ZYDIS_STATUS_ILLEGAL_REX,
  InvalidMap = ZYDIS_STATUS_INVALID_MAP,
  MalformedEvex = ZYDIS_STATUS_MALFORMED_EVEX,
  MalformedMvex = ZYDIS_STATUS_MALFORMED_MVEX,
  InvalidMask = ZYDIS_STATUS_INVALID_MASK,

  // Zydis formatter
  SkipToken = ZYDIS_STATUS_SKIP_TOKEN,

  User = ZYAN_MAKE_STATUS(1, ZYAN_MODULE_USER, 0x00),

  // Don't use this, it's only used so that you have to
  // add a `_` in all match statements in Rust.
  __NoExhaustiveMatching__ = 0xFFFFFFFF,
};

