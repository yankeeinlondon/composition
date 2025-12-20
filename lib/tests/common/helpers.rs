#![allow(dead_code)]

use lib::cache::init_database;
use lib::error::Result;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use tempfile::TempDir;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Initialize a test database in a temporary directory with unique name
pub async fn init_test_db() -> Result<(Surreal<Db>, TempDir)> {
    let counter = TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join(format!("test_{}.db", counter));
    let db = init_database(&db_path).await?;
    Ok((db, temp_dir))
}

/// Compute a simple hash for testing (using XXH3)
pub fn compute_test_hash(data: &str) -> String {
    use xxhash_rust::xxh3::xxh3_64;
    format!("{:016x}", xxh3_64(data.as_bytes()))
}
