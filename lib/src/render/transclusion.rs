use crate::cache::CacheOperations;
use crate::error::RenderError;
use crate::parse::parse_document;
use crate::types::{DarkMatterNode, Frontmatter, LineRange, Resource, ResourceSource};
use std::fs;
use std::path::PathBuf;
use std::pin::Pin;
use std::future::Future;
use tracing::instrument;
use csv;

/// Resolve a resource path relative to a base path
fn resolve_resource_path(
    resource: &Resource,
    base_path: Option<&PathBuf>,
) -> Result<Resource, RenderError> {
    match &resource.source {
        ResourceSource::Local(path) if path.is_relative() => {
            if let Some(base) = base_path {
                let parent_dir = base.parent()
                    .ok_or_else(|| RenderError::InvalidPath(base.display().to_string()))?;

                let mut resolved_path = parent_dir.join(path);

                // Normalize path by removing "." components
                let normalized: PathBuf = resolved_path
                    .components()
                    .filter(|c| c.as_os_str() != ".")
                    .collect();
                resolved_path = normalized;

                Ok(Resource {
                    source: ResourceSource::Local(resolved_path),
                    requirement: resource.requirement,
                    cache_duration: resource.cache_duration,
                })
            } else {
                // No base path, resolve relative to current directory
                let cwd = std::env::current_dir()
                    .map_err(|e| RenderError::IoError(e.to_string()))?;
                let resolved_path = cwd.join(path);

                Ok(Resource {
                    source: ResourceSource::Local(resolved_path),
                    requirement: resource.requirement,
                    cache_duration: resource.cache_duration,
                })
            }
        }
        // Already absolute or remote - return as-is
        _ => Ok(resource.clone())
    }
}

/// Resolve a transclusion directive by loading and parsing the referenced resource
///
/// This function:
/// 1. Resolves relative resource paths
/// 2. Loads the resource (from cache for remote, or filesystem for local)
/// 3. Applies line range filtering if specified
/// 4. Parses the transcluded content as a DarkMatter document
/// 5. Recursively resolves nested transclusions
/// 6. Resolves external table sources to inline tables
#[instrument(skip(_cache, _frontmatter))]
pub fn resolve_transclusion<'a>(
    node: &'a DarkMatterNode,
    _frontmatter: &'a Frontmatter,
    _cache: &'a CacheOperations,
    base_path: Option<&'a PathBuf>,
) -> Pin<Box<dyn Future<Output = Result<Vec<DarkMatterNode>, RenderError>> + Send + 'a>> {
    Box::pin(async move {
    match node {
        DarkMatterNode::File { resource, range } => {
            // 1. Resolve the resource path if relative
            let resolved_resource = resolve_resource_path(resource, base_path)?;

            // 2. Load resource content using the resolved path
            let content = load_resource(&resolved_resource, _cache, None).await?;

            // 3. Apply line range if specified
            let content = apply_line_range(&content, range)?;

            // 4. Parse the transcluded content
            let doc = parse_document(&content, resolved_resource.clone())
                .map_err(|e| RenderError::ParseError(e.to_string()))?;

            // 5. Recursively resolve transclusions in the transcluded content
            //    Now use the resolved resource as the base path
            let mut resolved = Vec::new();
            for child in &doc.content {
                let resolved_children = resolve_transclusion(
                    child,
                    &doc.frontmatter,
                    _cache,
                    extract_base_path(&resolved_resource),
                )
                .await?;
                resolved.extend(resolved_children);
            }

            Ok(resolved)
        }
        DarkMatterNode::Table { source, has_heading } => {
            // Resolve external table sources to inline tables
            use crate::types::TableSource;

            match source {
                TableSource::External(resource) => {
                    // Resolve the resource path if relative
                    let resolved_resource = resolve_resource_path(resource, base_path)?;

                    // Load and parse CSV
                    let csv_data = load_csv_data(&resolved_resource).await?;

                    // Return as inline table
                    Ok(vec![DarkMatterNode::Table {
                        source: TableSource::Inline(csv_data),
                        has_heading: *has_heading,
                    }])
                }
                TableSource::Inline(_) => {
                    // Already inline, pass through
                    Ok(vec![node.clone()])
                }
            }
        }
        // Pass through other nodes unchanged
        other => Ok(vec![other.clone()]),
    }
    })
}

/// Load resource content from filesystem or cache
async fn load_resource(
    resource: &Resource,
    _cache: &CacheOperations,
    base_path: Option<&PathBuf>,
) -> Result<String, RenderError> {
    match &resource.source {
        ResourceSource::Local(path) => {
            // Resolve relative paths
            let mut full_path = if path.is_absolute() {
                path.clone()
            } else if let Some(base) = base_path {
                base.parent()
                    .ok_or_else(|| RenderError::InvalidPath(base.display().to_string()))?
                    .join(path)
            } else {
                std::env::current_dir()
                    .map_err(|e| RenderError::IoError(e.to_string()))?
                    .join(path)
            };

            // Normalize path by removing "." components
            let normalized: PathBuf = full_path
                .components()
                .filter(|c| c.as_os_str() != ".")
                .collect();
            full_path = normalized;

            // Read from filesystem
            fs::read_to_string(&full_path)
                .map_err(|e| RenderError::ResourceNotFound(full_path.display().to_string(), e.to_string()))
        }
        ResourceSource::Remote(url) => {
            // Check cache first
            let url_str = url.to_string();

            // For now, we'll use reqwest to fetch remote content
            // In a full implementation, this would check cache first
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

/// Apply line range filtering to content
fn apply_line_range(content: &str, range: &Option<LineRange>) -> Result<String, RenderError> {
    let Some(range) = range else {
        return Ok(content.to_string());
    };

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    // Validate start line
    if range.start == 0 {
        return Err(RenderError::InvalidLineRange(
            "Line numbers are 1-indexed, cannot start at 0".to_string(),
        ));
    }

    if range.start > total_lines {
        return Err(RenderError::InvalidLineRange(format!(
            "Start line {} exceeds document length {}",
            range.start, total_lines
        )));
    }

    // Determine end line
    let end = range.end.unwrap_or(total_lines);

    if end < range.start {
        return Err(RenderError::InvalidLineRange(format!(
            "End line {} is before start line {}",
            end, range.start
        )));
    }

    if end > total_lines {
        return Err(RenderError::InvalidLineRange(format!(
            "End line {} exceeds document length {}",
            end, total_lines
        )));
    }

    // Extract range (converting from 1-indexed to 0-indexed)
    let selected_lines = &lines[(range.start - 1)..end];
    Ok(selected_lines.join("\n"))
}

/// Extract base path from a resource for resolving relative paths
fn extract_base_path(resource: &Resource) -> Option<&PathBuf> {
    match &resource.source {
        ResourceSource::Local(path) => Some(path),
        ResourceSource::Remote(_) => None,
    }
}

/// Load and parse CSV data from a resource
async fn load_csv_data(resource: &Resource) -> Result<Vec<Vec<String>>, RenderError> {
    // Load the CSV content
    let content = match &resource.source {
        ResourceSource::Local(path) => {
            fs::read_to_string(path)
                .map_err(|e| RenderError::ResourceNotFound(
                    path.display().to_string(),
                    e.to_string()
                ))?
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
                .map_err(|e| RenderError::RemoteFetchError(url_str, e.to_string()))?
        }
    };

    // Parse the CSV
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(content.as_bytes());

    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| RenderError::CsvError(e.to_string()))?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        rows.push(row);
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_line_range_full() {
        let content = "line1\nline2\nline3\nline4";
        let result = apply_line_range(content, &None).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_apply_line_range_partial() {
        let content = "line1\nline2\nline3\nline4";
        let range = Some(LineRange {
            start: 2,
            end: Some(3),
        });
        let result = apply_line_range(content, &range).unwrap();
        assert_eq!(result, "line2\nline3");
    }

    #[test]
    fn test_apply_line_range_from_start() {
        let content = "line1\nline2\nline3\nline4";
        let range = Some(LineRange {
            start: 1,
            end: Some(2),
        });
        let result = apply_line_range(content, &range).unwrap();
        assert_eq!(result, "line1\nline2");
    }

    #[test]
    fn test_apply_line_range_to_end() {
        let content = "line1\nline2\nline3\nline4";
        let range = Some(LineRange {
            start: 3,
            end: None,
        });
        let result = apply_line_range(content, &range).unwrap();
        assert_eq!(result, "line3\nline4");
    }

    #[test]
    fn test_apply_line_range_invalid_zero() {
        let content = "line1\nline2";
        let range = Some(LineRange {
            start: 0,
            end: Some(1),
        });
        let result = apply_line_range(content, &range);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_line_range_out_of_bounds() {
        let content = "line1\nline2";
        let range = Some(LineRange {
            start: 1,
            end: Some(10),
        });
        let result = apply_line_range(content, &range);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_line_range_reversed() {
        let content = "line1\nline2\nline3";
        let range = Some(LineRange {
            start: 3,
            end: Some(1),
        });
        let result = apply_line_range(content, &range);
        assert!(result.is_err());
    }
}
