use crate::error::{CacheError, Result};
use std::path::{Path, PathBuf};
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;
use tracing::{info, instrument};

/// Initialize a SurrealDB database connection
#[instrument(skip_all, fields(path = %db_path.as_ref().display()))]
pub async fn init_database(db_path: impl AsRef<Path>) -> Result<Surreal<Db>> {
    let path = db_path.as_ref();

    info!("Initializing SurrealDB at {}", path.display());

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CacheError::InitializationFailed {
                    path: path.to_path_buf(),
                    error: format!("Failed to create parent directory: {}", e),
                }
            })?;
        }
    }

    // Connect to RocksDB backend
    let db = Surreal::new::<RocksDb>(path).await.map_err(|e| {
        CacheError::ConnectionFailed(format!("RocksDB connection failed: {}", e))
    })?;

    // Use default namespace and database
    db.use_ns("composition")
        .use_db("composition")
        .await
        .map_err(|e| CacheError::ConnectionFailed(format!("Failed to select namespace/database: {}", e)))?;

    info!("Database initialized successfully");

    Ok(db)
}

/// Locate or create the database file path based on project scope
#[instrument]
pub fn locate_database_path(start_dir: Option<&Path>) -> Result<PathBuf> {
    let start = start_dir
        .map(|p| p.to_path_buf())
        .or_else(|| std::env::current_dir().ok())
        .ok_or_else(|| CacheError::InitializationFailed {
            path: PathBuf::from("."),
            error: "Could not determine current directory".to_string(),
        })?;

    // Try to find git root
    if let Some(git_root) = find_git_root(&start) {
        info!("Found git repository at {}", git_root.display());
        return Ok(git_root.join(".composition.db"));
    }

    // If start_dir was explicitly provided and not in a git repo,
    // use it directly (useful for tests with temp directories)
    if start_dir.is_some() {
        info!("No git repository found, using provided directory");
        return Ok(start.join(".composition.db"));
    }

    // Fall back to home directory only if no start_dir was provided
    let home = dirs::home_dir().ok_or_else(|| CacheError::InitializationFailed {
        path: PathBuf::from("~"),
        error: "Could not determine home directory".to_string(),
    })?;

    info!("No git repository found, using home directory");
    Ok(home.join(".composition.db"))
}

/// Find the git root directory starting from the given path
fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = start;

    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() {
            return Some(current.to_path_buf());
        }

        current = current.parent()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_git_root() {
        // This test assumes we're running in a git repo
        let current = std::env::current_dir().unwrap();
        let git_root = find_git_root(&current);

        if git_root.is_some() {
            let root = git_root.unwrap();
            assert!(root.join(".git").exists());
        }
    }
}
