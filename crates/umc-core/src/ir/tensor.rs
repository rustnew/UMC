use std::sync::Arc;
use memmap2::Mmap;
use xxhash_rust::xxh64::xxh64;
use crate::{DType, UmcError};
use super::quantization::TensorQuantization;

// ── SecurityBounds ────────────────────────────────────────────────────────────

/// Hard limits validated on every tensor insertion to prevent DoS via malformed files.
#[derive(Debug, Clone)]
pub struct SecurityBounds {
    pub max_tensor_count: usize,
    pub max_metadata_count: usize,
    pub max_string_length: usize,
    pub max_shape_rank: usize,
    pub max_tensor_size_bytes: usize,
    pub max_extension_bytes: usize,
}

impl Default for SecurityBounds {
    fn default() -> Self {
        Self {
            max_tensor_count: 1_000_000,
            max_metadata_count: 10_000,
            max_string_length: 1_048_576,  // 1 MiB
            max_shape_rank: 8,
            max_tensor_size_bytes: 100 * 1024 * 1024 * 1024,  // 100 GiB per tensor
            max_extension_bytes: 100 * 1024 * 1024,            // 100 MiB total extensions
        }
    }
}

// ── Layout ────────────────────────────────────────────────────────────────────

/// Memory layout of a tensor.
#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    CContiguous,   // Row-major (NumPy, PyTorch default)
    FContiguous,   // Column-major (Fortran, some BLAS)
    Custom,        // Custom strides (see Tensor.strides)
}

// ── TensorData ────────────────────────────────────────────────────────────────

/// Tensor data storage — zero-copy via mmap for large tensors.
#[derive(Debug, Clone)]
pub enum TensorData {
    /// Direct mmap view — ZERO copy, the OS manages caching.
    MmapView {
        mmap: Arc<Mmap>,
        offset: usize,
        length: usize,
    },
    /// In-RAM data (small tensors or results of transformation).
    Owned(Arc<Vec<u8>>),
    /// Lazy load — materialised on demand with checksum verification.
    Lazy {
        file_path: std::path::PathBuf,
        offset: u64,
        length: usize,
        checksum: u64,
    },
    /// Tied-weight reference to another tensor (e.g., embed_tokens == lm_head).
    Shared {
        target_name: String,
        transforms: Vec<TensorTransform>,
    },
}

/// Lightweight transformations that can be applied to a shared tensor.
#[derive(Debug, Clone)]
pub enum TensorTransform {
    Transpose(Vec<usize>),
    Slice { axis: usize, start: usize, end: usize },
}

impl TensorData {
    /// Byte slice view — never copies for MmapView.
    pub fn as_bytes(&self) -> Result<&[u8], UmcError> {
        match self {
            Self::MmapView { mmap, offset, length } => {
                Ok(&mmap[*offset..*offset + *length])
            }
            Self::Owned(data) => Ok(data.as_slice()),
            Self::Lazy { .. } => Err(UmcError::NotMaterialized("lazy tensor".into())),
            Self::Shared { target_name, .. } => Err(UmcError::IsReference(target_name.clone())),
        }
    }

    /// Byte length of the data.
    pub fn len(&self) -> usize {
        match self {
            Self::MmapView { length, .. } => *length,
            Self::Owned(v) => v.len(),
            Self::Lazy { length, .. } => *length,
            Self::Shared { .. } => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Compute xxHash64 checksum of the raw bytes.
    pub fn compute_checksum(&self) -> Result<u64, UmcError> {
        let bytes = self.as_bytes()?;
        Ok(xxh64(bytes, 0))
    }

    /// Materialise a Lazy tensor into a MmapView, verifying its checksum.
    pub fn materialize(&mut self) -> Result<(), UmcError> {
        if let Self::Lazy { file_path, offset, length, checksum } = self {
            let path_str = file_path.display().to_string();
            let file = std::fs::File::open(&*file_path).map_err(UmcError::Io)?;
            let mmap = Arc::new(unsafe {
                Mmap::map(&file).map_err(|e| UmcError::Mmap {
                    context: path_str.clone(),
                    msg: e.to_string(),
                })?
            });
            let start = *offset as usize;
            let end = start + *length;
            let saved_length = *length;
            let saved_checksum = *checksum;
            let actual = xxh64(&mmap[start..end], 0);
            if actual != saved_checksum {
                return Err(UmcError::ChecksumMismatch {
                    context: path_str,
                    expected: saved_checksum,
                    actual,
                });
            }
            *self = Self::MmapView {
                mmap,
                offset: start,
                length: saved_length,
            };
        }
        Ok(())
    }
}

// ── Tensor ────────────────────────────────────────────────────────────────────

/// A single tensor in the IR.
#[derive(Debug, Clone)]
pub struct Tensor {
    pub name: String,
    pub dtype: DType,
    pub shape: Vec<usize>,
    /// None means C-contiguous (row-major).
    pub strides: Option<Vec<usize>>,
    pub layout: Layout,
    pub data: TensorData,
    /// xxHash64 of raw bytes — computed at load time, verified after conversion.
    pub checksum: u64,
    pub quantization: Option<TensorQuantization>,
}

impl Tensor {
    /// Create a new tensor from owned bytes.
    pub fn from_bytes(
        name: impl Into<String>,
        dtype: DType,
        shape: Vec<usize>,
        data: Vec<u8>,
    ) -> Self {
        let checksum = xxh64(&data, 0);
        Self {
            name: name.into(),
            dtype,
            shape,
            strides: None,
            layout: Layout::CContiguous,
            data: TensorData::Owned(Arc::new(data)),
            checksum,
            quantization: None,
        }
    }

    /// Create a tensor backed by a mmap region.
    pub fn from_mmap(
        name: impl Into<String>,
        dtype: DType,
        shape: Vec<usize>,
        mmap: Arc<Mmap>,
        offset: usize,
        length: usize,
    ) -> Self {
        let checksum = xxh64(&mmap[offset..offset + length], 0);
        Self {
            name: name.into(),
            dtype,
            shape,
            strides: None,
            layout: Layout::CContiguous,
            data: TensorData::MmapView { mmap, offset, length },
            checksum,
            quantization: None,
        }
    }

    /// Number of elements (product of shape dimensions).
    pub fn num_elements(&self) -> usize {
        if self.shape.is_empty() {
            1  // scalar
        } else {
            self.shape.iter().product()
        }
    }

    /// Byte size, computed from dtype and shape.
    pub fn byte_size(&self) -> Option<usize> {
        let bpe = self.dtype.bytes_per_element()?;
        Some((self.num_elements() as f64 * bpe).ceil() as usize)
    }
}

// ── TensorStore ───────────────────────────────────────────────────────────────

/// Ordered tensor storage with security validation on every insertion.
#[derive(Debug, Clone)]
pub struct TensorStore {
    tensors: indexmap::IndexMap<String, Tensor>,
    ram_usage_bytes: usize,
    pub mmap_threshold_bytes: usize,
    bounds: SecurityBounds,
}

impl TensorStore {
    pub fn new() -> Self {
        Self {
            tensors: indexmap::IndexMap::new(),
            ram_usage_bytes: 0,
            mmap_threshold_bytes: 64 * 1024 * 1024,  // 64 MiB threshold
            bounds: SecurityBounds::default(),
        }
    }

    pub fn with_bounds(bounds: SecurityBounds) -> Self {
        Self { bounds, ..Self::new() }
    }

    /// Insert a tensor with full security validation.
    pub fn insert(&mut self, tensor: Tensor) -> Result<(), UmcError> {
        if self.tensors.len() >= self.bounds.max_tensor_count {
            return Err(UmcError::SecurityViolation {
                field: "tensor_count".into(),
                value: self.tensors.len(),
                limit: self.bounds.max_tensor_count,
            });
        }
        if tensor.shape.len() > self.bounds.max_shape_rank {
            return Err(UmcError::SecurityViolation {
                field: "shape_rank".into(),
                value: tensor.shape.len(),
                limit: self.bounds.max_shape_rank,
            });
        }
        if tensor.data.len() > self.bounds.max_tensor_size_bytes {
            return Err(UmcError::SecurityViolation {
                field: "tensor_size_bytes".into(),
                value: tensor.data.len(),
                limit: self.bounds.max_tensor_size_bytes,
            });
        }
        if tensor.name.len() > self.bounds.max_string_length {
            return Err(UmcError::SecurityViolation {
                field: "tensor_name_length".into(),
                value: tensor.name.len(),
                limit: self.bounds.max_string_length,
            });
        }
        if tensor.name.contains('\0') {
            return Err(UmcError::InvalidTensorName(tensor.name.clone()));
        }

        if let TensorData::Owned(ref v) = tensor.data {
            self.ram_usage_bytes = self.ram_usage_bytes.saturating_add(v.len());
        }
        self.tensors.insert(tensor.name.clone(), tensor);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Tensor> {
        self.tensors.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Tensor> {
        self.tensors.get_mut(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Tensor)> {
        self.tensors.iter()
    }

    pub fn len(&self) -> usize {
        self.tensors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tensors.is_empty()
    }

    pub fn ram_usage_mb(&self) -> f64 {
        self.ram_usage_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Resolve a Shared tensor reference to the concrete target tensor.
    pub fn resolve_shared<'a>(&'a self, tensor: &'a Tensor) -> Result<&'a Tensor, UmcError> {
        match &tensor.data {
            TensorData::Shared { target_name, .. } => self
                .tensors
                .get(target_name)
                .ok_or_else(|| UmcError::MissingSharedTensor(target_name.clone())),
            _ => Ok(tensor),
        }
    }

    /// Names of all tensors in insertion order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.tensors.keys().map(|s| s.as_str())
    }
}

impl Default for TensorStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tensor(name: &str, shape: Vec<usize>, size: usize) -> Tensor {
        Tensor::from_bytes(name, DType::F32, shape, vec![0u8; size])
    }

    #[test]
    fn test_insert_and_get() {
        let mut store = TensorStore::new();
        let t = make_tensor("weight", vec![4, 4], 64);
        store.insert(t).unwrap();
        assert_eq!(store.len(), 1);
        assert!(store.get("weight").is_some());
        assert!(store.get("missing").is_none());
    }

    #[test]
    fn test_security_tensor_count() {
        let mut bounds = SecurityBounds::default();
        bounds.max_tensor_count = 2;
        let mut store = TensorStore::with_bounds(bounds);
        store.insert(make_tensor("a", vec![1], 4)).unwrap();
        store.insert(make_tensor("b", vec![1], 4)).unwrap();
        let err = store.insert(make_tensor("c", vec![1], 4));
        assert!(matches!(err, Err(UmcError::SecurityViolation { field, .. }) if field == "tensor_count"));
    }

    #[test]
    fn test_security_shape_rank() {
        let mut bounds = SecurityBounds::default();
        bounds.max_shape_rank = 3;
        let mut store = TensorStore::with_bounds(bounds);
        let t = make_tensor("deep", vec![1, 2, 3, 4], 96);
        let err = store.insert(t);
        assert!(matches!(err, Err(UmcError::SecurityViolation { field, .. }) if field == "shape_rank"));
    }

    #[test]
    fn test_null_byte_in_name() {
        let mut store = TensorStore::new();
        let mut t = make_tensor("bad\0name", vec![1], 4);
        t.name = "bad\0name".into();
        let err = store.insert(t);
        assert!(matches!(err, Err(UmcError::InvalidTensorName(_))));
    }

    #[test]
    fn test_tensor_num_elements() {
        let t = make_tensor("w", vec![3, 4, 5], 0);
        assert_eq!(t.num_elements(), 60);
    }

    #[test]
    fn test_checksum_computed() {
        let data = vec![1u8, 2, 3, 4];
        let t = Tensor::from_bytes("x", DType::F32, vec![1], data.clone());
        let expected = xxh64(&data, 0);
        assert_eq!(t.checksum, expected);
    }
}
