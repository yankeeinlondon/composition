//! Phase 0.1: SurrealDB Embedded Mode Validation
//!
//! This spike validates that SurrealDB can:
//! - Initialize with embedded RocksDB backend
//! - Handle concurrent read/write operations
//! - Create and traverse graph edges
//! - Recover database file and persist data
//! - Perform basic CRUD operations efficiently

use serde::{Deserialize, Serialize};
use surrealdb::engine::local::RocksDb;
use surrealdb::Surreal;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Document {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<surrealdb::sql::Thing>,
    resource_hash: String,
    content_hash: String,
    file_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DependsOn {
    reference_type: String,
    required: bool,
}

/// Task 0.1.1: Create prototype with embedded RocksDB backend
#[tokio::test]
async fn test_initialize_embedded_db() -> Result<(), Box<dyn std::error::Error>> {
    // Create temp directory for database
    let temp_dir = std::env::temp_dir().join("composition-spike-db");

    // Clean up from previous runs
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)?;
    }

    // Initialize SurrealDB with embedded RocksDB
    let db = Surreal::new::<RocksDb>(temp_dir.clone()).await?;

    // Use namespace and database
    db.use_ns("test").use_db("composition").await?;

    println!("✓ Successfully initialized embedded RocksDB at {:?}", temp_dir);

    // Verify the database directory was created
    assert!(temp_dir.exists(), "Database directory should be created");

    // Clean up
    drop(db);
    std::fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Task 0.1.2: Test concurrent read/write operations
#[tokio::test]
async fn test_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("composition-spike-concurrent");

    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)?;
    }

    let db = Surreal::new::<RocksDb>(temp_dir.clone()).await?;
    db.use_ns("test").use_db("composition").await?;

    // Define schema
    db.query("DEFINE TABLE document SCHEMAFULL;").await?;
    db.query("DEFINE FIELD resource_hash ON document TYPE string;").await?;
    db.query("DEFINE FIELD content_hash ON document TYPE string;").await?;
    db.query("DEFINE FIELD file_path ON document TYPE option<string>;").await?;

    // Test concurrent writes
    let mut handles = vec![];
    let db_ref = std::sync::Arc::new(db);

    for i in 0..1000 {
        let db_clone = db_ref.clone();
        let handle = tokio::spawn(async move {
            let doc = Document {
                id: None,
                resource_hash: format!("hash_{}", i),
                content_hash: format!("content_{}", i),
                file_path: Some(format!("file_{}.md", i)),
            };

            let result: Result<Vec<Document>, surrealdb::Error> = db_clone
                .create("document")
                .content(doc)
                .await;

            result.expect("Insert should succeed")
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await?;
    }

    println!("✓ Successfully completed 1000 concurrent write operations");

    // Verify all records were inserted
    let records: Vec<Document> = db_ref.select("document").await?;
    assert_eq!(records.len(), 1000, "All 1000 records should be inserted");

    println!("✓ All 1000 records verified in database");

    // Test concurrent reads
    let mut read_handles = vec![];
    for i in 0..100 {
        let db_clone = db_ref.clone();
        let handle = tokio::spawn(async move {
            let query = format!("SELECT * FROM document WHERE resource_hash = 'hash_{}'", i);
            let mut result = db_clone.query(query).await.expect("Query should succeed");
            let docs: Vec<Document> = result.take(0).expect("Should get results");
            assert_eq!(docs.len(), 1, "Should find exactly one document");
        });
        read_handles.push(handle);
    }

    for handle in read_handles {
        handle.await?;
    }

    println!("✓ Successfully completed 100 concurrent read operations");

    // Clean up
    drop(db_ref);
    std::fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Task 0.1.3: Verify graph edge creation and traversal
#[tokio::test]
async fn test_graph_edges() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("composition-spike-graph");

    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)?;
    }

    let db = Surreal::new::<RocksDb>(temp_dir.clone()).await?;
    db.use_ns("test").use_db("composition").await?;

    // Define document table
    db.query("DEFINE TABLE document SCHEMAFULL;").await?;
    db.query("DEFINE FIELD resource_hash ON document TYPE string;").await?;
    db.query("DEFINE FIELD content_hash ON document TYPE string;").await?;
    db.query("DEFINE FIELD file_path ON document TYPE option<string>;").await?;

    // Define relation table for graph edges
    db.query("DEFINE TABLE depends_on SCHEMAFULL;").await?;
    db.query("DEFINE FIELD in ON depends_on TYPE record<document>;").await?;
    db.query("DEFINE FIELD out ON depends_on TYPE record<document>;").await?;
    db.query("DEFINE FIELD reference_type ON depends_on TYPE string;").await?;
    db.query("DEFINE FIELD required ON depends_on TYPE bool DEFAULT false;").await?;

    // Create documents
    let doc_a: Vec<Document> = db
        .create("document")
        .content(Document {
            id: None,
            resource_hash: "doc_a".to_string(),
            content_hash: "content_a".to_string(),
            file_path: Some("a.md".to_string()),
        })
        .await?;

    let doc_b: Vec<Document> = db
        .create("document")
        .content(Document {
            id: None,
            resource_hash: "doc_b".to_string(),
            content_hash: "content_b".to_string(),
            file_path: Some("b.md".to_string()),
        })
        .await?;

    let doc_c: Vec<Document> = db
        .create("document")
        .content(Document {
            id: None,
            resource_hash: "doc_c".to_string(),
            content_hash: "content_c".to_string(),
            file_path: Some("c.md".to_string()),
        })
        .await?;

    println!("✓ Created 3 document nodes");

    // Get record IDs
    let doc_a_id = doc_a[0].id.as_ref().unwrap();
    let doc_b_id = doc_b[0].id.as_ref().unwrap();
    let doc_c_id = doc_c[0].id.as_ref().unwrap();

    // Create graph edges: A -> B, A -> C
    let relation_query = format!(
        "RELATE {}->depends_on->{} CONTENT {{ reference_type: 'transclusion', required: true }};",
        doc_a_id, doc_b_id
    );
    db.query(relation_query).await?;

    let relation_query = format!(
        "RELATE {}->depends_on->{} CONTENT {{ reference_type: 'image', required: false }};",
        doc_a_id, doc_c_id
    );
    db.query(relation_query).await?;

    println!("✓ Created 2 graph edges");

    // Traverse graph - find all dependencies of doc_a
    let traversal_query = format!(
        "SELECT ->depends_on->document.* AS dependencies FROM document WHERE resource_hash = '{}'",
        doc_a[0].resource_hash
    );
    let mut result = db.query(traversal_query).await?;
    let traversal_result: Vec<serde_json::Value> = result.take(0)?;

    assert!(!traversal_result.is_empty(), "Should find dependencies");
    println!("✓ Successfully traversed graph edges");

    // Test reverse traversal - find what depends on doc_b
    let reverse_query = format!(
        "SELECT <-depends_on<-document.* AS dependents FROM document WHERE resource_hash = '{}'",
        doc_b[0].resource_hash
    );
    let mut result = db.query(reverse_query).await?;
    let reverse_result: Vec<serde_json::Value> = result.take(0)?;

    assert!(!reverse_result.is_empty(), "Should find dependents");
    println!("✓ Successfully traversed graph in reverse");

    // Clean up
    drop(db);
    std::fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Task 0.1.4: Test database file creation and recovery
#[tokio::test]
async fn test_database_recovery() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("composition-spike-recovery");

    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)?;
    }

    // Phase 1: Create database and insert data
    {
        let db = Surreal::new::<RocksDb>(temp_dir.clone()).await?;
        db.use_ns("test").use_db("composition").await?;

        db.query("DEFINE TABLE document SCHEMAFULL;").await?;
        db.query("DEFINE FIELD resource_hash ON document TYPE string;").await?;
        db.query("DEFINE FIELD content_hash ON document TYPE string;").await?;
        db.query("DEFINE FIELD file_path ON document TYPE option<string>;").await?;

        let doc = Document {
            id: None,
            resource_hash: "persistent_doc".to_string(),
            content_hash: "persistent_content".to_string(),
            file_path: Some("persistent.md".to_string()),
        };

        let _result: Vec<Document> = db.create("document").content(doc).await?;

        println!("✓ Created database and inserted test document");

        // Explicitly drop to ensure flush
        drop(db);
    }

    // Wait a moment for the lock to be fully released
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify database files exist
    assert!(temp_dir.exists(), "Database directory should exist");
    println!("✓ Database files persisted to disk");

    // Phase 2: Reconnect and verify data persists
    {
        let db = Surreal::new::<RocksDb>(temp_dir.clone()).await?;
        db.use_ns("test").use_db("composition").await?;

        let records: Vec<Document> = db.select("document").await?;

        assert_eq!(records.len(), 1, "Should recover 1 document");
        assert_eq!(records[0].resource_hash, "persistent_doc");
        assert_eq!(records[0].content_hash, "persistent_content");

        println!("✓ Successfully recovered data from persisted database");

        drop(db);
    }

    // Clean up
    std::fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Task 0.1.5: Benchmark basic CRUD operations
#[tokio::test]
async fn test_crud_performance() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("composition-spike-benchmark");

    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)?;
    }

    let db = Surreal::new::<RocksDb>(temp_dir.clone()).await?;
    db.use_ns("test").use_db("composition").await?;

    db.query("DEFINE TABLE document SCHEMAFULL;").await?;
    db.query("DEFINE FIELD resource_hash ON document TYPE string;").await?;
    db.query("DEFINE FIELD content_hash ON document TYPE string;").await?;
    db.query("DEFINE FIELD file_path ON document TYPE option<string>;").await?;
    db.query("DEFINE INDEX idx_resource_hash ON document FIELDS resource_hash UNIQUE;").await?;

    let start = std::time::Instant::now();

    // Create 100 documents
    for i in 0..100 {
        let doc = Document {
            id: None,
            resource_hash: format!("bench_hash_{}", i),
            content_hash: format!("bench_content_{}", i),
            file_path: Some(format!("bench_{}.md", i)),
        };
        let _result: Vec<Document> = db.create("document").content(doc).await?;
    }

    let create_duration = start.elapsed();
    println!("✓ CREATE: 100 documents in {:?} ({:.2} ops/sec)",
        create_duration,
        100.0 / create_duration.as_secs_f64()
    );

    // Read 100 documents
    let start = std::time::Instant::now();
    for i in 0..100 {
        let query = format!("SELECT * FROM document WHERE resource_hash = 'bench_hash_{}'", i);
        let mut result = db.query(query).await?;
        let _docs: Vec<Document> = result.take(0)?;
    }
    let read_duration = start.elapsed();
    println!("✓ READ: 100 queries in {:?} ({:.2} ops/sec)",
        read_duration,
        100.0 / read_duration.as_secs_f64()
    );

    // Update 100 documents
    let start = std::time::Instant::now();
    for i in 0..100 {
        let query = format!(
            "UPDATE document SET content_hash = 'updated_content_{}' WHERE resource_hash = 'bench_hash_{}'",
            i, i
        );
        db.query(query).await?;
    }
    let update_duration = start.elapsed();
    println!("✓ UPDATE: 100 documents in {:?} ({:.2} ops/sec)",
        update_duration,
        100.0 / update_duration.as_secs_f64()
    );

    // Delete 100 documents
    let start = std::time::Instant::now();
    for i in 0..100 {
        let query = format!("DELETE FROM document WHERE resource_hash = 'bench_hash_{}'", i);
        db.query(query).await?;
    }
    let delete_duration = start.elapsed();
    println!("✓ DELETE: 100 documents in {:?} ({:.2} ops/sec)",
        delete_duration,
        100.0 / delete_duration.as_secs_f64()
    );

    // Verify all deleted
    let records: Vec<Document> = db.select("document").await?;
    assert_eq!(records.len(), 0, "All documents should be deleted");

    // Clean up
    drop(db);
    std::fs::remove_dir_all(&temp_dir)?;

    Ok(())
}
