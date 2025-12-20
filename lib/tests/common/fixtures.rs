#![allow(dead_code)]

use lib::types::{Frontmatter, Resource};
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a temporary directory for testing
pub fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp dir")
}

/// Create a test resource pointing to a local file
pub fn test_local_resource(path: &str) -> Resource {
    Resource::local(PathBuf::from(path))
}

/// Create a test resource pointing to a remote URL
pub fn test_remote_resource(url: &str) -> Resource {
    let parsed_url = url::Url::parse(url).expect("Invalid URL");
    Resource::remote(parsed_url)
}

/// Create test frontmatter with common defaults
pub fn test_frontmatter() -> Frontmatter {
    let mut fm = Frontmatter::new();
    fm.summarize_model = Some("test/model".to_string());
    fm.consolidate_model = Some("test/model".to_string());
    fm
}

/// Create a test markdown file in a temporary directory
pub fn create_test_markdown_file(
    dir: &TempDir,
    name: &str,
    content: &str,
) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).expect("Failed to write test file");
    path
}
