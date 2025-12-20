use super::{Resource, ResourceHash};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dependency graph for a document tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub root: Resource,
    pub nodes: HashMap<ResourceHash, GraphNode>,
    pub edges: Vec<(ResourceHash, ResourceHash)>,
}

/// Node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub resource: Resource,
    pub content_hash: Option<String>,
    pub dependencies: Vec<ResourceHash>,
}

/// Execution plan for rendering documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPlan {
    pub layers: Vec<WorkLayer>,
    pub total_tasks: usize,
}

/// A layer of work that can be executed in parallel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkLayer {
    pub resources: Vec<Resource>,
    pub parallelizable: bool,
}

impl DependencyGraph {
    pub fn new(root: Resource) -> Self {
        Self {
            root,
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, hash: ResourceHash, node: GraphNode) {
        self.nodes.insert(hash, node);
    }

    pub fn add_edge(&mut self, from: ResourceHash, to: ResourceHash) {
        self.edges.push((from, to));
    }
}

impl WorkPlan {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            total_tasks: 0,
        }
    }

    pub fn add_layer(&mut self, layer: WorkLayer) {
        self.total_tasks += layer.resources.len();
        self.layers.push(layer);
    }
}

impl Default for WorkPlan {
    fn default() -> Self {
        Self::new()
    }
}
