mod builder;
mod cycles;
mod workplan;
mod cache;
pub mod utils;
pub mod gitignore;

pub use builder::build_graph;
pub use cycles::detect_cycles;
pub use workplan::generate_workplan;
pub use cache::{persist_graph, load_graph};
pub use utils::{compute_resource_hash, compute_content_hash, load_resource};

use crate::error::Result;
use crate::types::{DependencyGraph, Resource, Frontmatter};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

/// Build a dependency graph for a resource
pub async fn graph(
    resource: Resource,
    db: &Surreal<Db>,
    frontmatter: &Frontmatter,
) -> Result<DependencyGraph> {
    build_graph(resource, db, frontmatter).await
}
