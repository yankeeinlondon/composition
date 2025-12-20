mod common;

use chrono::Utc;
use common::*;
use lib::cache::{apply_schema, DocumentCacheEntry, ImageCacheEntry, LlmCacheEntry};
use lib::init;

/// Test basic database initialization
#[tokio::test]
async fn test_init_database() {
    let (db, _temp_dir) = init_test_db().await.unwrap();

    // Verify we can execute a simple query
    let result = db.query("SELECT * FROM document").await;
    assert!(result.is_ok());
}

/// Test database schema application
#[tokio::test]
async fn test_schema_application() {
    let (db, _temp_dir) = init_test_db().await.unwrap();

    // Apply schema
    apply_schema(&db).await.unwrap();

    // Verify tables exist by querying them
    let result = db.query("SELECT * FROM document").await;
    assert!(result.is_ok());

    let result = db.query("SELECT * FROM image_cache").await;
    assert!(result.is_ok());

    let result = db.query("SELECT * FROM llm_cache").await;
    assert!(result.is_ok());
}

/// Test init function with temporary directory
#[tokio::test]
async fn test_init_function() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static INIT_COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = INIT_COUNTER.fetch_add(1, Ordering::SeqCst);

    let temp_dir = temp_dir();
    // Create a unique subdirectory to avoid lock conflicts
    let unique_dir = temp_dir.path().join(format!("test_init_{}", counter));
    std::fs::create_dir_all(&unique_dir).unwrap();

    // Create a fake .git directory so init treats this as a repo root
    // Otherwise it falls back to ~/.composition.db which causes lock conflicts
    std::fs::create_dir_all(unique_dir.join(".git")).unwrap();

    // Initialize with temp directory
    let api = init(Some(&unique_dir), None).await.unwrap();

    // Verify we can access the database
    assert!(api.db().query("SELECT * FROM document").await.is_ok());

    // Verify default frontmatter is set
    assert!(api.frontmatter().summarize_model.is_some());
    assert!(api.frontmatter().consolidate_model.is_some());
}

/// Test init function with custom frontmatter
#[tokio::test]
async fn test_init_with_custom_frontmatter() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static CUSTOM_FM_COUNTER: AtomicU64 = AtomicU64::new(100);
    let counter = CUSTOM_FM_COUNTER.fetch_add(1, Ordering::SeqCst);

    let temp_dir = temp_dir();
    let unique_dir = temp_dir.path().join(format!("test_custom_fm_{}", counter));
    std::fs::create_dir_all(&unique_dir).unwrap();

    // Create a fake .git directory so init treats this as a repo root
    std::fs::create_dir_all(unique_dir.join(".git")).unwrap();

    let mut fm = test_frontmatter();
    fm.summarize_model = Some("custom/model".to_string());

    let api = init(Some(&unique_dir), Some(fm)).await.unwrap();

    assert_eq!(
        api.frontmatter().summarize_model,
        Some("custom/model".to_string())
    );
}

/// Test document cache operations
#[tokio::test]
async fn test_document_cache_operations() {
    let (db, _temp_dir) = init_test_db().await.unwrap();
    apply_schema(&db).await.unwrap();

    let cache = lib::cache::CacheOperations::new(db);

    // Create a test entry
    let entry = DocumentCacheEntry {
        id: None,
        resource_hash: "test_hash_123".to_string(),
        content_hash: "content_abc".to_string(),
        file_path: Some("/tmp/test.md".to_string()),
        url: None,
        last_validated: Utc::now(),
    };

    // Upsert
    cache.upsert_document(entry.clone()).await.unwrap();

    // Get
    let retrieved = cache
        .get_document(&entry.resource_hash)
        .await
        .unwrap();

    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.resource_hash, entry.resource_hash);
    assert_eq!(retrieved.content_hash, entry.content_hash);
}

/// Test image cache operations
#[tokio::test]
async fn test_image_cache_operations() {
    let (db, _temp_dir) = init_test_db().await.unwrap();
    apply_schema(&db).await.unwrap();

    let cache = lib::cache::CacheOperations::new(db);

    // Create a test entry
    let entry = ImageCacheEntry {
        id: None,
        resource_hash: "image_hash_456".to_string(),
        content_hash: "image_content_def".to_string(),
        created_at: Utc::now(),
        expires_at: None,
        source_type: "local".to_string(),
        source: "/tmp/image.png".to_string(),
        has_transparency: true,
        original_width: 1920,
        original_height: 1080,
    };

    // Upsert
    cache.upsert_image(entry.clone()).await.unwrap();

    // Get
    let retrieved = cache.get_image(&entry.resource_hash).await.unwrap();

    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.resource_hash, entry.resource_hash);
    assert_eq!(retrieved.has_transparency, true);
    assert_eq!(retrieved.original_width, 1920);
}

/// Test LLM cache operations
#[tokio::test]
async fn test_llm_cache_operations() {
    let (db, _temp_dir) = init_test_db().await.unwrap();
    apply_schema(&db).await.unwrap();

    let cache = lib::cache::CacheOperations::new(db);

    // Create a test entry
    let entry = LlmCacheEntry {
        id: None,
        operation: "summarize".to_string(),
        input_hash: "input_hash_789".to_string(),
        model: "test/model".to_string(),
        response: "This is a test summary".to_string(),
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::days(30),
        tokens_used: Some(100),
    };

    // Upsert
    cache.upsert_llm(entry.clone()).await.unwrap();

    // Get
    let retrieved = cache
        .get_llm(&entry.operation, &entry.input_hash, &entry.model)
        .await
        .unwrap();

    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.operation, entry.operation);
    assert_eq!(retrieved.response, entry.response);
    assert_eq!(retrieved.tokens_used, Some(100));
}

/// Test LLM cache expiration
#[tokio::test]
async fn test_llm_cache_expiration() {
    let (db, _temp_dir) = init_test_db().await.unwrap();
    apply_schema(&db).await.unwrap();

    let cache = lib::cache::CacheOperations::new(db);

    // Create an expired entry
    let entry = LlmCacheEntry {
        id: None,
        operation: "summarize".to_string(),
        input_hash: "expired_hash".to_string(),
        model: "test/model".to_string(),
        response: "This should be expired".to_string(),
        created_at: Utc::now() - chrono::Duration::days(60),
        expires_at: Utc::now() - chrono::Duration::days(30),
        tokens_used: None,
    };

    // Upsert
    cache.upsert_llm(entry.clone()).await.unwrap();

    // Try to get (should return None because it's expired)
    let retrieved = cache
        .get_llm(&entry.operation, &entry.input_hash, &entry.model)
        .await
        .unwrap();

    assert!(retrieved.is_none());
}

/// Test cache invalidation
#[tokio::test]
async fn test_cache_invalidation() {
    let (db, _temp_dir) = init_test_db().await.unwrap();
    apply_schema(&db).await.unwrap();

    let cache = lib::cache::CacheOperations::new(db);

    // Create and insert an image entry
    let entry = ImageCacheEntry {
        id: None,
        resource_hash: "invalidate_test".to_string(),
        content_hash: "content".to_string(),
        created_at: Utc::now(),
        expires_at: None,
        source_type: "local".to_string(),
        source: "/tmp/test.png".to_string(),
        has_transparency: false,
        original_width: 100,
        original_height: 100,
    };

    cache.upsert_image(entry.clone()).await.unwrap();

    // Verify it exists
    let retrieved = cache.get_image(&entry.resource_hash).await.unwrap();
    assert!(retrieved.is_some());

    // Invalidate
    cache.invalidate_image(&entry.resource_hash).await.unwrap();

    // Verify it's gone
    let retrieved = cache.get_image(&entry.resource_hash).await.unwrap();
    assert!(retrieved.is_none());
}

/// Test project scope detection (git vs non-git)
#[tokio::test]
async fn test_project_scope_detection() {
    use lib::cache::locate_database_path;

    // Test with temp directory (non-git)
    let temp_dir = temp_dir();
    let db_path = locate_database_path(Some(temp_dir.path())).unwrap();

    // Should use home directory for non-git projects
    assert!(db_path.to_string_lossy().contains(".composition.db"));
}

/// Test frontmatter merging
#[test]
fn test_frontmatter_merge() {
    let mut base = lib::types::Frontmatter::new();
    base.summarize_model = Some("base/model".to_string());
    base.custom.insert("key1".to_string(), serde_json::json!("value1"));

    let mut override_fm = lib::types::Frontmatter::new();
    override_fm.summarize_model = Some("override/model".to_string());
    override_fm.custom.insert("key2".to_string(), serde_json::json!("value2"));

    base.merge(override_fm);

    // Override takes precedence
    assert_eq!(base.summarize_model, Some("override/model".to_string()));

    // Both custom keys present
    assert!(base.custom.contains_key("key1"));
    assert!(base.custom.contains_key("key2"));
}

/// Test resource creation
#[test]
fn test_resource_creation() {
    use lib::types::{ResourceRequirement, ResourceSource};
    use std::path::PathBuf;

    let resource = test_local_resource("/tmp/test.md");

    match resource.source {
        ResourceSource::Local(path) => assert_eq!(path, PathBuf::from("/tmp/test.md")),
        _ => panic!("Expected local resource"),
    }

    assert!(matches!(resource.requirement, ResourceRequirement::Default));
}

/// Test hash computation helper
#[test]
fn test_hash_computation() {
    let hash1 = compute_test_hash("test data");
    let hash2 = compute_test_hash("test data");
    let hash3 = compute_test_hash("different data");

    // Same data produces same hash
    assert_eq!(hash1, hash2);

    // Different data produces different hash
    assert_ne!(hash1, hash3);
}
