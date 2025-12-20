use crate::error::{ParseError, Result};
use crate::types::{DependencyGraph, ResourceHash};
use std::collections::{HashMap, HashSet};
use tracing::{debug, instrument};

/// Detect cycles in a dependency graph using depth-first search
///
/// Returns Ok(()) if the graph is acyclic, or an error with the cycle path
/// if a cycle is detected.
#[instrument(skip(graph))]
pub fn detect_cycles(graph: &DependencyGraph) -> Result<()> {
    debug!("Detecting cycles in graph with {} nodes", graph.nodes.len());

    let mut visiting = HashSet::new(); // Gray nodes (currently being visited)
    let mut visited = HashSet::new(); // Black nodes (completely processed)

    // Build adjacency map for efficient lookups
    let mut adjacency: HashMap<ResourceHash, Vec<ResourceHash>> = HashMap::new();
    for (from, to) in &graph.edges {
        adjacency.entry(*from).or_insert_with(Vec::new).push(*to);
    }

    // Try DFS from each unvisited node
    for &hash in graph.nodes.keys() {
        if !visited.contains(&hash) {
            let mut path = Vec::new();
            if let Err(cycle_path) = dfs(
                hash,
                &adjacency,
                &mut visiting,
                &mut visited,
                &mut path,
                graph,
            ) {
                return Err(cycle_path);
            }
        }
    }

    debug!("No cycles detected");
    Ok(())
}

/// Depth-first search for cycle detection
///
/// Returns Err with a formatted error message if a cycle is found
fn dfs(
    node: ResourceHash,
    adjacency: &HashMap<ResourceHash, Vec<ResourceHash>>,
    visiting: &mut HashSet<ResourceHash>,
    visited: &mut HashSet<ResourceHash>,
    path: &mut Vec<ResourceHash>,
    graph: &DependencyGraph,
) -> Result<()> {
    // If we're currently visiting this node, we found a cycle
    if visiting.contains(&node) {
        // Find where the cycle starts in the path
        let cycle_start = path.iter().position(|&h| h == node).unwrap_or(0);
        let cycle_nodes: Vec<ResourceHash> = path[cycle_start..].to_vec();

        // Build error message with resource paths
        let cycle_description = cycle_nodes
            .iter()
            .filter_map(|h| graph.nodes.get(h))
            .map(|n| match &n.resource.source {
                crate::types::ResourceSource::Local(path) => path.to_string_lossy().to_string(),
                crate::types::ResourceSource::Remote(url) => url.to_string(),
            })
            .collect::<Vec<_>>()
            .join(" -> ");

        return Err(crate::error::CompositionError::Parse(ParseError::CircularDependency {
            cycle: cycle_description,
        }));
    }

    // If already fully visited, nothing to do
    if visited.contains(&node) {
        return Ok(());
    }

    // Mark as currently visiting
    visiting.insert(node);
    path.push(node);

    // Visit all neighbors
    if let Some(neighbors) = adjacency.get(&node) {
        for &neighbor in neighbors {
            dfs(neighbor, adjacency, visiting, visited, path, graph)?;
        }
    }

    // Done visiting this node
    path.pop();
    visiting.remove(&node);
    visited.insert(node);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DependencyGraph, GraphNode, Resource};
    use std::path::PathBuf;

    #[test]
    fn test_detect_cycles_acyclic() {
        // Create a simple acyclic graph: A -> B -> C
        let a = Resource::local(PathBuf::from("a.md"));
        let b = Resource::local(PathBuf::from("b.md"));
        let c = Resource::local(PathBuf::from("c.md"));

        let hash_a = crate::graph::utils::compute_resource_hash(&a);
        let hash_b = crate::graph::utils::compute_resource_hash(&b);
        let hash_c = crate::graph::utils::compute_resource_hash(&c);

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

        let result = detect_cycles(&graph);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_cycles_simple_cycle() {
        // Create a cycle: A -> B -> A
        let a = Resource::local(PathBuf::from("a.md"));
        let b = Resource::local(PathBuf::from("b.md"));

        let hash_a = crate::graph::utils::compute_resource_hash(&a);
        let hash_b = crate::graph::utils::compute_resource_hash(&b);

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

        let result = detect_cycles(&graph);
        assert!(result.is_err());

        if let Err(crate::error::CompositionError::Parse(
            crate::error::ParseError::CircularDependency { cycle }
        )) = result {
            assert!(cycle.contains("a.md"));
            assert!(cycle.contains("b.md"));
        } else {
            panic!("Expected CircularDependency error, got: {:?}", result);
        }
    }

    #[test]
    fn test_detect_cycles_self_reference() {
        // Create a self-reference: A -> A
        let a = Resource::local(PathBuf::from("a.md"));
        let hash_a = crate::graph::utils::compute_resource_hash(&a);

        let mut graph = DependencyGraph::new(a.clone());

        graph.add_node(hash_a, GraphNode {
            resource: a.clone(),
            content_hash: Some("hash_a".to_string()),
            dependencies: vec![hash_a],
        });

        graph.add_edge(hash_a, hash_a);

        let result = detect_cycles(&graph);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_cycles_diamond() {
        // Create a diamond: A -> B -> D, A -> C -> D (no cycle)
        let a = Resource::local(PathBuf::from("a.md"));
        let b = Resource::local(PathBuf::from("b.md"));
        let c = Resource::local(PathBuf::from("c.md"));
        let d = Resource::local(PathBuf::from("d.md"));

        let hash_a = crate::graph::utils::compute_resource_hash(&a);
        let hash_b = crate::graph::utils::compute_resource_hash(&b);
        let hash_c = crate::graph::utils::compute_resource_hash(&c);
        let hash_d = crate::graph::utils::compute_resource_hash(&d);

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

        let result = detect_cycles(&graph);
        assert!(result.is_ok());
    }
}
