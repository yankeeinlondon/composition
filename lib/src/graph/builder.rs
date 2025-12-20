use crate::error::Result;
use crate::parse::parse_document;
use crate::types::{DependencyGraph, Frontmatter, GraphNode, Resource, ResourceHash};
use futures::future::BoxFuture;
use std::collections::HashMap;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use tracing::{debug, instrument};

use super::utils::{compute_content_hash, compute_resource_hash, load_resource};

/// Build a dependency graph starting from a root resource
///
/// This recursively parses all referenced documents and builds a complete
/// dependency graph with content hashes for cache validation.
#[instrument(skip(db, frontmatter), fields(root = ?root.source))]
pub async fn build_graph(
    root: Resource,
    db: &Surreal<Db>,
    frontmatter: &Frontmatter,
) -> Result<DependencyGraph> {
    let mut graph = DependencyGraph::new(root.clone());
    let mut visited: HashMap<ResourceHash, bool> = HashMap::new();

    // Start recursive traversal
    visit_resource(&root, &mut graph, &mut visited, db, frontmatter).await?;

    debug!("Graph built with {} nodes and {} edges", graph.nodes.len(), graph.edges.len());

    Ok(graph)
}

/// Recursively visit a resource and build the graph
#[instrument(skip_all, fields(source = ?resource.source))]
fn visit_resource<'a>(
    resource: &'a Resource,
    graph: &'a mut DependencyGraph,
    visited: &'a mut HashMap<ResourceHash, bool>,
    db: &'a Surreal<Db>,
    frontmatter: &'a Frontmatter,
) -> BoxFuture<'a, Result<ResourceHash>> {
    Box::pin(async move {
    let hash = compute_resource_hash(resource);

    // Check if already visited
    if visited.contains_key(&hash) {
        debug!("Resource already visited, skipping");
        return Ok(hash);
    }

    // Mark as being visited (for cycle detection)
    visited.insert(hash, true);

    // Load and parse the resource
    debug!("Loading resource");
    let content = load_resource(resource).await?;
    let content_hash = compute_content_hash(&content);

    debug!("Parsing document");
    let document = parse_document(&content, resource.clone())?;

    // Collect dependency hashes
    let mut dependency_hashes = Vec::new();

    // Recursively visit dependencies
    for dep in &document.dependencies {
        debug!("Processing dependency: {:?}", dep.source);
        let dep_hash = visit_resource(dep, graph, visited, db, frontmatter).await?;
        dependency_hashes.push(dep_hash);

        // Add edge to graph
        graph.add_edge(hash, dep_hash);
    }

    // Create graph node
    let node = GraphNode {
        resource: resource.clone(),
        content_hash: Some(content_hash),
        dependencies: dependency_hashes,
    };

    // Add node to graph
    graph.add_node(hash, node);

    Ok(hash)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    async fn setup_test_db() -> (Surreal<Db>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = crate::cache::init_database(&db_path).await.unwrap();
        crate::cache::apply_schema(&db).await.unwrap();

        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_build_graph_single_file() {
        let (db, _temp_dir) = setup_test_db().await;

        // Create a test file
        let test_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(test_file.path(), "# Hello\n\nThis is a test.").unwrap();

        let resource = Resource::local(test_file.path().to_path_buf());
        let frontmatter = Frontmatter::default();

        let graph = build_graph(resource.clone(), &db, &frontmatter).await.unwrap();

        assert_eq!(graph.root.source, resource.source);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 0);
    }

    #[tokio::test]
    async fn test_build_graph_with_dependencies() {
        let (db, _temp_dir) = setup_test_db().await;

        // Create test files
        let temp_dir = TempDir::new().unwrap();
        let dep_file = temp_dir.path().join("dep.md");
        let root_file = temp_dir.path().join("root.md");

        std::fs::write(&dep_file, "# Dependency\n\nDependency content.").unwrap();
        std::fs::write(
            &root_file,
            format!("# Root\n\n::file {}", dep_file.to_string_lossy())
        ).unwrap();

        let resource = Resource::local(root_file.clone());
        let frontmatter = Frontmatter::default();

        let graph = build_graph(resource, &db, &frontmatter).await.unwrap();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }

    #[tokio::test]
    async fn test_build_graph_deduplicates() {
        let (db, _temp_dir) = setup_test_db().await;

        // Create test files where two files depend on the same third file
        let temp_dir = TempDir::new().unwrap();
        let shared_file = temp_dir.path().join("shared.md");
        let dep1_file = temp_dir.path().join("dep1.md");
        let dep2_file = temp_dir.path().join("dep2.md");
        let root_file = temp_dir.path().join("root.md");

        std::fs::write(&shared_file, "# Shared\n\nShared content.").unwrap();
        std::fs::write(
            &dep1_file,
            format!("# Dep1\n\n::file {}", shared_file.to_string_lossy())
        ).unwrap();
        std::fs::write(
            &dep2_file,
            format!("# Dep2\n\n::file {}", shared_file.to_string_lossy())
        ).unwrap();
        std::fs::write(
            &root_file,
            format!(
                "# Root\n\n::file {}\n\n::file {}",
                dep1_file.to_string_lossy(),
                dep2_file.to_string_lossy()
            )
        ).unwrap();

        let resource = Resource::local(root_file.clone());
        let frontmatter = Frontmatter::default();

        let graph = build_graph(resource, &db, &frontmatter).await.unwrap();

        // Should have 4 nodes: root, dep1, dep2, shared
        // Shared should only appear once due to deduplication
        assert_eq!(graph.nodes.len(), 4);
    }
}
