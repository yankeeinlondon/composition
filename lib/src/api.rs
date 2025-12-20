use crate::cache::CacheOperations;
use crate::error::Result;
use crate::types::{
    DependencyGraph, Document, Frontmatter, Resource, WorkPlan,
};
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use tracing::{debug, instrument, info};

/// Main API handle for the Composition library
pub struct CompositionApi {
    db: Arc<Surreal<Db>>,
    cache: Arc<CacheOperations>,
    frontmatter: Frontmatter,
    config: CompositionConfig,
}

/// Configuration for the Composition library
#[derive(Debug, Clone)]
pub struct CompositionConfig {
    pub db_path: std::path::PathBuf,
    pub project_root: Option<std::path::PathBuf>,
}

impl CompositionApi {
    /// Create a new CompositionApi instance (internal use)
    pub(crate) async fn new(
        db: Surreal<Db>,
        frontmatter: Frontmatter,
        config: CompositionConfig,
    ) -> Result<Self> {
        let db = Arc::new(db);
        let cache = Arc::new(CacheOperations::new((*db).clone()));

        Ok(Self {
            db,
            cache,
            frontmatter,
            config,
        })
    }

    /// Get the database connection
    pub fn db(&self) -> &Surreal<Db> {
        &self.db
    }

    /// Get the cache operations
    pub fn cache(&self) -> &CacheOperations {
        &self.cache
    }

    /// Get the frontmatter
    pub fn frontmatter(&self) -> &Frontmatter {
        &self.frontmatter
    }

    /// Get the configuration
    pub fn config(&self) -> &CompositionConfig {
        &self.config
    }

    // ===== Core API Functions =====

    /// Build dependency graph for a resource
    ///
    /// Recursively parses the given resource and all its dependencies, building
    /// a complete dependency graph. The graph includes content hashes for cache
    /// validation and detects circular dependencies.
    ///
    /// # Arguments
    ///
    /// * `resource` - The root resource to analyze
    ///
    /// # Returns
    ///
    /// A `DependencyGraph` containing all nodes (resources) and edges (dependencies).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The resource cannot be loaded or parsed
    /// - A circular dependency is detected
    /// - A required dependency is missing
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use lib::{init, Resource, ResourceSource, ResourceRequirement};
    /// # use std::path::PathBuf;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = init(None, None).await?;
    /// let resource = Resource {
    ///     source: ResourceSource::Local(PathBuf::from("document.md")),
    ///     requirement: ResourceRequirement::Required,
    ///     cache_duration: None,
    /// };
    ///
    /// let graph = api.graph(resource).await?;
    /// println!("Found {} dependencies", graph.nodes.len() - 1);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(source = ?resource.source))]
    pub async fn graph(&self, resource: Resource) -> Result<DependencyGraph> {
        info!("Building dependency graph");
        let graph = crate::graph::build_graph(resource, &self.db, &self.frontmatter).await?;
        debug!("Graph built with {} nodes", graph.nodes.len());
        Ok(graph)
    }

    /// Generate work plan for rendering resources
    ///
    /// Analyzes dependency graphs for multiple resources and generates an optimized
    /// work plan that groups tasks into layers for parallel execution. Resources
    /// that are already cached with fresh content are skipped.
    ///
    /// # Arguments
    ///
    /// * `resources` - A list of resources to render
    ///
    /// # Returns
    ///
    /// A `WorkPlan` with layers of tasks that can be executed in parallel.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use lib::{init, Resource, ResourceSource, ResourceRequirement};
    /// # use std::path::PathBuf;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = init(None, None).await?;
    /// let resources = vec![
    ///     Resource {
    ///         source: ResourceSource::Local(PathBuf::from("doc1.md")),
    ///         requirement: ResourceRequirement::Required,
    ///         cache_duration: None,
    ///     },
    ///     Resource {
    ///         source: ResourceSource::Local(PathBuf::from("doc2.md")),
    ///         requirement: ResourceRequirement::Required,
    ///         cache_duration: None,
    ///     },
    /// ];
    ///
    /// let plan = api.generate_workplan(resources).await?;
    /// println!("Work plan has {} layers with {} total tasks",
    ///     plan.layers.len(), plan.total_tasks);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(num_resources = resources.len()))]
    pub async fn generate_workplan(&self, resources: Vec<Resource>) -> Result<WorkPlan> {
        info!("Generating work plan");
        // Build graphs for all resources and merge them
        let mut combined_graph: Option<DependencyGraph> = None;

        for resource in resources {
            let graph = self.graph(resource).await?;

            if let Some(ref mut combined) = combined_graph {
                // Merge graphs - add all nodes and edges
                for (hash, node) in graph.nodes {
                    combined.add_node(hash, node);
                }
                for edge in graph.edges {
                    combined.add_edge(edge.0, edge.1);
                }
            } else {
                combined_graph = Some(graph);
            }
        }

        match combined_graph {
            Some(graph) => {
                let plan = crate::graph::generate_workplan(&graph)?;
                info!("Work plan generated with {} layers and {} total tasks", plan.layers.len(), plan.total_tasks);
                Ok(plan)
            },
            None => Ok(WorkPlan::new()),
        }
    }

    /// Render resources to documents
    pub async fn render(
        &self,
        _resources: Vec<Resource>,
        _state: Option<Frontmatter>,
    ) -> Result<Vec<Document>> {
        todo!("Implement in Phase 5")
    }

    /// Convert markdown to HTML
    pub async fn to_html(&self, _patterns: Vec<String>) -> Result<Vec<HtmlOutput>> {
        todo!("Implement in Phase 5")
    }

    // ===== Supplemental API Functions =====

    /// Transclude a resource
    pub async fn transclude(&self, _resource: Resource) -> Result<Document> {
        todo!("Implement in Phase 5")
    }

    /// Optimize an image for responsive web delivery
    ///
    /// Processes an image to generate optimized variants at multiple breakpoint widths
    /// in modern formats (AVIF, WebP, JPEG/PNG). Automatically detects transparency
    /// and generates appropriate formats. Includes blur placeholder for progressive loading.
    ///
    /// # Arguments
    ///
    /// * `source` - The image source (local path or URL)
    ///
    /// # Returns
    ///
    /// A `SmartImageOutput` containing HTML with `<picture>` element and all variants.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use lib::{init, ImageSource};
    /// # use std::path::PathBuf;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = init(None, None).await?;
    /// let source = ImageSource::Local(PathBuf::from("photo.jpg"));
    ///
    /// let output = api.optimize_image(source).await?;
    /// println!("Generated {} variants", output.variants.len());
    /// println!("HTML: {}", output.html);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(source = ?source))]
    pub async fn optimize_image(&self, source: ImageSource) -> Result<SmartImageOutput> {
        use crate::image::{ImageOptions, get_or_process_image};
        use crate::image::html::HtmlOptions;

        info!("Optimizing image");
        let options = ImageOptions::default();
        let html_options = HtmlOptions::default();

        let result = get_or_process_image(&source, options, html_options, &self.db).await?;
        debug!("Image optimization complete");
        Ok(result)
    }

    /// Summarize a resource
    pub async fn summarize(&self, _resource: Resource) -> Result<String> {
        todo!("Implement in Phase 6")
    }

    /// Consolidate multiple resources
    pub async fn consolidate(&self, _resources: Vec<Resource>) -> Result<String> {
        todo!("Implement in Phase 6")
    }

    /// Extract topic from resources
    pub async fn topic_extraction(&self, _topic: &str, _resources: Vec<Resource>) -> Result<String> {
        todo!("Implement in Phase 6")
    }
}

// Re-export image types for convenience
pub use crate::image::{ImageSource, SmartImageOutput};

// Placeholder types for future implementation
#[derive(Debug, Clone)]
pub struct HtmlOutput {
    pub path: std::path::PathBuf,
    pub html: String,
}
