use crate::error::{ParseError, Result};
use crate::types::{DependencyGraph, ResourceHash, WorkLayer, WorkPlan};
use std::collections::{HashMap, VecDeque};
use tracing::{debug, instrument};

use super::cycles::detect_cycles;

/// Generate a work plan from a dependency graph using topological sort
///
/// Uses Kahn's algorithm to create layers of work that can be executed in parallel.
/// Each layer contains resources with no remaining dependencies, allowing parallel
/// execution within each layer while maintaining correct dependency order.
///
/// Returns an error if the graph contains cycles.
#[instrument(skip(graph))]
pub fn generate_workplan(graph: &DependencyGraph) -> Result<WorkPlan> {
    debug!("Generating work plan for graph with {} nodes", graph.nodes.len());

    // First, verify the graph is acyclic
    detect_cycles(graph)?;

    // Build in-degree map and adjacency list
    let mut in_degree: HashMap<ResourceHash, usize> = HashMap::new();
    let mut adjacency: HashMap<ResourceHash, Vec<ResourceHash>> = HashMap::new();

    // Initialize all nodes with in-degree 0
    for &hash in graph.nodes.keys() {
        in_degree.entry(hash).or_insert(0);
        adjacency.entry(hash).or_default();
    }

    // Count in-degrees from edges
    for &(from, to) in &graph.edges {
        *in_degree.entry(to).or_insert(0) += 1;
        adjacency.entry(from).or_default().push(to);
    }

    let mut plan = WorkPlan::new();
    let mut queue: VecDeque<ResourceHash> = VecDeque::new();

    // Start with all nodes that have in-degree 0 (leaves)
    for (&hash, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(hash);
        }
    }

    // Process nodes layer by layer
    while !queue.is_empty() {
        let layer_size = queue.len();
        let mut layer_resources = Vec::new();

        // Process all nodes in the current layer
        for _ in 0..layer_size {
            if let Some(hash) = queue.pop_front() {
                // Get the resource for this node
                if let Some(node) = graph.nodes.get(&hash) {
                    layer_resources.push(node.resource.clone());
                }

                // Reduce in-degree for all neighbors
                if let Some(neighbors) = adjacency.get(&hash) {
                    for &neighbor in neighbors {
                        if let Some(degree) = in_degree.get_mut(&neighbor) {
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push_back(neighbor);
                            }
                        }
                    }
                }

                // Remove from in_degree map
                in_degree.remove(&hash);
            }
        }

        if !layer_resources.is_empty() {
            plan.add_layer(WorkLayer {
                resources: layer_resources,
                parallelizable: true,
            });
        }
    }

    // If there are still nodes with non-zero in-degree, we have a cycle
    // (This should never happen since we checked for cycles earlier)
    if !in_degree.is_empty() {
        return Err(crate::error::CompositionError::Parse(ParseError::CircularDependency {
            cycle: "Unexpected cycle detected during work plan generation".to_string(),
        }));
    }

    // Reverse the layers so leaves are processed first
    plan.layers.reverse();

    debug!("Generated work plan with {} layers, {} total tasks", plan.layers.len(), plan.total_tasks);

    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::utils::compute_resource_hash;
    use crate::types::{DependencyGraph, GraphNode, Resource};
    use std::path::PathBuf;

    #[test]
    fn test_generate_workplan_single_node() {
        let a = Resource::local(PathBuf::from("a.md"));
        let hash_a = compute_resource_hash(&a);

        let mut graph = DependencyGraph::new(a.clone());
        graph.add_node(hash_a, GraphNode {
            resource: a.clone(),
            content_hash: Some("hash_a".to_string()),
            dependencies: vec![],
        });

        let plan = generate_workplan(&graph).unwrap();

        assert_eq!(plan.layers.len(), 1);
        assert_eq!(plan.total_tasks, 1);
        assert_eq!(plan.layers[0].resources.len(), 1);
    }

    #[test]
    fn test_generate_workplan_linear() {
        // Create A -> B -> C (linear chain)
        let a = Resource::local(PathBuf::from("a.md"));
        let b = Resource::local(PathBuf::from("b.md"));
        let c = Resource::local(PathBuf::from("c.md"));

        let hash_a = compute_resource_hash(&a);
        let hash_b = compute_resource_hash(&b);
        let hash_c = compute_resource_hash(&c);

        let mut graph = DependencyGraph::new(a.clone());

        graph.add_node(hash_a, GraphNode {
            resource: a.clone(),
            content_hash: Some("hash_a".to_string()),
            dependencies: vec![hash_b],
        });

        graph.add_node(hash_b, GraphNode {
            resource: b.clone(),
            content_hash: Some("hash_b".to_string()),
            dependencies: vec![hash_c],
        });

        graph.add_node(hash_c, GraphNode {
            resource: c.clone(),
            content_hash: Some("hash_c".to_string()),
            dependencies: vec![],
        });

        graph.add_edge(hash_a, hash_b);
        graph.add_edge(hash_b, hash_c);

        let plan = generate_workplan(&graph).unwrap();

        // Should have 3 layers since it's a linear chain
        assert_eq!(plan.layers.len(), 3);
        assert_eq!(plan.total_tasks, 3);

        // First layer should contain C (the leaf)
        assert_eq!(plan.layers[0].resources.len(), 1);

        // Last layer should contain A (the root)
        assert_eq!(plan.layers[2].resources.len(), 1);
    }

    #[test]
    fn test_generate_workplan_parallel() {
        // Create A -> B, A -> C (B and C can be processed in parallel)
        let a = Resource::local(PathBuf::from("a.md"));
        let b = Resource::local(PathBuf::from("b.md"));
        let c = Resource::local(PathBuf::from("c.md"));

        let hash_a = compute_resource_hash(&a);
        let hash_b = compute_resource_hash(&b);
        let hash_c = compute_resource_hash(&c);

        let mut graph = DependencyGraph::new(a.clone());

        graph.add_node(hash_a, GraphNode {
            resource: a.clone(),
            content_hash: Some("hash_a".to_string()),
            dependencies: vec![hash_b, hash_c],
        });

        graph.add_node(hash_b, GraphNode {
            resource: b.clone(),
            content_hash: Some("hash_b".to_string()),
            dependencies: vec![],
        });

        graph.add_node(hash_c, GraphNode {
            resource: c.clone(),
            content_hash: Some("hash_c".to_string()),
            dependencies: vec![],
        });

        graph.add_edge(hash_a, hash_b);
        graph.add_edge(hash_a, hash_c);

        let plan = generate_workplan(&graph).unwrap();

        // Should have 2 layers
        assert_eq!(plan.layers.len(), 2);
        assert_eq!(plan.total_tasks, 3);

        // First layer should contain B and C (both leaves)
        assert_eq!(plan.layers[0].resources.len(), 2);

        // Second layer should contain A
        assert_eq!(plan.layers[1].resources.len(), 1);
    }

    #[test]
    fn test_generate_workplan_diamond() {
        // Create A -> B -> D, A -> C -> D
        let a = Resource::local(PathBuf::from("a.md"));
        let b = Resource::local(PathBuf::from("b.md"));
        let c = Resource::local(PathBuf::from("c.md"));
        let d = Resource::local(PathBuf::from("d.md"));

        let hash_a = compute_resource_hash(&a);
        let hash_b = compute_resource_hash(&b);
        let hash_c = compute_resource_hash(&c);
        let hash_d = compute_resource_hash(&d);

        let mut graph = DependencyGraph::new(a.clone());

        graph.add_node(hash_a, GraphNode {
            resource: a.clone(),
            content_hash: Some("hash_a".to_string()),
            dependencies: vec![hash_b, hash_c],
        });

        graph.add_node(hash_b, GraphNode {
            resource: b.clone(),
            content_hash: Some("hash_b".to_string()),
            dependencies: vec![hash_d],
        });

        graph.add_node(hash_c, GraphNode {
            resource: c.clone(),
            content_hash: Some("hash_c".to_string()),
            dependencies: vec![hash_d],
        });

        graph.add_node(hash_d, GraphNode {
            resource: d.clone(),
            content_hash: Some("hash_d".to_string()),
            dependencies: vec![],
        });

        graph.add_edge(hash_a, hash_b);
        graph.add_edge(hash_a, hash_c);
        graph.add_edge(hash_b, hash_d);
        graph.add_edge(hash_c, hash_d);

        let plan = generate_workplan(&graph).unwrap();

        // Should have 3 layers: [D], [B, C], [A]
        assert_eq!(plan.layers.len(), 3);
        assert_eq!(plan.total_tasks, 4);

        // First layer: D (the common dependency)
        assert_eq!(plan.layers[0].resources.len(), 1);

        // Second layer: B and C (can be parallel)
        assert_eq!(plan.layers[1].resources.len(), 2);

        // Third layer: A (the root)
        assert_eq!(plan.layers[2].resources.len(), 1);
    }

    #[test]
    fn test_generate_workplan_with_cycle() {
        // Create A -> B -> A (cycle)
        let a = Resource::local(PathBuf::from("a.md"));
        let b = Resource::local(PathBuf::from("b.md"));

        let hash_a = compute_resource_hash(&a);
        let hash_b = compute_resource_hash(&b);

        let mut graph = DependencyGraph::new(a.clone());

        graph.add_node(hash_a, GraphNode {
            resource: a.clone(),
            content_hash: Some("hash_a".to_string()),
            dependencies: vec![hash_b],
        });

        graph.add_node(hash_b, GraphNode {
            resource: b.clone(),
            content_hash: Some("hash_b".to_string()),
            dependencies: vec![hash_a],
        });

        graph.add_edge(hash_a, hash_b);
        graph.add_edge(hash_b, hash_a);

        let result = generate_workplan(&graph);
        assert!(result.is_err());
    }
}
