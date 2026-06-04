use crate::DType;
use super::super::ir::RopeScalingConfig;

/// A directed acyclic compute graph.
#[derive(Debug, Clone, Default)]
pub struct ComputeGraph {
    pub nodes: Vec<ComputeNode>,
    pub edges: Vec<ComputeEdge>,
    pub inputs: Vec<GraphTensor>,
    pub outputs: Vec<GraphTensor>,
    pub opset_version: Option<u32>,
}

/// Typed tensor edge in the graph (name, optional type+shape).
#[derive(Debug, Clone)]
pub struct GraphTensor {
    pub name: String,
    pub dtype: Option<DType>,
    pub shape: Option<Vec<Option<i64>>>,
}

/// A single compute node in the DAG.
#[derive(Debug, Clone)]
pub struct ComputeNode {
    pub id: String,
    pub op_type: UniversalOp,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub attributes: OpAttributes,
    pub domain: String,
}

#[derive(Debug, Clone)]
pub struct ComputeEdge {
    pub from_node: String,
    pub from_output: usize,
    pub to_node: String,
    pub to_input: usize,
}

/// Attribute container for a compute node.
#[derive(Debug, Clone, Default)]
pub struct OpAttributes {
    pub floats: std::collections::HashMap<String, f64>,
    pub ints: std::collections::HashMap<String, i64>,
    pub strings: std::collections::HashMap<String, String>,
    pub tensors: std::collections::HashMap<String, Vec<u8>>,
    pub graphs: std::collections::HashMap<String, ComputeGraph>,
}

/// Universal operator set — superset of all formats.
#[derive(Debug, Clone)]
pub enum UniversalOp {
    // Elementwise
    Add, Sub, Mul, Div, Pow, Sqrt, Rsqrt, Abs, Neg, Exp, Log,
    Tanh, Sigmoid, Erf, Sign, Ceil, Floor, Round,
    // Activations
    Relu, Relu6, LeakyRelu { alpha: f64 },
    Gelu, GeluApprox, Silu, Swish, HardSwish, HardSigmoid,
    Mish, QuickGelu, Elu { alpha: f64 },
    // Reduction
    ReduceSum { axes: Vec<i64>, keepdims: bool },
    ReduceMean { axes: Vec<i64>, keepdims: bool },
    ReduceMax { axes: Vec<i64>, keepdims: bool },
    ReduceMin { axes: Vec<i64>, keepdims: bool },
    ReduceProd { axes: Vec<i64>, keepdims: bool },
    // Normalization
    LayerNorm { axis: i64, eps: f64 },
    RmsNorm { eps: f64 },
    BatchNorm { eps: f64, momentum: f64, training: bool },
    GroupNorm { num_groups: i64, eps: f64 },
    InstanceNorm { eps: f64 },
    // Linear algebra
    Gemm { alpha: f64, beta: f64, trans_a: bool, trans_b: bool },
    MatMul,
    Conv2D {
        kernel_shape: Vec<i64>, strides: Vec<i64>,
        pads: Vec<i64>, dilations: Vec<i64>, group: i64, auto_pad: String,
    },
    ConvTranspose2D { kernel_shape: Vec<i64>, strides: Vec<i64>, pads: Vec<i64> },
    Linear { in_features: i64, out_features: i64, bias: bool },
    // Attention
    MultiHeadAttention { num_heads: i64, head_dim: i64 },
    GroupedQueryAttention { num_heads: i64, num_kv_heads: i64, head_dim: i64 },
    ScaledDotProductAttention { is_causal: bool },
    // Positional
    RotaryPositionEmbedding { base: f64, scaling: Option<RopeScalingConfig> },
    AlibiPositionEmbedding,
    // MoE
    MoeLayer { num_experts: i64, top_k: i64 },
    // Reshape / indexing
    Reshape, Transpose { perm: Vec<i64> },
    Flatten { axis: i64 }, Squeeze { axes: Vec<i64> }, Unsqueeze { axes: Vec<i64> },
    Concat { axis: i64 }, Split { axis: i64, sizes: Vec<i64> },
    Gather { axis: i64 },
    Slice { axes: Vec<i64>, starts: Vec<i64>, ends: Vec<i64>, steps: Vec<i64> },
    Tile { repeats: Vec<i64> }, Expand,
    Pad { mode: PadMode, pads: Vec<i64>, constant_value: f64 },
    // Pooling
    MaxPool2D { kernel_shape: Vec<i64>, strides: Vec<i64>, pads: Vec<i64> },
    AveragePool2D { kernel_shape: Vec<i64>, strides: Vec<i64>, pads: Vec<i64> },
    GlobalAveragePool, GlobalMaxPool,
    // Misc
    Softmax { axis: i64 }, LogSoftmax { axis: i64 },
    Cast { to: DType },
    Clip { min: Option<f64>, max: Option<f64> },
    Constant { value: ConstantValue },
    Identity, Where,
    Dropout { ratio: f64, training: bool },
    Embedding { padding_idx: Option<i64>, vocab_size: i64, embed_dim: i64 },
    ArgMax { axis: i64, keepdims: bool },
    ArgMin { axis: i64, keepdims: bool },
    TopK { axis: i64, largest: bool, sorted: bool },
    // Unknown operator — preserved for round-trip, NOT a fatal error.
    Custom {
        domain: String,
        op_type: String,
        attributes: std::collections::HashMap<String, Vec<u8>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PadMode { Constant, Reflect, Edge, Wrap }

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Float(f64),
    Int(i64),
    Bool(bool),
    Tensor(Vec<u8>),
}

impl ComputeGraph {
    pub fn empty() -> Self { Self::default() }

    /// Cycle detection via DFS colouring (returns false if cycles found).
    pub fn is_valid_dag(&self) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut in_stack = std::collections::HashSet::new();

        fn has_cycle(
            node_id: &str,
            edges: &[ComputeEdge],
            visited: &mut std::collections::HashSet<String>,
            in_stack: &mut std::collections::HashSet<String>,
        ) -> bool {
            if in_stack.contains(node_id) { return true; }
            if visited.contains(node_id) { return false; }
            visited.insert(node_id.to_string());
            in_stack.insert(node_id.to_string());
            for edge in edges.iter().filter(|e| e.from_node == node_id) {
                if has_cycle(&edge.to_node, edges, visited, in_stack) {
                    return true;
                }
            }
            in_stack.remove(node_id);
            false
        }

        for node in &self.nodes {
            if has_cycle(&node.id, &self.edges, &mut visited, &mut in_stack) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph_is_valid_dag() {
        let g = ComputeGraph::empty();
        assert!(g.is_valid_dag());
    }

    #[test]
    fn test_linear_graph_is_valid_dag() {
        let mut g = ComputeGraph::empty();
        g.nodes.push(ComputeNode {
            id: "a".into(), op_type: UniversalOp::Relu,
            inputs: vec![], outputs: vec!["y".into()],
            attributes: Default::default(), domain: "".into(),
        });
        g.nodes.push(ComputeNode {
            id: "b".into(), op_type: UniversalOp::Relu,
            inputs: vec!["y".into()], outputs: vec!["z".into()],
            attributes: Default::default(), domain: "".into(),
        });
        g.edges.push(ComputeEdge {
            from_node: "a".into(), from_output: 0,
            to_node: "b".into(), to_input: 0,
        });
        assert!(g.is_valid_dag());
    }

    #[test]
    fn test_cycle_detection() {
        let mut g = ComputeGraph::empty();
        // a → b → a (cycle)
        g.edges.push(ComputeEdge { from_node: "a".into(), from_output: 0, to_node: "b".into(), to_input: 0 });
        g.edges.push(ComputeEdge { from_node: "b".into(), from_output: 0, to_node: "a".into(), to_input: 0 });
        g.nodes.push(ComputeNode { id: "a".into(), op_type: UniversalOp::Add, inputs: vec![], outputs: vec![], attributes: Default::default(), domain: "".into() });
        g.nodes.push(ComputeNode { id: "b".into(), op_type: UniversalOp::Add, inputs: vec![], outputs: vec![], attributes: Default::default(), domain: "".into() });
        assert!(!g.is_valid_dag());
    }
}
