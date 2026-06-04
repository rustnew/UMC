use crate::UmcError;

/// Parsed parts of a namespaced extension key.
struct ExtensionKeyParts {
    format_name: String,
    field_path: String,
}

/// Validate a namespaced key: `"FORMAT@VERSION/field.path"`
fn parse_key(key: &str) -> Result<ExtensionKeyParts, UmcError> {
    if key.len() > 512 {
        return Err(UmcError::InvalidExtensionKey {
            key: key.to_string(),
            reason: "Key too long (max 512 characters)".into(),
        });
    }
    let at = key.find('@').ok_or_else(|| UmcError::InvalidExtensionKey {
        key: key.to_string(),
        reason: "Key must contain '@' for namespace: FORMAT@VERSION/path".into(),
    })?;
    let slash = key.find('/').ok_or_else(|| UmcError::InvalidExtensionKey {
        key: key.to_string(),
        reason: "Key must contain '/' for path: FORMAT@VERSION/path".into(),
    })?;
    if slash <= at {
        return Err(UmcError::InvalidExtensionKey {
            key: key.to_string(),
            reason: "'/' must come after '@'".into(),
        });
    }
    if !key.chars().all(|c| c.is_alphanumeric() || "@/._-".contains(c)) {
        return Err(UmcError::InvalidExtensionKey {
            key: key.to_string(),
            reason: "Only alphanumeric and @/._- characters are allowed".into(),
        });
    }
    Ok(ExtensionKeyParts {
        format_name: key[..at].to_string(),
        field_path: key[slash + 1..].to_string(),
    })
}

/// Per-format extension blob storage.
#[derive(Debug, Clone, Default)]
pub struct FormatExtension {
    pub format_name: String,
    pub format_version: String,
    pub custom_fields: indexmap::IndexMap<String, Vec<u8>>,
    pub original_hash: Option<String>,
}

/// Zero-information-loss opaque extension storage.
///
/// All fields not representable natively in the IR are stored here
/// and restored verbatim on round-trip.
///
/// Keys MUST be namespaced: `"FORMAT@VERSION/field.path"`
/// Examples:
/// - `"GGUF@v3/tokenizer.chat_template"`
/// - `"ONNX@opset21/custom_metadata/key"`
#[derive(Debug, Clone)]
pub struct ExtensionStore {
    format_extensions: std::collections::HashMap<String, FormatExtension>,
    total_bytes: usize,
    max_bytes: usize,
}

impl Default for ExtensionStore {
    fn default() -> Self {
        Self::new(100 * 1024 * 1024)  // 100 MiB
    }
}

impl ExtensionStore {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            format_extensions: std::collections::HashMap::new(),
            total_bytes: 0,
            max_bytes,
        }
    }

    /// Store a blob under a namespaced key.
    pub fn set(&mut self, key: &str, value: Vec<u8>) -> Result<(), UmcError> {
        let parts = parse_key(key)?;
        let new_total = self.total_bytes.saturating_add(value.len());
        if new_total > self.max_bytes {
            return Err(UmcError::ExtensionStoreFull {
                current_bytes: self.total_bytes,
                max_bytes: self.max_bytes,
                tried_to_add: value.len(),
            });
        }
        let ext = self
            .format_extensions
            .entry(parts.format_name)
            .or_insert_with(FormatExtension::default);
        // Subtract old value size if key already exists
        self.total_bytes = self.total_bytes
            .saturating_sub(ext.custom_fields.get(&parts.field_path).map_or(0, |v| v.len()))
            .saturating_add(value.len());
        ext.custom_fields.insert(parts.field_path, value);
        Ok(())
    }

    /// Retrieve a blob by namespaced key.
    pub fn get(&self, key: &str) -> Option<&[u8]> {
        let parts = parse_key(key).ok()?;
        self.format_extensions
            .get(&parts.format_name)
            .and_then(|ext| ext.custom_fields.get(&parts.field_path))
            .map(|v| v.as_slice())
    }

    /// Get all extensions for a specific format.
    pub fn get_all_for_format(&self, format_name: &str) -> Option<&FormatExtension> {
        self.format_extensions.get(format_name)
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    pub fn usage_percent(&self) -> f64 {
        self.total_bytes as f64 / self.max_bytes as f64 * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let mut store = ExtensionStore::default();
        store
            .set("GGUF@v3/tokenizer.chat_template", b"hello".to_vec())
            .unwrap();
        assert_eq!(
            store.get("GGUF@v3/tokenizer.chat_template"),
            Some(b"hello".as_slice())
        );
    }

    #[test]
    fn test_missing_key() {
        let store = ExtensionStore::default();
        assert_eq!(store.get("GGUF@v3/missing"), None);
    }

    #[test]
    fn test_invalid_key_no_at() {
        let mut store = ExtensionStore::default();
        let err = store.set("GGUFv3/field", b"data".to_vec());
        assert!(matches!(err, Err(UmcError::InvalidExtensionKey { .. })));
    }

    #[test]
    fn test_invalid_key_slash_before_at() {
        let mut store = ExtensionStore::default();
        let err = store.set("GGUF/v3@field", b"data".to_vec());
        assert!(matches!(err, Err(UmcError::InvalidExtensionKey { .. })));
    }

    #[test]
    fn test_size_limit() {
        let mut store = ExtensionStore::new(10);
        let err = store.set("FMT@v1/big", vec![0u8; 11]);
        assert!(matches!(err, Err(UmcError::ExtensionStoreFull { .. })));
    }

    #[test]
    fn test_overwrite_same_key() {
        let mut store = ExtensionStore::default();
        store.set("GGUF@v3/key", b"hello".to_vec()).unwrap();
        store.set("GGUF@v3/key", b"world!!".to_vec()).unwrap();
        // byte count should reflect new value, not cumulative
        assert_eq!(store.total_bytes(), 7);
    }
}
