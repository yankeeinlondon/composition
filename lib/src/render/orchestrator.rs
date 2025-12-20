use crate::cache::CacheOperations;
use crate::error::RenderError;
use crate::parse::parse_document;
use crate::types::{Document, Frontmatter, Resource, WorkPlan};
use std::sync::Arc;
use tracing::{info, instrument, span, Level};

use super::interpolation::process_nodes_interpolation;
use super::transclusion::resolve_transclusion;

/// Orchestrate the rendering of documents according to a work plan
///
/// This function:
/// 1. Processes work plan layers in order
/// 2. Parallelizes independent resources within each layer using rayon
/// 3. Resolves transclusions recursively
/// 4. Applies frontmatter interpolation
/// 5. Reports progress via tracing
#[instrument(skip(plan, frontmatter, cache))]
pub async fn execute_workplan(
    plan: &WorkPlan,
    frontmatter: &Frontmatter,
    cache: &Arc<CacheOperations>,
) -> Result<Vec<Document>, RenderError> {
    let mut results = Vec::new();
    let total_layers = plan.layers.len();

    info!(
        "Executing work plan with {} layers and {} total tasks",
        total_layers, plan.total_tasks
    );

    for (layer_idx, layer) in plan.layers.iter().enumerate() {
        let span = span!(Level::INFO, "layer", index = layer_idx, count = layer.resources.len());
        let _enter = span.enter();

        info!(
            "Processing layer {}/{} with {} resources (parallelizable: {})",
            layer_idx + 1,
            total_layers,
            layer.resources.len(),
            layer.parallelizable
        );

        if layer.parallelizable && layer.resources.len() > 1 {
            // Use tokio for parallel processing with join_all
            let mut tasks = Vec::new();

            for resource in &layer.resources {
                let fm = frontmatter.clone();
                let cache_ref = Arc::clone(cache);
                let resource = resource.clone();

                let task = tokio::spawn(async move {
                    render_document(&resource, &fm, &cache_ref).await
                });

                tasks.push(task);
            }

            // Wait for all tasks to complete
            let layer_results = futures::future::join_all(tasks).await;

            // Collect results and handle errors
            for result in layer_results {
                let doc = result
                    .map_err(|e| RenderError::HtmlGenerationFailed(format!("Task join error: {}", e)))??;
                results.push(doc);
            }
        } else {
            // Process sequentially
            for resource in &layer.resources {
                let doc = render_document(resource, frontmatter, cache).await?;
                results.push(doc);
            }
        }

        info!("Completed layer {}/{}", layer_idx + 1, total_layers);
    }

    info!("Work plan execution complete. Rendered {} documents", results.len());
    Ok(results)
}

/// Render a single document
///
/// This function:
/// 1. Loads and parses the document
/// 2. Resolves all transclusions recursively
/// 3. Applies frontmatter interpolation
/// 4. Returns the fully resolved document
#[instrument(skip(frontmatter, cache))]
async fn render_document(
    resource: &Resource,
    frontmatter: &Frontmatter,
    cache: &CacheOperations,
) -> Result<Document, RenderError> {
    info!("Rendering document: {:?}", resource.source);

    // 1. Load and parse the document
    let content = load_resource_content(resource, cache).await?;
    let mut doc = parse_document(&content, resource.clone())
        .map_err(|e| RenderError::ParseError(e.to_string()))?;

    // 2. Merge frontmatter
    let mut merged_frontmatter = frontmatter.clone();
    merged_frontmatter.merge(doc.frontmatter.clone());

    // 3. Resolve transclusions recursively
    let mut resolved_nodes = Vec::new();
    for node in &doc.content {
        let resolved = resolve_transclusion(
            node,
            &merged_frontmatter,
            cache,
            extract_base_path(resource),
        )
        .await?;
        resolved_nodes.extend(resolved);
    }

    // 4. Apply frontmatter interpolation
    let interpolated_nodes = process_nodes_interpolation(&resolved_nodes, &merged_frontmatter)
        .map_err(|e| RenderError::HtmlGenerationFailed(e.to_string()))?;

    // 5. Update document with processed content
    doc.content = interpolated_nodes;
    doc.frontmatter = merged_frontmatter;

    Ok(doc)
}

/// Load resource content (similar to transclusion but without parsing)
async fn load_resource_content(
    resource: &Resource,
    _cache: &CacheOperations,
) -> Result<String, RenderError> {
    use crate::types::ResourceSource;
    use std::fs;

    match &resource.source {
        ResourceSource::Local(path) => {
            fs::read_to_string(path)
                .map_err(|e| RenderError::ResourceNotFound(
                    path.display().to_string(),
                    e.to_string()
                ))
        }
        ResourceSource::Remote(url) => {
            let url_str = url.to_string();
            let response = reqwest::get(url.clone())
                .await
                .map_err(|e| RenderError::RemoteFetchError(url_str.clone(), e.to_string()))?;

            if !response.status().is_success() {
                return Err(RenderError::RemoteFetchError(
                    url_str,
                    format!("HTTP {}", response.status()),
                ));
            }

            response
                .text()
                .await
                .map_err(|e| RenderError::RemoteFetchError(url_str, e.to_string()))
        }
    }
}

/// Extract base path from resource for relative path resolution
fn extract_base_path(resource: &Resource) -> Option<&std::path::PathBuf> {
    use crate::types::ResourceSource;
    match &resource.source {
        ResourceSource::Local(path) => Some(path),
        ResourceSource::Remote(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ResourceSource;
    use std::path::PathBuf;

    #[test]
    fn test_extract_base_path_local() {
        let resource = Resource {
            source: ResourceSource::Local(PathBuf::from("/tmp/test.md")),
            requirement: Default::default(),
            cache_duration: None,
        };

        let base = extract_base_path(&resource);
        assert!(base.is_some());
        assert_eq!(base.unwrap(), &PathBuf::from("/tmp/test.md"));
    }

    #[test]
    fn test_extract_base_path_remote() {
        use url::Url;

        let resource = Resource {
            source: ResourceSource::Remote(Url::parse("https://example.com/test.md").unwrap()),
            requirement: Default::default(),
            cache_duration: None,
        };

        let base = extract_base_path(&resource);
        assert!(base.is_none());
    }

    // Note: Full integration tests for execute_workplan would require
    // setting up test fixtures and a database, which is better suited
    // for integration tests in the tests/ directory
}
