use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use umc_core::UmcError;
use crate::ConversionGraph;

/// A hop in the conversion path.
#[derive(Debug, Clone)]
pub struct ConversionHop {
    pub source: String,
    pub target: String,
    pub cost: f64,
    pub native: bool,
    pub description: String,
}

/// Result of a path search.
#[derive(Debug, Clone)]
pub struct ConversionPath {
    pub hops: Vec<ConversionHop>,
    pub total_cost: f64,
}

impl ConversionPath {
    pub fn is_direct(&self) -> bool {
        self.hops.len() == 1
    }

    pub fn hop_count(&self) -> usize {
        self.hops.len()
    }

    pub fn all_native(&self) -> bool {
        self.hops.iter().all(|h| h.native)
    }

    /// Human-readable path description.
    pub fn display_path(&self) -> String {
        let parts: Vec<String> = self.hops.iter()
            .map(|h| format!("{} → {}", h.source, h.target))
            .collect();
        parts.join(" → ")
    }
}

// ── Dijkstra implementation ───────────────────────────────────────────────────

#[derive(Clone)]
struct State {
    cost: f64,
    node: String,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool { self.cost == other.cost && self.node == other.node }
}
impl Eq for State {}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
            .then_with(|| self.node.cmp(&other.node))
    }
}

/// Find the shortest (lowest-cost) path from `source` to `target` in the graph.
///
/// Uses Dijkstra's algorithm on the weighted directed graph.
/// Returns an error if no path exists.
pub fn find_path(
    graph: &ConversionGraph,
    source: &str,
    target: &str,
) -> Result<ConversionPath, UmcError> {
    if source == target {
        return Ok(ConversionPath {
            hops: vec![],
            total_cost: 0.0,
        });
    }

    let mut dist: HashMap<String, f64> = HashMap::new();
    let mut prev: HashMap<String, (String, f64, bool, String)> = HashMap::new();
    let mut heap = BinaryHeap::new();

    dist.insert(source.to_string(), 0.0);
    heap.push(State { cost: 0.0, node: source.to_string() });

    while let Some(State { cost, node }) = heap.pop() {
        if node == target {
            // Reconstruct path
            let mut path = Vec::new();
            let mut current = target.to_string();
            while let Some((prev_node, hop_cost, native, desc)) = prev.get(&current) {
                path.push(ConversionHop {
                    source: prev_node.clone(),
                    target: current.clone(),
                    cost: *hop_cost,
                    native: *native,
                    description: desc.clone(),
                });
                current = prev_node.clone();
            }
            path.reverse();
            return Ok(ConversionPath {
                total_cost: dist[target],
                hops: path,
            });
        }

        if cost > *dist.get(&node).unwrap_or(&f64::INFINITY) {
            continue;
        }

        for edge in graph.edges_from(&node) {
            let next_cost = cost + edge.cost;
            let current_best = dist.get(&edge.target).copied().unwrap_or(f64::INFINITY);
            if next_cost < current_best {
                dist.insert(edge.target.clone(), next_cost);
                prev.insert(
                    edge.target.clone(),
                    (node.clone(), edge.cost, edge.native, edge.description.clone()),
                );
                heap.push(State { cost: next_cost, node: edge.target.clone() });
            }
        }
    }

    Err(UmcError::NoConversionPath {
        from: source.to_string(),
        to: target.to_string(),
        available: graph.formats().collect::<Vec<_>>().join(", "),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConversionGraph;

    #[test]
    fn test_direct_path() {
        let g = ConversionGraph::default_graph();
        let path = find_path(&g, "GGUF", "SafeTensors").unwrap();
        assert_eq!(path.hop_count(), 1);
        assert_eq!(path.hops[0].source, "GGUF");
        assert_eq!(path.hops[0].target, "SafeTensors");
        assert!(path.all_native());
    }

    #[test]
    fn test_multihop_path() {
        let g = ConversionGraph::default_graph();
        // GGUF → ONNX (direct or via SafeTensors → ONNX, Dijkstra picks cheapest)
        let path = find_path(&g, "GGUF", "ONNX").unwrap();
        assert!(path.hop_count() >= 1);
        assert_eq!(path.hops.first().unwrap().source, "GGUF");
        assert_eq!(path.hops.last().unwrap().target, "ONNX");
    }

    #[test]
    fn test_no_path() {
        let g = ConversionGraph::default_graph();
        let result = find_path(&g, "GGUF", "NONEXISTENT_FORMAT_XYZ");
        assert!(matches!(result, Err(UmcError::NoConversionPath { .. })));
    }

    #[test]
    fn test_same_source_and_target() {
        let g = ConversionGraph::default_graph();
        let path = find_path(&g, "GGUF", "GGUF").unwrap();
        assert_eq!(path.hop_count(), 0);
        assert_eq!(path.total_cost, 0.0);
    }

    #[test]
    fn test_path_display() {
        let g = ConversionGraph::default_graph();
        let path = find_path(&g, "GGUF", "SafeTensors").unwrap();
        let display = path.display_path();
        assert!(display.contains("GGUF"));
        assert!(display.contains("SafeTensors"));
    }
}
