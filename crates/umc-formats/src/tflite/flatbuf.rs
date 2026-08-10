/// Minimal FlatBuffer reader for TFLite Model files.
/// Reads tensor names, shapes, types, and raw buffer data from a .tflite file.
///
/// FlatBuffer wire format (little-endian):
///   File[0..4]:  root_offset (u32) — offset from this field to the root table
///   File[4..8]:  file identifier "TFL3" / "TFL2" etc.
///   Root table at (root_offset + 0):
///     Table layout: soffset_t vtable_offset, then field data
///     vtable: vtable_size(u16), object_size(u16), field_offsets[i](u16)
use umc_core::{DType, UmcError};

// ── TFLite TensorType enum ────────────────────────────────────────────────────
fn tensor_type_to_dtype(t: i32) -> DType {
    match t {
        0 => DType::F32,
        1 => DType::F16,
        2 => DType::I32,
        3 => DType::U8,
        4 => DType::I64,
        6 => DType::Bool,
        7 => DType::I16,
        9 => DType::I8,
        16 => DType::BF16,
        _ => DType::F32,
    }
}

// ── Low-level FlatBuffer reader ───────────────────────────────────────────────

pub struct FlatBufReader<'a> {
    pub data: &'a [u8],
}

impl<'a> FlatBufReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn u8_at(&self, pos: usize) -> Option<u8> {
        self.data.get(pos).copied()
    }

    fn u16_at(&self, pos: usize) -> Option<u16> {
        let b = self.data.get(pos..pos + 2)?;
        Some(u16::from_le_bytes([b[0], b[1]]))
    }

    fn u32_at(&self, pos: usize) -> Option<u32> {
        let b = self.data.get(pos..pos + 4)?;
        Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn i32_at(&self, pos: usize) -> Option<i32> {
        self.u32_at(pos).map(|v| v as i32)
    }

    /// Read indirect u32 offset (offset relative to its own position).
    fn follow_u32_offset(&self, pos: usize) -> Option<usize> {
        let offset = self.u32_at(pos)? as usize;
        Some(pos + offset)
    }

    /// Read a table at `table_pos`. Returns the absolute offset of field `field_idx`.
    /// Returns None if the field doesn't exist in the vtable.
    fn table_field_offset(&self, table_pos: usize, field_idx: usize) -> Option<usize> {
        // vtable_soffset is a signed 32-bit value at table_pos, pointing BACKWARDS to the vtable
        let soffset = self.i32_at(table_pos)? as isize;
        let vtable_pos = (table_pos as isize - soffset) as usize;

        let vtable_size = self.u16_at(vtable_pos)? as usize;
        let field_offset_pos = vtable_pos + 4 + field_idx * 2;
        if field_offset_pos + 2 > vtable_pos + vtable_size {
            return None; // Field not in vtable
        }
        let field_offset = self.u16_at(field_offset_pos)? as usize;
        if field_offset == 0 {
            return None; // Field not present
        }
        Some(table_pos + field_offset)
    }

    /// Read a string at a vector-of-bytes offset.
    /// String is: u32 length + utf8 bytes (no null terminator in FlatBuffer).
    fn read_string(&self, pos: usize) -> Option<String> {
        let target = self.follow_u32_offset(pos)?;
        let len = self.u32_at(target)? as usize;
        let bytes = self.data.get(target + 4..target + 4 + len)?;
        Some(String::from_utf8_lossy(bytes).into_owned())
    }

    /// Read a vector header at `pos` → returns (vector_start, count).
    fn read_vector_header(&self, pos: usize) -> Option<(usize, usize)> {
        let target = self.follow_u32_offset(pos)?;
        let count = self.u32_at(target)? as usize;
        Some((target + 4, count))
    }

    /// Read a vector of i32 values.
    fn read_i32_vector(&self, pos: usize) -> Option<Vec<i32>> {
        let (start, count) = self.read_vector_header(pos)?;
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            result.push(self.i32_at(start + i * 4)?);
        }
        Some(result)
    }

    /// Read a vector of u8 bytes (buffer data).
    fn read_u8_vector(&self, pos: usize) -> Option<Vec<u8>> {
        let target = self.follow_u32_offset(pos)?;
        let len = self.u32_at(target)? as usize;
        let bytes = self.data.get(target + 4..target + 4 + len)?;
        Some(bytes.to_vec())
    }

    /// Get root table position.
    pub fn root(&self) -> Option<usize> {
        let offset = self.u32_at(0)? as usize;
        Some(offset)
    }
}

// ── TFLite high-level structs ──────────────────────────────────────────────────

#[derive(Debug)]
pub struct TfTensor {
    pub name: String,
    pub dtype: DType,
    pub shape: Vec<i32>,
    pub buffer_idx: usize,
}

#[derive(Debug)]
pub struct TfModel {
    pub tensors: Vec<TfTensor>,
    pub buffers: Vec<Vec<u8>>,
}

/// Parse a TFLite file bytes into TfModel.
/// Field indices per TFLite schema:
///   Model:    version=0, operator_codes=1, subgraphs=2, description=3, buffers=4
///   SubGraph: tensors=0, inputs=1, outputs=2, operators=3, name=4
///   Tensor:   shape=0, type=1, buffer=2, name=3
///   Buffer:   data=0
pub fn parse_tflite(data: &[u8]) -> Result<TfModel, UmcError> {
    let fb = FlatBufReader::new(data);
    let model_pos = fb
        .root()
        .ok_or_else(|| UmcError::Other("TFLite: cannot read root offset".into()))?;

    // ── Read buffers ────────────────────────────────────────────────────────
    // Model.buffers = field index 4
    let buffers_field = fb
        .table_field_offset(model_pos, 4)
        .ok_or_else(|| UmcError::Other("TFLite: no buffers field".into()))?;
    let (buffers_vec_start, n_buffers) = fb
        .read_vector_header(buffers_field)
        .ok_or_else(|| UmcError::Other("TFLite: cannot read buffers vector".into()))?;

    let mut buffers: Vec<Vec<u8>> = Vec::with_capacity(n_buffers);
    for i in 0..n_buffers {
        let buf_offset_pos = buffers_vec_start + i * 4;
        let buf_table_pos = fb
            .follow_u32_offset(buf_offset_pos)
            .ok_or_else(|| UmcError::Other(format!("TFLite: buffer[{}] offset invalid", i)))?;
        // Buffer.data = field 0
        let data_vec = if let Some(data_field) = fb.table_field_offset(buf_table_pos, 0) {
            fb.read_u8_vector(data_field).unwrap_or_default()
        } else {
            vec![]
        };
        buffers.push(data_vec);
    }

    // ── Read subgraphs[0] ────────────────────────────────────────────────────
    // Model.subgraphs = field index 2
    let subgraphs_field = fb
        .table_field_offset(model_pos, 2)
        .ok_or_else(|| UmcError::Other("TFLite: no subgraphs field".into()))?;
    let (subgraphs_vec_start, n_subgraphs) = fb
        .read_vector_header(subgraphs_field)
        .ok_or_else(|| UmcError::Other("TFLite: cannot read subgraphs vector".into()))?;
    if n_subgraphs == 0 {
        return Err(UmcError::Other("TFLite: no subgraphs".into()));
    }

    // Use first subgraph
    let sg_offset_pos = subgraphs_vec_start;
    let sg_table_pos = fb
        .follow_u32_offset(sg_offset_pos)
        .ok_or_else(|| UmcError::Other("TFLite: cannot read subgraph[0]".into()))?;

    // SubGraph.tensors = field 0
    let tensors_field = fb
        .table_field_offset(sg_table_pos, 0)
        .ok_or_else(|| UmcError::Other("TFLite: subgraph has no tensors".into()))?;
    let (tensors_vec_start, n_tensors) = fb
        .read_vector_header(tensors_field)
        .ok_or_else(|| UmcError::Other("TFLite: cannot read tensors vector".into()))?;

    let mut tensors: Vec<TfTensor> = Vec::with_capacity(n_tensors);
    for i in 0..n_tensors {
        let t_offset_pos = tensors_vec_start + i * 4;
        let t_table_pos = match fb.follow_u32_offset(t_offset_pos) {
            Some(p) => p,
            None => continue,
        };

        // Tensor.shape = field 0
        let shape = fb
            .table_field_offset(t_table_pos, 0)
            .and_then(|f| fb.read_i32_vector(f))
            .unwrap_or_default();

        // Tensor.type = field 1
        let dtype_int = fb
            .table_field_offset(t_table_pos, 1)
            .and_then(|f| fb.i32_at(f))
            .unwrap_or(0);

        // Tensor.buffer = field 2
        let buffer_idx = fb
            .table_field_offset(t_table_pos, 2)
            .and_then(|f| fb.u32_at(f))
            .unwrap_or(0) as usize;

        // Tensor.name = field 3
        let name = fb
            .table_field_offset(t_table_pos, 3)
            .and_then(|f| fb.read_string(f))
            .unwrap_or_else(|| format!("tensor_{}", i));

        tensors.push(TfTensor {
            name,
            dtype: tensor_type_to_dtype(dtype_int),
            shape,
            buffer_idx,
        });
    }

    Ok(TfModel { tensors, buffers })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dtype_mapping() {
        assert_eq!(tensor_type_to_dtype(0), DType::F32);
        assert_eq!(tensor_type_to_dtype(3), DType::U8);
        assert_eq!(tensor_type_to_dtype(9), DType::I8);
    }
}
