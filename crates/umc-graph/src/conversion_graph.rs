/// A directed weighted edge between two formats.
#[derive(Debug, Clone)]
pub struct ConversionEdge {
    pub source: String,
    pub target: String,
    /// Cost in seconds (used by Dijkstra to find the fastest path).
    pub cost: f64,
    /// True if this edge is fully native Rust (no external tools).
    pub native: bool,
    /// Human-readable description.
    pub description: String,
}

/// Directed weighted conversion graph.
///
/// Nodes are format names (strings).
/// Edges are conversion paths with costs.
/// Dijkstra finds the optimal multi-hop path automatically.
pub struct ConversionGraph {
    edges: Vec<ConversionEdge>,
    formats: std::collections::HashSet<String>,
}

impl ConversionGraph {
    /// Create an empty graph.
    pub fn empty() -> Self {
        Self {
            edges: Vec::new(),
            formats: std::collections::HashSet::new(),
        }
    }

    /// Create the default graph with all built-in conversion paths.
    pub fn default_graph() -> Self {
        let mut g = Self::empty();

        // ── Tier 1 native conversions ─────────────────────────────────────
        // GGUF self-loop (round-trip, re-serialise metadata + weights unchanged)
        g.add_edge(
            "GGUF",
            "GGUF",
            1.0,
            true,
            "GGUF → GGUF (native round-trip, sémantique)",
        );

        // GGUF ↔ SafeTensors (weights-only, always native)
        g.add_edge(
            "GGUF",
            "SafeTensors",
            5.0,
            true,
            "GGUF → SafeTensors (native Rust, dequantize)",
        );
        g.add_edge(
            "SafeTensors",
            "GGUF",
            8.0,
            true,
            "SafeTensors → GGUF (native Rust, quantize)",
        );

        // SafeTensors ↔ PyTorch
        g.add_edge(
            "SafeTensors",
            "PyTorch",
            3.0,
            true,
            "SafeTensors → PyTorch (native Rust)",
        );
        g.add_edge(
            "PyTorch",
            "SafeTensors",
            3.0,
            true,
            "PyTorch → SafeTensors (native Rust)",
        );

        // GGUF → ONNX (requires GraphTemplate for architecture)
        g.add_edge(
            "GGUF",
            "ONNX",
            15.0,
            true,
            "GGUF → ONNX (GraphTemplate + native Rust)",
        );
        g.add_edge(
            "SafeTensors",
            "ONNX",
            12.0,
            true,
            "SafeTensors → ONNX (GraphTemplate + native Rust)",
        );
        g.add_edge(
            "PyTorch",
            "ONNX",
            10.0,
            true,
            "PyTorch → ONNX (native Rust)",
        );
        g.add_edge(
            "ONNX",
            "SafeTensors",
            8.0,
            true,
            "ONNX → SafeTensors (native Rust)",
        );
        g.add_edge(
            "ONNX",
            "GGUF",
            18.0,
            true,
            "ONNX → GGUF (via SafeTensors bridge)",
        );

        // ONNX ↔ TFLite
        g.add_edge(
            "ONNX",
            "TFLite",
            20.0,
            true,
            "ONNX → TFLite (native FlatBuffers)",
        );
        g.add_edge(
            "TFLite",
            "ONNX",
            15.0,
            true,
            "TFLite → ONNX (native FlatBuffers)",
        );

        // Legacy GGML → GGUF (read-only migration)
        g.add_edge("GGML", "GGUF", 10.0, true, "GGML → GGUF (legacy migration)");

        // AWQ / GPTQ → SafeTensors (dequantize to F16)
        g.add_edge(
            "AWQ",
            "SafeTensors",
            8.0,
            true,
            "AWQ → SafeTensors (dequantize)",
        );
        g.add_edge(
            "GPTQ",
            "SafeTensors",
            8.0,
            true,
            "GPTQ → SafeTensors (dequantize)",
        );

        // SafeTensors ↔ TFLite (direct native, weights-only)
        g.add_edge(
            "SafeTensors",
            "TFLite",
            12.0,
            true,
            "SafeTensors → TFLite (native FlatBuffers)",
        );
        g.add_edge(
            "TFLite",
            "SafeTensors",
            10.0,
            true,
            "TFLite → SafeTensors (native FlatBuffers)",
        );

        // PyTorch ↔ GGUF (via SafeTensors hop)
        g.add_edge(
            "PyTorch",
            "GGUF",
            12.0,
            true,
            "PyTorch → GGUF (via SafeTensors bridge)",
        );
        g.add_edge(
            "GGUF",
            "PyTorch",
            12.0,
            true,
            "GGUF → PyTorch (via SafeTensors bridge)",
        );

        // AWQ / GPTQ → ONNX (via SafeTensors dequantize)
        g.add_edge(
            "AWQ",
            "ONNX",
            18.0,
            true,
            "AWQ → ONNX (dequantize then export)",
        );
        g.add_edge(
            "GPTQ",
            "ONNX",
            18.0,
            true,
            "GPTQ → ONNX (dequantize then export)",
        );

        // ── Tier 2: external tool conversions (cost=2.0× native) ─────────
        // ONNX → CoreML (requires: pip install coremltools)
        g.add_edge(
            "ONNX",
            "CoreML",
            25.0,
            false,
            "ONNX → CoreML (coremltools subprocess)",
        );
        g.add_edge(
            "SafeTensors",
            "CoreML",
            35.0,
            false,
            "SafeTensors → CoreML (via ONNX + coremltools)",
        );

        // ONNX → TensorRT (requires: trtexec from TensorRT toolkit)
        g.add_edge(
            "ONNX",
            "TensorRT",
            30.0,
            false,
            "ONNX → TensorRT (trtexec subprocess)",
        );
        g.add_edge(
            "SafeTensors",
            "TensorRT",
            40.0,
            false,
            "SafeTensors → TensorRT (via ONNX + trtexec)",
        );

        // ONNX → OpenVINO (requires: Model Optimizer `mo`)
        g.add_edge(
            "ONNX",
            "OpenVINO",
            25.0,
            false,
            "ONNX → OpenVINO (Model Optimizer subprocess)",
        );
        g.add_edge(
            "SafeTensors",
            "OpenVINO",
            35.0,
            false,
            "SafeTensors → OpenVINO (via ONNX + mo)",
        );

        // ONNX → ExecuTorch (requires: executorch Python package)
        g.add_edge(
            "ONNX",
            "ExecuTorch",
            30.0,
            false,
            "ONNX → ExecuTorch (executorch subprocess)",
        );
        g.add_edge(
            "PyTorch",
            "ExecuTorch",
            20.0,
            false,
            "PyTorch → ExecuTorch (via ONNX bridge)",
        );

        // Adapters
        g.add_edge(
            "LoRA",
            "SafeTensors",
            3.0,
            true,
            "LoRA → SafeTensors (merge or export)",
        );
        g.add_edge(
            "PEFT",
            "SafeTensors",
            3.0,
            true,
            "PEFT → SafeTensors (merge or export)",
        );

        g
    }

    fn add_edge(&mut self, src: &str, tgt: &str, cost: f64, native: bool, desc: &str) {
        self.formats.insert(src.to_string());
        self.formats.insert(tgt.to_string());
        self.edges.push(ConversionEdge {
            source: src.to_string(),
            target: tgt.to_string(),
            cost,
            native,
            description: desc.to_string(),
        });
    }

    pub fn edges(&self) -> &[ConversionEdge] {
        &self.edges
    }

    pub fn formats(&self) -> impl Iterator<Item = &str> {
        self.formats.iter().map(|s| s.as_str())
    }

    /// Find direct edges from a source format.
    pub fn edges_from(&self, source: &str) -> impl Iterator<Item = &ConversionEdge> {
        let src = source.to_string();
        self.edges.iter().filter(move |e| e.source == src)
    }

    /// Check if a direct conversion edge exists.
    pub fn has_direct_edge(&self, source: &str, target: &str) -> bool {
        self.edges
            .iter()
            .any(|e| e.source == source && e.target == target)
    }

    /// Register a custom conversion path (for plugins).
    pub fn register_edge(&mut self, edge: ConversionEdge) {
        self.formats.insert(edge.source.clone());
        self.formats.insert(edge.target.clone());
        self.edges.push(edge);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_graph_has_gguf_safetensors() {
        let g = ConversionGraph::default_graph();
        assert!(g.has_direct_edge("GGUF", "SafeTensors"));
        assert!(g.has_direct_edge("SafeTensors", "GGUF"));
    }

    #[test]
    fn test_edges_from() {
        let g = ConversionGraph::default_graph();
        let edges: Vec<_> = g.edges_from("GGUF").collect();
        assert!(!edges.is_empty());
        for e in &edges {
            assert_eq!(e.source, "GGUF");
        }
    }

    #[test]
    fn test_formats_non_empty() {
        let g = ConversionGraph::default_graph();
        let names: Vec<_> = g.formats().collect();
        assert!(names.contains(&"GGUF"));
        assert!(names.contains(&"ONNX"));
        assert!(names.contains(&"SafeTensors"));
    }
}
