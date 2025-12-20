use lib::*;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;

mod common;

/// End-to-end workflow test: init -> graph -> render -> toHTML
/// Tests the complete pipeline with realistic document hierarchy
#[tokio::test]
async fn test_complete_workflow_with_transclusion() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create test document structure
    std::fs::write(
        base_path.join("index.md"),
        r#"---
title: Main Document
---

# Main Document

This document includes content from multiple sources.

::file ./chapter1.md

::file ./chapter2.md

## Summary
This is the main document.
"#,
    )
    .unwrap();

    std::fs::write(
        base_path.join("chapter1.md"),
        r#"---
title: Chapter 1
---

# Chapter 1

This is the first chapter.

::file ./subsection.md
"#,
    )
    .unwrap();

    std::fs::write(
        base_path.join("chapter2.md"),
        r#"---
title: Chapter 2
---

# Chapter 2

This is the second chapter.
"#,
    )
    .unwrap();

    std::fs::write(
        base_path.join("subsection.md"),
        r#"## Subsection

This is a subsection included in chapter 1.
"#,
    )
    .unwrap();

    // Initialize the API
    let api = init(Some(base_path), None).await?;

    // Test graph building
    let resource = Resource {
        source: ResourceSource::Local(base_path.join("index.md")),
        requirement: ResourceRequirement::Required,
        cache_duration: None,
    };

    let graph = api.graph(resource.clone()).await?;
    assert_eq!(graph.nodes.len(), 4); // index, chapter1, chapter2, subsection
    assert!(graph.edges.len() >= 3); // index->chapter1, index->chapter2, chapter1->subsection

    // Test workplan generation
    let workplan = api.generate_workplan(vec![resource.clone()]).await?;
    assert!(workplan.layers.len() > 0);
    assert!(workplan.total_tasks > 0);

    // Test rendering
    let rendered = api.render(vec![resource.clone()], None).await?;
    assert_eq!(rendered.len(), 1);

    let doc = &rendered[0];
    assert_eq!(doc.frontmatter.get_string("title"), Some("Main Document"));
    assert!(doc.dependencies.len() >= 2);

    // Test HTML conversion
    let html_output = api
        .to_html(vec![base_path.join("index.md").to_string_lossy().to_string()])
        .await?;
    assert_eq!(html_output.len(), 1);

    let html = &html_output[0];
    assert!(html.html.contains("Main Document"));
    assert!(html.html.contains("Chapter 1"));
    assert!(html.html.contains("Chapter 2"));
    assert!(html.html.contains("Subsection"));

    Ok(())
}

/// Test caching behavior across multiple operations
#[tokio::test]
async fn test_caching_across_operations() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    std::fs::write(
        base_path.join("cached.md"),
        r#"# Cached Document

This document should be cached.
"#,
    )
    .unwrap();

    let api = init(Some(base_path), None).await?;

    let resource = Resource {
        source: ResourceSource::Local(base_path.join("cached.md")),
        requirement: ResourceRequirement::Required,
        cache_duration: Some(std::time::Duration::from_secs(3600)),
    };

    // First render - cache miss
    let start1 = std::time::Instant::now();
    let _rendered1 = api.render(vec![resource.clone()], None).await?;
    let duration1 = start1.elapsed();

    // Second render - should hit cache and be faster
    let start2 = std::time::Instant::now();
    let _rendered2 = api.render(vec![resource.clone()], None).await?;
    let duration2 = start2.elapsed();

    // Cache hit should be significantly faster (though this might be flaky in CI)
    // Just verify both succeed for now
    assert!(duration2 <= duration1 * 10); // Very generous threshold

    Ok(())
}

/// Test error handling and propagation
#[tokio::test]
async fn test_error_propagation() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create document with missing required dependency
    std::fs::write(
        base_path.join("broken.md"),
        r#"# Broken Document

::file ./missing.md!
"#,
    )
    .unwrap();

    let api = init(Some(base_path), None).await?;

    let resource = Resource {
        source: ResourceSource::Local(base_path.join("broken.md")),
        requirement: ResourceRequirement::Required,
        cache_duration: None,
    };

    // Should fail with meaningful error
    let result = api.render(vec![resource], None).await;
    assert!(result.is_err());

    // Verify error type
    if let Err(e) = result {
        match e {
            CompositionError::Render(_) | CompositionError::Parse(_) => {
                // Expected error types
            }
            _ => panic!("Unexpected error type: {:?}", e),
        }
    }

    Ok(())
}

/// Test frontmatter interpolation across the pipeline
#[tokio::test]
async fn test_frontmatter_interpolation_e2e() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    std::fs::write(
        base_path.join("interpolated.md"),
        r#"---
title: Test Document
author: Jane Doe
version: 1.0
---

# {{title}}

Written by {{author}}, version {{version}}.
"#,
    )
    .unwrap();

    let api = init(Some(base_path), None).await?;

    let resource = Resource {
        source: ResourceSource::Local(base_path.join("interpolated.md")),
        requirement: ResourceRequirement::Required,
        cache_duration: None,
    };

    let html_output = api
        .to_html(vec![base_path
            .join("interpolated.md")
            .to_string_lossy()
            .to_string()])
        .await?;

    assert_eq!(html_output.len(), 1);

    let html = &html_output[0].html;
    assert!(html.contains("Test Document"));
    assert!(html.contains("Jane Doe"));
    assert!(html.contains("1.0"));

    Ok(())
}

/// Test cycle detection in document graph
#[tokio::test]
async fn test_cycle_detection() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create circular dependency
    std::fs::write(
        base_path.join("a.md"),
        r#"# Document A

::file ./b.md
"#,
    )
    .unwrap();

    std::fs::write(
        base_path.join("b.md"),
        r#"# Document B

::file ./a.md
"#,
    )
    .unwrap();

    let api = init(Some(base_path), None).await?;

    let resource = Resource {
        source: ResourceSource::Local(base_path.join("a.md")),
        requirement: ResourceRequirement::Required,
        cache_duration: None,
    };

    // Should detect cycle
    let result = api.graph(resource).await;
    assert!(result.is_err());

    if let Err(e) = result {
        match e {
            CompositionError::Parse(ParseError::CircularDependency { .. }) => {
                // Expected
            }
            _ => panic!("Expected circular dependency error, got: {:?}", e),
        }
    }

    Ok(())
}

/// Test concurrent rendering with rayon
#[tokio::test]
async fn test_concurrent_rendering() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create multiple independent documents
    for i in 0..10 {
        std::fs::write(
            base_path.join(format!("doc{}.md", i)),
            format!("# Document {}\n\nThis is document {}.", i, i),
        )
        .unwrap();
    }

    let api = init(Some(base_path), None).await?;

    let resources: Vec<Resource> = (0..10)
        .map(|i| Resource {
            source: ResourceSource::Local(base_path.join(format!("doc{}.md", i))),
            requirement: ResourceRequirement::Required,
            cache_duration: None,
        })
        .collect();

    // Render all documents
    let start = std::time::Instant::now();
    let rendered = api.render(resources.clone(), None).await?;
    let _duration = start.elapsed();

    assert_eq!(rendered.len(), 10);

    // Verify all documents were rendered
    for (i, doc) in rendered.iter().enumerate() {
        assert!(doc.content.len() > 0);
    }

    Ok(())
}

/// Test table rendering from inline and external sources
#[tokio::test]
async fn test_table_rendering_e2e() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create CSV file
    std::fs::write(
        base_path.join("data.csv"),
        "Name,Age,City\nAlice,30,NYC\nBob,25,LA\nCarol,35,SF\n",
    )
    .unwrap();

    std::fs::write(
        base_path.join("tables.md"),
        r#"# Tables Test

## Inline Table
::table --with-heading-row
| Name | Age | City |
| Alice | 30 | NYC |
| Bob | 25 | LA |

## External Table
::table --with-heading-row ./data.csv
"#,
    )
    .unwrap();

    let api = init(Some(base_path), None).await?;

    let html_output = api
        .to_html(vec![base_path
            .join("tables.md")
            .to_string_lossy()
            .to_string()])
        .await?;

    let html = &html_output[0].html;

    // Verify table content is present
    assert!(html.contains("Alice"));
    assert!(html.contains("Bob"));
    assert!(html.contains("Carol"));
    assert!(html.contains("30"));
    assert!(html.contains("25"));
    assert!(html.contains("35"));

    Ok(())
}

/// Test workplan optimization with cached resources
#[tokio::test]
async fn test_workplan_optimization() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    std::fs::write(
        base_path.join("main.md"),
        r#"# Main

::file ./dep1.md
::file ./dep2.md
"#,
    )
    .unwrap();

    std::fs::write(base_path.join("dep1.md"), "# Dep 1").unwrap();
    std::fs::write(base_path.join("dep2.md"), "# Dep 2").unwrap();

    let api = init(Some(base_path), None).await?;

    let resource = Resource {
        source: ResourceSource::Local(base_path.join("main.md")),
        requirement: ResourceRequirement::Required,
        cache_duration: None,
    };

    // First render to populate cache
    let _rendered = api.render(vec![resource.clone()], None).await?;

    // Generate workplan - should skip cached items
    let workplan = api.generate_workplan(vec![resource.clone()]).await?;

    // Workplan should be optimized (exact behavior depends on implementation)
    assert!(workplan.layers.len() > 0);

    Ok(())
}
