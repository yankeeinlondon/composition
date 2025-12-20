use crate::api::{CompositionApi, CompositionConfig};
use crate::cache::{apply_schema, init_database, locate_database_path};
use crate::error::Result;
use crate::types::Frontmatter;
use std::path::Path;
use tracing::{info, instrument};

/// Initialize the Composition library
///
/// # Arguments
///
/// * `dir` - Optional starting directory for project scope detection
/// * `frontmatter` - Optional initial frontmatter to merge with defaults
///
/// # Returns
///
/// A `CompositionApi` handle for interacting with the library
///
/// # Example
///
/// ```no_run
/// use lib::init;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let api = init(None, None).await?;
///     Ok(())
/// }
/// ```
#[instrument]
pub async fn init(
    dir: Option<&Path>,
    frontmatter: Option<Frontmatter>,
) -> Result<CompositionApi> {
    info!("Initializing Composition library");

    // Locate database path based on project scope
    let db_path = locate_database_path(dir)?;
    info!("Using database at: {}", db_path.display());

    // Determine project root (git root or current directory)
    let project_root = dir
        .map(|p| p.to_path_buf())
        .or_else(|| std::env::current_dir().ok());

    // Initialize database
    let db = init_database(&db_path).await?;

    // Apply schema
    apply_schema(&db).await?;

    // Merge frontmatter: ENV → utility defaults → passed frontmatter
    let mut merged_frontmatter = load_utility_frontmatter();
    merge_env_frontmatter(&mut merged_frontmatter);

    if let Some(user_frontmatter) = frontmatter {
        merged_frontmatter.merge(user_frontmatter);
    }

    // Create configuration
    let config = CompositionConfig {
        db_path,
        project_root,
    };

    // Create API instance
    let api = CompositionApi::new(db, merged_frontmatter, config).await?;

    info!("Composition library initialized successfully");
    Ok(api)
}

/// Load utility frontmatter defaults
fn load_utility_frontmatter() -> Frontmatter {
    let mut fm = Frontmatter::new();

    // Set default models (can be overridden by ENV or user)
    fm.summarize_model = Some("openai/gpt-4o-mini".to_string());
    fm.consolidate_model = Some("openai/gpt-4o".to_string());

    fm
}

/// Merge environment variable frontmatter
fn merge_env_frontmatter(frontmatter: &mut Frontmatter) {
    // Check for model overrides
    if let Ok(model) = std::env::var("COMPOSITION_SUMMARIZE_MODEL") {
        frontmatter.summarize_model = Some(model);
    }

    if let Ok(model) = std::env::var("COMPOSITION_CONSOLIDATE_MODEL") {
        frontmatter.consolidate_model = Some(model);
    }

    // Check for list expansion default
    if let Ok(expansion) = std::env::var("COMPOSITION_LIST_EXPANSION") {
        use crate::types::ListExpansion;
        frontmatter.list_expansion = match expansion.to_lowercase().as_str() {
            "expanded" => Some(ListExpansion::Expanded),
            "collapsed" => Some(ListExpansion::Collapsed),
            "none" => Some(ListExpansion::None),
            _ => None,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_utility_frontmatter() {
        let fm = load_utility_frontmatter();
        assert!(fm.summarize_model.is_some());
        assert!(fm.consolidate_model.is_some());
    }

    #[test]
    fn test_merge_env_frontmatter() {
        unsafe {
            std::env::set_var("COMPOSITION_SUMMARIZE_MODEL", "test/model");
        }

        let mut fm = Frontmatter::new();
        merge_env_frontmatter(&mut fm);

        assert_eq!(fm.summarize_model, Some("test/model".to_string()));

        unsafe {
            std::env::remove_var("COMPOSITION_SUMMARIZE_MODEL");
        }
    }
}
