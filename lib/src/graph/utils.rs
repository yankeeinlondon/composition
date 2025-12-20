use crate::error::{ParseError, Result};
use crate::types::{Resource, ResourceHash, ResourceSource};
use std::path::{Path, PathBuf};
use tracing::{debug, instrument};
use xxhash_rust::xxh3::xxh3_64;

/// Compute a hash for a resource (based on its source location)
#[instrument(skip(resource))]
pub fn compute_resource_hash(resource: &Resource) -> ResourceHash {
    let source_str = match &resource.source {
        ResourceSource::Local(path) => path.to_string_lossy().to_string(),
        ResourceSource::Remote(url) => url.to_string(),
    };

    xxh3_64(source_str.as_bytes())
}

/// Compute a hash for content (based on the actual bytes)
#[instrument(skip(content))]
pub fn compute_content_hash(content: &str) -> String {
    format!("{:016x}", xxh3_64(content.as_bytes()))
}

/// Load resource content from disk or network
#[instrument(skip_all, fields(source = ?resource.source))]
pub async fn load_resource(resource: &Resource) -> Result<String> {
    match &resource.source {
        ResourceSource::Local(path) => {
            debug!("Loading local file: {}", path.display());

            // Check if file is ignored by .gitignore
            // Determine project root: walk up from file path to find .git directory
            let project_root = find_project_root(path);
            if let Some(root) = project_root {
                if crate::graph::gitignore::is_ignored(path, &root)? {
                    return Err(crate::error::CompositionError::Parse(
                        ParseError::FileIgnored {
                            path: path.to_string_lossy().to_string(),
                        },
                    ));
                }
            }

            std::fs::read_to_string(path).map_err(|e| {
                crate::error::CompositionError::Parse(ParseError::ResourceNotFound {
                    path: path.to_string_lossy().to_string(),
                    error: e.to_string(),
                })
            })
        }
        ResourceSource::Remote(url) => {
            debug!("Fetching remote URL: {}", url);

            // For now, return an error - HTTP fetching will be implemented in Phase 5
            Err(crate::error::CompositionError::Parse(ParseError::UnsupportedFeature(
                format!("Remote resource loading not yet implemented: {}", url)
            )))
        }
    }
}

/// Find the project root by walking up from a path looking for .git directory
///
/// # Arguments
///
/// * `path` - Starting path (file or directory)
///
/// # Returns
///
/// * `Some(PathBuf)` - Path to project root (directory containing .git)
/// * `None` - No .git directory found
fn find_project_root(path: &Path) -> Option<PathBuf> {
    let mut current = path.to_path_buf();

    // If path is a file, start from its parent directory
    if current.is_file() {
        current = current.parent()?.to_path_buf();
    }

    loop {
        // Check if .git exists in current directory
        let git_dir = current.join(".git");
        if git_dir.exists() {
            return Some(current);
        }

        // Move to parent directory
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return None, // Reached filesystem root
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_compute_resource_hash() {
        let resource1 = Resource::local(PathBuf::from("/path/to/file.md"));
        let resource2 = Resource::local(PathBuf::from("/path/to/file.md"));
        let resource3 = Resource::local(PathBuf::from("/path/to/other.md"));

        let hash1 = compute_resource_hash(&resource1);
        let hash2 = compute_resource_hash(&resource2);
        let hash3 = compute_resource_hash(&resource3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_compute_content_hash() {
        let content1 = "Hello, world!";
        let content2 = "Hello, world!";
        let content3 = "Different content";

        let hash1 = compute_content_hash(content1);
        let hash2 = compute_content_hash(content2);
        let hash3 = compute_content_hash(content3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);

        // Should be hex string
        assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(hash1.len(), 16);
    }

    #[tokio::test]
    async fn test_load_resource_local_not_found() {
        let resource = Resource::local(PathBuf::from("/nonexistent/file.md"));
        let result = load_resource(&resource).await;

        assert!(result.is_err());
    }
}
