use thiserror::Error;

/// Central error type for all UMC operations.
/// Every error includes: what happened, where, and how to fix it.
#[derive(Debug, Error)]
pub enum UmcError {
    // ── I/O ────────────────────────────────────────────────────────────────
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Cannot memory-map '{context}': {msg}")]
    Mmap { context: String, msg: String },

    #[error("Atomic rename failed from '{src}' to '{dst}': {msg}")]
    AtomicRename {
        src: String,
        dst: String,
        msg: String,
    },

    // ── Format detection ───────────────────────────────────────────────────
    #[error(
        "Unknown format for '{path}'.\n\
         UMC does not recognise the magic bytes / extension.\n\
         Hint: {hint}"
    )]
    UnknownFormat { path: String, hint: String },

    #[error("Format '{format}' version '{version}' is not supported by this UMC build")]
    UnsupportedFormatVersion { format: String, version: String },

    // ── Magic bytes / parsing ──────────────────────────────────────────────
    #[error(
        "Invalid magic bytes in '{path}'.\n\
         Expected {expected:?}, found {found:?}.\n\
         The file may be corrupted or truncated."
    )]
    InvalidMagic {
        path: String,
        expected: Vec<u8>,
        found: Vec<u8>,
    },

    #[error("File '{path}' appears truncated: expected at least {expected} bytes, got {actual}")]
    FileTruncated {
        path: String,
        expected: usize,
        actual: usize,
    },

    #[error("Unexpected EOF while reading '{context}' at offset {offset}")]
    UnexpectedEof { context: String, offset: u64 },

    // ── Tensor errors ──────────────────────────────────────────────────────
    #[error(
        "Tensor '{name}' is out of bounds.\n\
         Declared offset {offset} + length {length} = {end} exceeds file size {file_size}.\n\
         The file is likely corrupted or truncated."
    )]
    TensorOutOfBounds {
        name: String,
        offset: u64,
        length: usize,
        end: u64,
        file_size: u64,
    },

    #[error("Checksum mismatch for '{context}': expected 0x{expected:016x}, got 0x{actual:016x}")]
    ChecksumMismatch {
        context: String,
        expected: u64,
        actual: u64,
    },

    #[error("Tensor '{0}' has not been materialised yet (still Lazy). Call materialize() first.")]
    NotMaterialized(String),

    #[error("Tensor '{0}' is a shared reference; resolve via TensorStore::resolve_shared().")]
    IsReference(String),

    #[error("Missing shared-tensor target '{0}' referenced by another tensor")]
    MissingSharedTensor(String),

    #[error("Invalid tensor name '{0}': contains null bytes or exceeds maximum length")]
    InvalidTensorName(String),

    #[error("Unsupported bit-width {0} in quantized tensor")]
    UnsupportedBitWidth(u8),

    // ── Security / DoS prevention ──────────────────────────────────────────
    #[error(
        "Security limit exceeded for field '{field}': value {value} > limit {limit}.\n\
         This may indicate a malformed or malicious file."
    )]
    SecurityViolation {
        field: String,
        value: usize,
        limit: usize,
    },

    #[error(
        "Suspicious value for '{field}': {value}.\n\
         This exceeds reasonable bounds and may indicate a malicious file."
    )]
    SuspiciousValue { field: String, value: u64 },

    // ── ExtensionStore ─────────────────────────────────────────────────────
    #[error(
        "ExtensionStore is full: {current_bytes} bytes used, limit {max_bytes}.\n\
         Tried to add {tried_to_add} more bytes."
    )]
    ExtensionStoreFull {
        current_bytes: usize,
        max_bytes: usize,
        tried_to_add: usize,
    },

    #[error("Invalid ExtensionStore key '{key}': {reason}")]
    InvalidExtensionKey { key: String, reason: String },

    // ── Conversion ─────────────────────────────────────────────────────────
    #[error(
        "No conversion path found from '{from}' to '{to}'.\n\
         Supported formats: {available}"
    )]
    NoConversionPath {
        from: String,
        to: String,
        available: String,
    },

    #[error(
        "Operator '{op_type}' (domain '{domain}') is not supported by format '{target}'.\n\
         It has been preserved in the ExtensionStore for round-trip, \
         but the output cannot be executed."
    )]
    UnsupportedOp {
        op_type: String,
        domain: String,
        target: String,
    },

    #[error("DType {dtype} is not supported by format '{target}'. Automatic conversion to {suggested} applied.")]
    DTypeNotSupported {
        dtype: String,
        target: String,
        suggested: String,
    },

    #[error(
        "Conversion from '{from}' to '{to}' requires the '{tool}' tool.\n\
             Install it at: {install_url}"
    )]
    ExternalToolRequired {
        from: String,
        to: String,
        tool: String,
        install_url: String,
    },

    // ── Validation ─────────────────────────────────────────────────────────
    #[error("Structural validation failed: {reason}")]
    StructuralValidationFailed { reason: String },

    #[error(
        "Numeric validation failed for tensor '{tensor}'.\n\
         Max divergence: {max_divergence:.2e} (threshold: {threshold:.2e}).\n\
         The conversion may have introduced numerical errors."
    )]
    NumericValidationFailed {
        tensor: String,
        max_divergence: f64,
        threshold: f64,
    },

    // ── Pipeline ───────────────────────────────────────────────────────────
    #[error("Thread '{thread}' panicked during conversion")]
    ThreadPanic { thread: String },

    #[error("Channel timeout on '{channel}' — possible deadlock or stagnation detected")]
    ChannelTimeout { channel: String },

    #[error("Conversion was cancelled by user request")]
    Cancelled,

    // ── General ────────────────────────────────────────────────────────────
    #[error("Protobuf decode error: {0}")]
    ProtobufDecode(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("UTF-8 decode error at '{context}': {msg}")]
    Utf8 { context: String, msg: String },

    #[error("Integer overflow computing {context}")]
    IntOverflow { context: String },

    #[error("{0}")]
    Other(String),
}

impl UmcError {
    /// Return true if this error is recoverable (conversion can continue with a warning).
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::UnsupportedOp { .. } | Self::DTypeNotSupported { .. }
        )
    }
}
