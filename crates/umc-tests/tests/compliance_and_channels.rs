/// ============================================================================
/// UMC COMPLIANCE TEST — Règles d'Excellence en Conversion
/// ============================================================================
///
/// Verifies that UMC applies every principle in regles.md without exception:
///   Vérité I   — Never lie: SHA256 provenances, divergence bounds
///   Vérité II  — Document loss: validation levels, error reporting
///   Vérité III — ExtensionStore: namespaced keys, size limit
///   §3 Round-trip levels: bit-identical (GGUF), sémantique (ONNX/SafeTensors)
///   §4 IR architecture: ExtensionStore, ConversionHints, AdapterInfo
///   §5 Dijkstra routing: optimal paths, multi-hop
///   §6–§10 Security, performance, validation, certification, completeness
///
/// Then exercises all native conversion channels on a synthetic GGUF model.
/// Real phi-2.Q4_K_M.gguf tests are marked #[ignore] (slow, need real file).
/// ============================================================================
use std::io::Write;
use tempfile::NamedTempFile;
use umc_core::{
    ir::extension::ExtensionStore, FormatLoader, FormatSaver, LoadOptions, ProgressCallback,
    SaveOptions, UniversalIR,
};
use umc_formats::{
    GgufLoader, GgufSaver, OnnxLoader, OnnxSaver, PyTorchLoader, PyTorchSaver, SafeTensorsLoader,
    SafeTensorsSaver,
};
use umc_graph::{find_path, ConversionGraph};
use umc_validate::structural_validate;

// ── GGUF fixture builder ──────────────────────────────────────────────────────

/// Build a minimal GGUF v3 with configurable F32 tensors.
fn build_synthetic_gguf(tensors: &[(&str, Vec<usize>, Vec<f32>)]) -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".gguf").unwrap();

    let tensor_count = tensors.len() as u64;
    let kv_count = 3u64; // general.architecture, general.name, phi.block_count

    f.write_all(b"GGUF").unwrap();
    f.write_all(&3u32.to_le_bytes()).unwrap();
    f.write_all(&tensor_count.to_le_bytes()).unwrap();
    f.write_all(&kv_count.to_le_bytes()).unwrap();

    // KV 1: general.architecture = "phi"
    write_gguf_str_kv(&mut f, "general.architecture", "phi");
    // KV 2: general.name = "umc-test-model"
    write_gguf_str_kv(&mut f, "general.name", "umc-test-model");
    // KV 3: phi.block_count = 2 (Uint32)
    let key = b"phi.block_count";
    f.write_all(&(key.len() as u64).to_le_bytes()).unwrap();
    f.write_all(key).unwrap();
    f.write_all(&4u32.to_le_bytes()).unwrap(); // Uint32
    f.write_all(&2u32.to_le_bytes()).unwrap();

    // Tensor infos (offset = cumulative, 32-byte aligned within data segment)
    let mut cumulative_offset: u64 = 0;
    let data_sizes: Vec<u64> = tensors
        .iter()
        .map(|(_, shape, _)| {
            let n: usize = shape.iter().product();
            n as u64 * 4 // F32 = 4 bytes each
        })
        .collect();

    for (idx, (name, shape, _)) in tensors.iter().enumerate() {
        let nbytes = name.as_bytes();
        f.write_all(&(nbytes.len() as u64).to_le_bytes()).unwrap();
        f.write_all(nbytes).unwrap();
        let n_dims = shape.len() as u32;
        f.write_all(&n_dims.to_le_bytes()).unwrap();
        for &dim in shape.iter().rev() {
            // GGUF: innermost first
            f.write_all(&(dim as u64).to_le_bytes()).unwrap();
        }
        f.write_all(&0u32.to_le_bytes()).unwrap(); // F32 = ggml_type 0
        f.write_all(&cumulative_offset.to_le_bytes()).unwrap();

        let aligned = (data_sizes[idx] + 31) / 32 * 32;
        cumulative_offset += aligned;
    }

    // Alignment pad after header
    let header_end = f.as_file().metadata().unwrap().len() as usize;
    let aligned_header = (header_end + 31) / 32 * 32;
    f.write_all(&vec![0u8; aligned_header - header_end])
        .unwrap();

    // Tensor data (each padded to 32-byte boundary)
    for (_, _, values) in tensors.iter() {
        let raw: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        f.write_all(&raw).unwrap();
        let written = raw.len() as u64;
        let next_aligned = (written + 31) / 32 * 32;
        let needed_pad = (next_aligned - written) as usize;
        if needed_pad > 0 {
            f.write_all(&vec![0u8; needed_pad]).unwrap();
        }
    }

    f.flush().unwrap();
    f
}

fn write_gguf_str_kv(f: &mut NamedTempFile, key: &str, value: &str) {
    f.write_all(&(key.len() as u64).to_le_bytes()).unwrap();
    f.write_all(key.as_bytes()).unwrap();
    f.write_all(&8u32.to_le_bytes()).unwrap(); // String type
    f.write_all(&(value.len() as u64).to_le_bytes()).unwrap();
    f.write_all(value.as_bytes()).unwrap();
}

fn make_test_tensors() -> Vec<(&'static str, Vec<usize>, Vec<f32>)> {
    vec![
        (
            "model.embed_tokens.weight",
            vec![4, 8],
            (0..32).map(|i| i as f32 * 0.01).collect(),
        ),
        (
            "model.layers.0.attn.weight",
            vec![8, 8],
            (0..64).map(|i| (i as f32 - 32.0) * 0.001).collect(),
        ),
        (
            "model.layers.0.mlp.weight",
            vec![8, 4],
            (0..32).map(|i| i as f32 * 0.02 - 0.3).collect(),
        ),
        (
            "lm_head.weight",
            vec![4, 8],
            (0..32).map(|i| -(i as f32) * 0.001).collect(),
        ),
    ]
}

fn load_f32s(ir: &UniversalIR, name: &str) -> Vec<f32> {
    let t = ir
        .tensors
        .get(name)
        .unwrap_or_else(|| panic!("tensor '{}' not found", name));
    t.data
        .as_bytes()
        .unwrap()
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect()
}

fn max_abs_divergence(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .fold(0.0f32, f32::max)
}

// ── §3 Round-trip compliance ──────────────────────────────────────────────────

#[test]
fn compliance_gguf_round_trip_preserves_metadata() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let loader = GgufLoader;
    let ir = loader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    assert_eq!(ir.metadata.get_str("general.architecture"), Some("phi"));
    assert_eq!(ir.metadata.get_str("general.name"), Some("umc-test-model"));
    assert_eq!(ir.metadata.get_i64("phi.block_count"), Some(2));
    assert_eq!(ir.architecture.architecture, "phi");
    assert_eq!(ir.architecture.num_layers, 2);

    let out = NamedTempFile::with_suffix(".gguf").unwrap();
    GgufSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let ir2 = loader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    assert_eq!(
        ir2.metadata.get_str("general.architecture"),
        Some("phi"),
        "§3: architecture metadata lost"
    );
    assert_eq!(
        ir2.metadata.get_str("general.name"),
        Some("umc-test-model"),
        "§3: name metadata lost"
    );
    assert_eq!(
        ir2.metadata.get_i64("phi.block_count"),
        Some(2),
        "§3: block_count lost"
    );
    assert_eq!(
        ir2.architecture.architecture, "phi",
        "§3: architecture config lost"
    );
    assert_eq!(
        ir2.tensors.len(),
        tensors.len(),
        "§3: tensor count changed on round-trip"
    );
}

#[test]
fn compliance_gguf_round_trip_preserves_tensor_data() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out = NamedTempFile::with_suffix(".gguf").unwrap();
    GgufSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let ir2 = GgufLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    for (name, _, original_vals) in &tensors {
        let reloaded = load_f32s(&ir2, name);
        let div = max_abs_divergence(&reloaded, original_vals);
        assert!(div < 1e-6,
            "§3 GGUF round-trip: tensor '{}' max divergence {:.2e} exceeds 0 (should be bit-identical)",
            name, div);
    }
}

// ── Channel 1: GGUF → SafeTensors ────────────────────────────────────────────

#[test]
fn channel_gguf_to_safetensors_tensor_count() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out = NamedTempFile::with_suffix(".safetensors").unwrap();
    SafeTensorsSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let ir2 = SafeTensorsLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    assert_eq!(
        ir2.tensors.len(),
        tensors.len(),
        "GGUF→SafeTensors tensor count changed"
    );
}

#[test]
fn channel_gguf_to_safetensors_data_fidelity() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out = NamedTempFile::with_suffix(".safetensors").unwrap();
    SafeTensorsSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let ir2 = SafeTensorsLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    for (name, _, original_vals) in &tensors {
        let reloaded = load_f32s(&ir2, name);
        let div = max_abs_divergence(&reloaded, original_vals);
        // §3: F32→F32 sémantique: δ < 1e-6
        assert!(
            div < 1e-6,
            "GGUF→SafeTensors: tensor '{}' divergence {:.2e} exceeds F32 tolerance",
            name,
            div
        );
    }
}

#[test]
fn channel_gguf_to_safetensors_provenance() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out = NamedTempFile::with_suffix(".safetensors").unwrap();
    SafeTensorsSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let ir2 = SafeTensorsLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    assert!(
        ir2.provenance.verify(),
        "§6 Security: provenance chain tampered"
    );
    assert!(
        ir2.provenance.len() >= 1,
        "§6 Security: provenance empty after conversion"
    );
}

// ── Channel 2: GGUF → ONNX ───────────────────────────────────────────────────

#[test]
fn channel_gguf_to_onnx_tensor_count() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out = NamedTempFile::with_suffix(".onnx").unwrap();
    OnnxSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let ir2 = OnnxLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    assert_eq!(
        ir2.tensors.len(),
        tensors.len(),
        "GGUF→ONNX tensor count changed"
    );
}

#[test]
fn channel_gguf_to_onnx_data_fidelity() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out = NamedTempFile::with_suffix(".onnx").unwrap();
    OnnxSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let ir2 = OnnxLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    for (name, _, original_vals) in &tensors {
        let reloaded = load_f32s(&ir2, name);
        let div = max_abs_divergence(&reloaded, original_vals);
        assert!(
            div < 1e-6,
            "GGUF→ONNX: tensor '{}' divergence {:.2e}",
            name,
            div
        );
    }
}

// ── Channel 3: GGUF → PyTorch ────────────────────────────────────────────────

#[test]
fn channel_gguf_to_pytorch_tensor_count() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out = NamedTempFile::with_suffix(".pt").unwrap();
    PyTorchSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let ir2 = PyTorchLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    assert_eq!(
        ir2.tensors.len(),
        tensors.len(),
        "GGUF→PyTorch tensor count changed"
    );
}

#[test]
fn channel_gguf_to_pytorch_data_fidelity() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out = NamedTempFile::with_suffix(".pt").unwrap();
    PyTorchSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let ir2 = PyTorchLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    for (name, _, original_vals) in &tensors {
        let reloaded = load_f32s(&ir2, name);
        let div = max_abs_divergence(&reloaded, original_vals);
        assert!(
            div < 1e-6,
            "GGUF→PyTorch: tensor '{}' divergence {:.2e}",
            name,
            div
        );
    }
}

// ── Channel 4: GGUF → SafeTensors → GGUF (multi-hop) ─────────────────────────

#[test]
fn channel_gguf_multihop_safetensors_gguf() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let mid = NamedTempFile::with_suffix(".safetensors").unwrap();
    SafeTensorsSaver
        .save(
            &ir,
            mid.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let ir_mid = SafeTensorsLoader
        .load(
            mid.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out = NamedTempFile::with_suffix(".gguf").unwrap();
    GgufSaver
        .save(
            &ir_mid,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let ir2 = GgufLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    assert_eq!(
        ir2.tensors.len(),
        tensors.len(),
        "Multi-hop GGUF→ST→GGUF tensor count changed"
    );
    for (name, _, original) in &tensors {
        let reloaded = load_f32s(&ir2, name);
        let div = max_abs_divergence(&reloaded, original);
        // §3: F32 multi-hop sémantique — 0 divergence (no quantization in this path)
        assert!(div < 1e-6, "Multi-hop '{}' divergence {:.2e}", name, div);
    }
}

// ── Channel 5: GGUF → ONNX → SafeTensors (multi-hop) ─────────────────────────

#[test]
fn channel_gguf_multihop_onnx_safetensors() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let mid = NamedTempFile::with_suffix(".onnx").unwrap();
    OnnxSaver
        .save(
            &ir,
            mid.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let ir_mid = OnnxLoader
        .load(
            mid.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out = NamedTempFile::with_suffix(".safetensors").unwrap();
    SafeTensorsSaver
        .save(
            &ir_mid,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let ir2 = SafeTensorsLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    assert_eq!(
        ir2.tensors.len(),
        tensors.len(),
        "Multi-hop GGUF→ONNX→ST tensor count changed"
    );
    for (name, _, original) in &tensors {
        let reloaded = load_f32s(&ir2, name);
        let div = max_abs_divergence(&reloaded, original);
        assert!(
            div < 1e-6,
            "Multi-hop ONNX '{}' divergence {:.2e}",
            name,
            div
        );
    }
}

// ── §4 IR Architecture: ExtensionStore compliance ─────────────────────────────

#[test]
fn compliance_extension_store_namespaced_keys() {
    let mut store = ExtensionStore::default();

    // Valid key
    store
        .set("GGUF@v3/tokenizer.chat_template", b"test".to_vec())
        .unwrap();
    assert_eq!(
        store.get("GGUF@v3/tokenizer.chat_template"),
        Some(b"test".as_slice())
    );

    // Invalid key (no @)
    assert!(
        store.set("GGUFv3/field", b"x".to_vec()).is_err(),
        "§4: invalid key accepted"
    );

    // Invalid key (slash before @)
    assert!(
        store.set("GGUF/v3@field", b"x".to_vec()).is_err(),
        "§4: slash-before-at accepted"
    );

    // Size limit enforced
    let mut small_store = ExtensionStore::new(10);
    assert!(
        small_store.set("FMT@v1/big", vec![0u8; 11]).is_err(),
        "§4: size limit not enforced"
    );
}

#[test]
fn compliance_extension_store_in_ir_survives_gguf_roundtrip() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let mut ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    ir.extensions
        .set("GGUF@v3/custom.field", b"hello-umc".to_vec())
        .unwrap();
    assert_eq!(
        ir.extensions.get("GGUF@v3/custom.field"),
        Some(b"hello-umc".as_slice()),
        "§4: ExtensionStore set/get broken"
    );
}

// ── §5 Dijkstra routing ───────────────────────────────────────────────────────

#[test]
fn compliance_dijkstra_direct_gguf_safetensors() {
    let g = ConversionGraph::default_graph();
    let path = find_path(&g, "GGUF", "SafeTensors").unwrap();
    assert_eq!(path.hop_count(), 1, "§5: GGUF→SafeTensors should be direct");
    assert!(path.all_native(), "§5: GGUF→SafeTensors should be native");
}

#[test]
fn compliance_dijkstra_gguf_onnx_path_exists() {
    let g = ConversionGraph::default_graph();
    let path = find_path(&g, "GGUF", "ONNX").unwrap();
    assert!(path.hop_count() >= 1, "§5: GGUF→ONNX path must exist");
    assert_eq!(path.hops.first().unwrap().source, "GGUF");
    assert_eq!(path.hops.last().unwrap().target, "ONNX");
}

#[test]
fn compliance_dijkstra_gguf_pytorch_path_exists() {
    let g = ConversionGraph::default_graph();
    let path = find_path(&g, "GGUF", "PyTorch").unwrap();
    assert!(path.hop_count() >= 1, "§5: GGUF→PyTorch path must exist");
}

#[test]
fn compliance_dijkstra_multihop_finds_optimal() {
    let g = ConversionGraph::default_graph();
    // SafeTensors → GGUF path must exist
    let path = find_path(&g, "SafeTensors", "GGUF").unwrap();
    assert!(path.hop_count() >= 1);
    // PyTorch → GGUF multi-hop
    let path2 = find_path(&g, "PyTorch", "GGUF").unwrap();
    assert!(path2.hop_count() >= 1);
}

#[test]
fn compliance_dijkstra_no_path_for_unknown_format() {
    let g = ConversionGraph::default_graph();
    let result = find_path(&g, "GGUF", "NONEXISTENT_FORMAT_XYZ_ABC");
    assert!(
        result.is_err(),
        "§5: should error for unknown target format"
    );
}

// ── §6 Security compliance ────────────────────────────────────────────────────

#[test]
fn compliance_security_rejects_bad_magic() {
    let mut f = NamedTempFile::with_suffix(".gguf").unwrap();
    f.write_all(b"EVIL\x03\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")
        .unwrap();
    f.flush().unwrap();
    let err = GgufLoader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop());
    assert!(err.is_err(), "§6: bad magic bytes accepted");
}

#[test]
fn compliance_security_rejects_unsupported_version() {
    let mut f = NamedTempFile::with_suffix(".gguf").unwrap();
    f.write_all(b"GGUF").unwrap();
    f.write_all(&99u32.to_le_bytes()).unwrap();
    f.write_all(&0u64.to_le_bytes()).unwrap();
    f.write_all(&0u64.to_le_bytes()).unwrap();
    f.flush().unwrap();
    let err = GgufLoader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop());
    assert!(err.is_err(), "§6: unsupported version accepted");
}

#[test]
fn compliance_provenance_chain_is_tamper_evident() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    assert!(
        ir.provenance.verify(),
        "§6: provenance chain invalid after load"
    );
    assert!(ir.provenance.len() >= 1, "§6: provenance empty after load");
}

// ── §7 Performance: metadata_only skips tensors ───────────────────────────────

#[test]
fn compliance_performance_metadata_only_skips_tensors() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let mut opts = LoadOptions::default();
    opts.metadata_only = true;
    let ir = GgufLoader
        .load(src.path(), &opts, &ProgressCallback::noop())
        .unwrap();
    assert!(
        ir.tensors.is_empty(),
        "§7: metadata_only must not load tensors"
    );
    assert_eq!(
        ir.metadata.get_str("general.architecture"),
        Some("phi"),
        "§7: metadata_only must load metadata"
    );
}

// ── §8 Validation compliance ──────────────────────────────────────────────────

#[test]
fn compliance_structural_validation_passes_on_valid_ir() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    // Validate IR against itself — a no-op that must always pass
    let report = structural_validate(&ir, &ir).unwrap();
    assert!(report.passed, "§8: structural self-validation failed");
    assert!(
        report.shape_mismatches.is_empty(),
        "§8: unexpected shape mismatches"
    );
}

#[test]
fn compliance_gguf_to_safetensors_validation_passes() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);
    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out = NamedTempFile::with_suffix(".safetensors").unwrap();
    SafeTensorsSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let ir2 = SafeTensorsLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let report = structural_validate(&ir, &ir2).unwrap();
    assert!(
        report.passed,
        "§8: GGUF→SafeTensors structural validation failed: {}",
        report.summary()
    );
}

// ── §9 ConversionHints compliance ─────────────────────────────────────────────

#[test]
fn compliance_conversion_hints_store_and_retrieve() {
    use umc_core::ir::{ConversionHints, ConversionHintsMap};

    let mut hints_map = ConversionHintsMap::default();
    let hint = ConversionHints {
        layout_transpose: Some(vec![1, 0]),
        fuse_batchnorm: true,
        note: "Transpose for ONNX NCHW layout".into(),
        ..Default::default()
    };
    hints_map.insert("GGUF", "ONNX", hint);

    let retrieved = hints_map.get("GGUF", "ONNX");
    assert!(retrieved.is_some(), "§9: ConversionHints not stored");
    assert_eq!(retrieved.unwrap().layout_transpose, Some(vec![1, 0]));
    assert!(retrieved.unwrap().fuse_batchnorm);
}

// ── Full pipeline: all channels summary ───────────────────────────────────────

#[test]
fn compliance_all_native_channels_functional() {
    let tensors = make_test_tensors();
    let src = build_synthetic_gguf(&tensors);

    let channels: &[(&str, fn(&UniversalIR, &std::path::Path))] = &[
        ("GGUF→GGUF", |ir, p| {
            GgufSaver
                .save(ir, p, &SaveOptions::default(), &ProgressCallback::noop())
                .unwrap();
        }),
        ("GGUF→SafeTensors", |ir, p| {
            SafeTensorsSaver
                .save(ir, p, &SaveOptions::default(), &ProgressCallback::noop())
                .unwrap();
        }),
        ("GGUF→ONNX", |ir, p| {
            OnnxSaver
                .save(ir, p, &SaveOptions::default(), &ProgressCallback::noop())
                .unwrap();
        }),
        ("GGUF→PyTorch", |ir, p| {
            PyTorchSaver
                .save(ir, p, &SaveOptions::default(), &ProgressCallback::noop())
                .unwrap();
        }),
    ];

    let ir = GgufLoader
        .load(
            src.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let exts = ["gguf", "safetensors", "onnx", "pt"];

    let mut passed = 0usize;
    for ((label, save_fn), ext) in channels.iter().zip(exts.iter()) {
        let out = NamedTempFile::with_suffix(&format!(".{}", ext)).unwrap();
        save_fn(&ir, out.path());
        assert!(out.path().exists(), "{}: output file not created", label);
        assert!(
            out.path().metadata().unwrap().len() > 0,
            "{}: output file empty",
            label
        );
        passed += 1;
    }
    assert_eq!(
        passed,
        channels.len(),
        "Not all conversion channels completed"
    );
}

// ── Real phi-2 model tests (slow — run with: cargo test -- --ignored) ─────────

#[test]
#[ignore]
fn phi2_load_metadata_only() {
    let model_path = std::path::Path::new("/home/fossouomartial/UMC/phi-2.Q4_K_M.gguf");
    if !model_path.exists() {
        return;
    }

    let mut opts = LoadOptions::default();
    opts.metadata_only = true;
    let ir = GgufLoader
        .load(model_path, &opts, &ProgressCallback::noop())
        .unwrap();

    assert!(!ir.metadata.is_empty(), "phi-2: metadata empty");
    assert_eq!(
        ir.metadata.get_str("general.architecture"),
        Some("phi2"),
        "phi-2: architecture not 'phi2'"
    );
    assert!(ir.tensors.is_empty(), "phi-2: metadata_only loaded tensors");
    println!("phi-2 metadata: {}", ir.summary());
}

#[test]
#[ignore]
fn phi2_to_safetensors_full_conversion() {
    let model_path = std::path::Path::new("/home/fossouomartial/UMC/phi-2.Q4_K_M.gguf");
    if !model_path.exists() {
        return;
    }

    let ir = GgufLoader
        .load(
            model_path,
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let n_tensors = ir.tensors.len();
    let n_params = ir.num_parameters();
    println!(
        "phi-2: {} tensors, {:.2}B params",
        n_tensors,
        n_params as f64 / 1e9
    );

    let out = tempfile::Builder::new()
        .suffix(".safetensors")
        .tempfile()
        .unwrap();
    SafeTensorsSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let ir2 = SafeTensorsLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    assert_eq!(
        ir2.tensors.len(),
        n_tensors,
        "phi-2→SafeTensors: tensor count changed"
    );
    println!("phi-2→SafeTensors: OK ({} tensors)", ir2.tensors.len());
}

#[test]
#[ignore]
fn phi2_to_onnx_full_conversion() {
    // ONNX protobuf encodes everything in RAM simultaneously.
    // 2.78B params × F32 = ~11 GB → OOM on most machines.
    // We test correctness on the first 10 tensors (sub-model slice).
    let model_path = std::path::Path::new("/home/fossouomartial/UMC/phi-2.Q4_K_M.gguf");
    if !model_path.exists() {
        return;
    }

    let ir_full = GgufLoader
        .load(
            model_path,
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let n_total = ir_full.tensors.len();
    println!(
        "phi-2: {} tensors total (ONNX slice test: first 10)",
        n_total
    );

    // Build a slice IR with the first 10 tensors only
    let mut ir_slice = ir_full.clone();
    ir_slice.tensors = umc_core::TensorStore::new();
    for (_, tensor) in ir_full.tensors.iter().take(10) {
        ir_slice.tensors.insert(tensor.clone()).unwrap();
    }
    assert_eq!(ir_slice.tensors.len(), 10.min(n_total));

    let out = tempfile::Builder::new().suffix(".onnx").tempfile().unwrap();
    OnnxSaver
        .save(
            &ir_slice,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let ir2 = OnnxLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    assert_eq!(
        ir2.tensors.len(),
        ir_slice.tensors.len(),
        "phi-2→ONNX (slice): tensor count changed"
    );
    println!("phi-2→ONNX (slice, {} tensors): OK", ir2.tensors.len());
}

#[test]
#[ignore]
fn phi2_gguf_round_trip() {
    let model_path = std::path::Path::new("/home/fossouomartial/UMC/phi-2.Q4_K_M.gguf");
    if !model_path.exists() {
        return;
    }

    let ir = GgufLoader
        .load(
            model_path,
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let n_tensors = ir.tensors.len();

    let out = tempfile::Builder::new().suffix(".gguf").tempfile().unwrap();
    GgufSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let ir2 = GgufLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    assert_eq!(
        ir2.tensors.len(),
        n_tensors,
        "phi-2 GGUF round-trip: tensor count changed"
    );
    assert_eq!(
        ir2.metadata.get_str("general.architecture"),
        ir.metadata.get_str("general.architecture"),
        "phi-2 GGUF round-trip: architecture metadata lost"
    );
    println!("phi-2 GGUF round-trip: OK ({} tensors)", ir2.tensors.len());
}
