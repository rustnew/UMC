/// TFLite saver — writes a minimal but valid FlatBuffer .tflite file.
/// Uses a forward-only offset strategy (patch-later) to produce correct FlatBuffer bytes.
/// Layout: header → model_vtable → model_table → subgraphs_vec → sg_vtable → sg_table →
///          tensors_vec → tensor_vtables/tables → shapes/names → inputs/outputs →
///          buffers_vec → buffer_vtables/tables → buffer_data
use std::io::Write;
use std::path::Path;
use umc_core::{DType, UmcError, UniversalIR};
use umc_core::{FormatSaver, ProgressCallback, SaveOptions};

pub struct TFLiteSaver;

impl FormatSaver for TFLiteSaver {
    fn format_name(&self) -> &'static str {
        "TFLite"
    }
    fn default_extension(&self) -> &'static str {
        "tflite"
    }

    fn save(
        &self,
        ir: &UniversalIR,
        path: &Path,
        _opts: &SaveOptions,
        progress: &ProgressCallback,
    ) -> Result<(), UmcError> {
        let tmp_path = path.with_extension("tflite.tmp");
        let bytes = build_tflite_flatbuffer(ir, progress)?;
        {
            let mut f = std::fs::File::create(&tmp_path).map_err(UmcError::Io)?;
            f.write_all(&bytes).map_err(UmcError::Io)?;
        }
        std::fs::rename(&tmp_path, path).map_err(UmcError::Io)?;
        Ok(())
    }
}

fn dtype_to_tflite_type(dtype: &DType) -> i32 {
    match dtype {
        DType::F32 => 0,
        DType::F16 => 1,
        DType::I32 => 2,
        DType::U8 => 3,
        DType::I64 => 4,
        DType::Bool => 6,
        DType::I16 => 7,
        DType::I8 => 9,
        DType::BF16 => 16,
        _ => 0,
    }
}

// ── FlatBuffer builder with forward-only offsets (patch-later) ────────────────

struct Fb {
    buf: Vec<u8>,
}

impl Fb {
    fn new() -> Self {
        Self { buf: Vec::new() }
    }
    fn pos(&self) -> usize {
        self.buf.len()
    }

    fn pu16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn pu32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn pi32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn align4(&mut self) {
        while self.buf.len() % 4 != 0 {
            self.buf.push(0);
        }
    }
    fn align2(&mut self) {
        while self.buf.len() % 2 != 0 {
            self.buf.push(0);
        }
    }

    /// Reserve a u32 slot (placeholder 0) and return its position.
    fn slot(&mut self) -> usize {
        let p = self.pos();
        self.pu32(0);
        p
    }

    /// Patch a slot with a FORWARD offset: target_pos - slot_pos.
    fn patch(&mut self, slot_pos: usize, target_pos: usize) {
        debug_assert!(
            target_pos >= slot_pos,
            "patch: backward offset {}->{}",
            slot_pos,
            target_pos
        );
        let offset = (target_pos - slot_pos) as u32;
        self.buf[slot_pos..slot_pos + 4].copy_from_slice(&offset.to_le_bytes());
    }

    /// Patch the root slot with an absolute position (reader reads it directly).
    fn patch_root(&mut self, slot_pos: usize, target_pos: usize) {
        let v = target_pos as u32;
        self.buf[slot_pos..slot_pos + 4].copy_from_slice(&v.to_le_bytes());
    }

    /// Write vtable: fields is a slice of (present, field_offset_in_object) pairs.
    /// Returns vtable_pos.
    fn write_vtable(&mut self, field_offsets: &[u16]) -> usize {
        self.align2();
        let vt_pos = self.pos();
        let vtable_size = 4 + 2 * field_offsets.len() as u16;
        let object_size = 4 + 4 * field_offsets.len() as u16; // 4-byte soffset + N×4-byte fields
        self.pu16(vtable_size);
        self.pu16(object_size);
        for &fo in field_offsets {
            self.pu16(fo);
        }
        vt_pos
    }

    /// Write table header (soffset to vtable). Returns table_pos.
    fn write_table_header(&mut self, vtable_pos: usize) -> usize {
        self.align4();
        let tp = self.pos();
        let soffset = (tp as i32) - (vtable_pos as i32);
        self.pi32(soffset);
        tp
    }

    /// Write a u8 vector: u32 count + bytes. Returns its start pos.
    fn write_u8_vec(&mut self, data: &[u8]) -> usize {
        self.align4();
        let p = self.pos();
        self.pu32(data.len() as u32);
        self.buf.extend_from_slice(data);
        p
    }

    /// Write a i32 vector: u32 count + i32s. Returns its start pos.
    fn write_i32_vec(&mut self, data: &[i32]) -> usize {
        self.align4();
        let p = self.pos();
        self.pu32(data.len() as u32);
        for &v in data {
            self.pi32(v);
        }
        p
    }

    /// Write a string: u32 len + bytes. Returns its start pos.
    fn write_str(&mut self, s: &str) -> usize {
        self.align4();
        let p = self.pos();
        self.pu32(s.len() as u32);
        self.buf.extend_from_slice(s.as_bytes());
        p
    }
}

/// Build a valid TFLite FlatBuffer with forward-only offsets.
/// Layout (in order):
///   header(8) → model_vtable → model_table → subgraphs_vec → sg_vtable → sg_table →
///   tensors_vec → [tensor_vtable + tensor_table] × N →
///   [shape_vec + name_str] × N →
///   inputs_vec → outputs_vec →
///   buffers_vec → [buf_vtable + buf_table] × (N+1) → [buf_data] × N
fn build_tflite_flatbuffer(
    ir: &UniversalIR,
    progress: &ProgressCallback,
) -> Result<Vec<u8>, UmcError> {
    let mut tensor_list: Vec<(&str, &umc_core::Tensor)> =
        ir.tensors.iter().map(|(n, t)| (n.as_str(), t)).collect();
    tensor_list.sort_by_key(|(n, _)| *n);
    let n = tensor_list.len();
    progress.set_total(n as u64);

    let mut fb = Fb::new();

    // ── Header: root_offset(slot) + "TFL3" ───────────────────────────────
    let root_slot = fb.slot(); // [0..4]
    fb.buf.extend_from_slice(b"TFL3"); // [4..8]

    // ── Model vtable: version(0), op_codes(1=absent), subgraphs(2), desc(3=absent), buffers(4)
    // Each field occupies a 4-byte slot: soffset(4)+ver(4)+ops(4)+sg(4)+desc(4)+bufs(4)=24 bytes
    // Absent fields (vtable offset=0) still occupy physical slots for proper alignment.
    let model_vt = fb.write_vtable(&[4, 0, 12, 0, 20]);
    // ── Model table ───────────────────────────────────────────────────────
    let model_tp = fb.write_table_header(model_vt);
    fb.patch_root(root_slot, model_tp); // patch root_offset = model_tp (absolute)
    fb.pu32(3); // version = 3
    fb.pu32(0); // op_codes = absent (0 for unused slot)
    let model_subgraphs_slot = fb.slot(); // slot for subgraphs offset
    fb.pu32(0); // description = absent
    let model_buffers_slot = fb.slot(); // slot for buffers offset

    // ── Subgraphs vector: count=1 + slot for subgraph[0] offset ──────────
    fb.align4();
    let subgraphs_vec_pos = fb.pos();
    fb.patch(model_subgraphs_slot, subgraphs_vec_pos);
    fb.pu32(1); // count = 1
    let sg_slot = fb.slot(); // slot for subgraph[0] table offset

    // ── SubGraph vtable: tensors(0), inputs(1), outputs(2) ───────────────
    let sg_vt = fb.write_vtable(&[4, 8, 12]);
    // ── SubGraph table ────────────────────────────────────────────────────
    let sg_tp = fb.write_table_header(sg_vt);
    fb.patch(sg_slot, sg_tp); // patch subgraph[0] slot → sg_tp
    let sg_tensors_slot = fb.slot();
    let sg_inputs_slot = fb.slot();
    let sg_outputs_slot = fb.slot();

    // ── Tensors vector: count=N + N slots for tensor table offsets ────────
    fb.align4();
    let tensors_vec_pos = fb.pos();
    fb.patch(sg_tensors_slot, tensors_vec_pos);
    fb.pu32(n as u32);
    let tensor_slots: Vec<usize> = (0..n).map(|_| fb.slot()).collect();

    // ── Tensor vtables + tables (all shapes/names written later) ──────────
    let mut tensor_shape_slots: Vec<usize> = Vec::with_capacity(n);
    let mut tensor_name_slots: Vec<usize> = Vec::with_capacity(n);

    for (i, (name, tensor)) in tensor_list.iter().enumerate() {
        let tflite_type = dtype_to_tflite_type(&tensor.dtype);
        let buffer_idx = (i + 1) as u32; // buffer[0] is reserved empty

        // Vtable: shape(0)=4, type(1)=8, buffer(2)=12, name(3)=16
        let t_vt = fb.write_vtable(&[4, 8, 12, 16]);
        let t_tp = fb.write_table_header(t_vt);
        fb.patch(tensor_slots[i], t_tp);

        tensor_shape_slots.push(fb.slot()); // slot for shape vector offset
        fb.pi32(tflite_type); // type (inline)
        fb.pu32(buffer_idx); // buffer (inline)
        tensor_name_slots.push(fb.slot()); // slot for name string offset

        progress.increment(name);
    }

    // ── Tensor shapes and names (written after all tensor tables) ─────────
    for (i, (name, tensor)) in tensor_list.iter().enumerate() {
        let shape_i32: Vec<i32> = tensor.shape.iter().map(|&s| s as i32).collect();
        let shape_pos = fb.write_i32_vec(&shape_i32);
        fb.patch(tensor_shape_slots[i], shape_pos);

        let name_pos = fb.write_str(name);
        fb.patch(tensor_name_slots[i], name_pos);
    }

    // ── Inputs vector (empty) + Outputs vector (all tensor indices) ───────
    fb.align4();
    let inputs_pos = fb.pos();
    fb.patch(sg_inputs_slot, inputs_pos);
    fb.pu32(0); // count = 0

    fb.align4();
    let outputs_pos = fb.pos();
    fb.patch(sg_outputs_slot, outputs_pos);
    let all_idx: Vec<i32> = (0..n as i32).collect();
    fb.pu32(n as u32);
    for v in all_idx {
        fb.pi32(v);
    }

    // ── Buffers vector: count=(N+1) + (N+1) slots ─────────────────────────
    fb.align4();
    let buffers_vec_pos = fb.pos();
    fb.patch(model_buffers_slot, buffers_vec_pos);
    fb.pu32((n + 1) as u32);
    let buf_table_slots: Vec<usize> = (0..n + 1).map(|_| fb.slot()).collect();

    // ── Buffer[0]: empty — 0 field vtable (object = soffset only) ────────
    let b0_vt = fb.write_vtable(&[]); // 0 fields
    let b0_tp = fb.write_table_header(b0_vt);
    fb.patch(buf_table_slots[0], b0_tp);

    // ── Buffer[i] (i=1..N+1): has data field ─────────────────────────────
    let mut buf_data_slots: Vec<usize> = Vec::with_capacity(n);
    for i in 1..=n {
        let bi_vt = fb.write_vtable(&[4]); // field 0 at offset 4
        let bi_tp = fb.write_table_header(bi_vt);
        fb.patch(buf_table_slots[i], bi_tp);
        buf_data_slots.push(fb.slot()); // slot for data offset
    }

    // ── Buffer data (written last) ─────────────────────────────────────────
    for (i, (_name, tensor)) in tensor_list.iter().enumerate() {
        let raw = tensor
            .data
            .as_bytes()
            .map_err(|e| UmcError::Other(format!("TFLite saver: {}", e)))?;
        let data_pos = fb.write_u8_vec(raw);
        fb.patch(buf_data_slots[i], data_pos);
    }

    Ok(fb.buf)
}
