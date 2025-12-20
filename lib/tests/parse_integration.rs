use lib::parse::parse_document;
use lib::types::{DarkMatterNode, Resource};
use std::path::PathBuf;

#[test]
fn test_full_document_parsing() {
    let content = r#"---
title: Integration Test Document
author: Test Suite
list_expansion: expanded
summarize_model: gpt-4
---
# Introduction

Welcome to {{title}}!

## File Inclusion

::file ./external-content.md

## AI Features

::summarize ./long-document.md

::consolidate ./doc1.md ./doc2.md

::topic "testing" ./test-docs/*.md --review

## Data Visualization

::table ./data.csv --with-heading-row

::bar-chart ./sales.csv

## Layout

Some regular markdown content with **bold** and *italic* text.
"#;

    let resource = Resource::local(PathBuf::from("test.md"));
    let doc = parse_document(content, resource).unwrap();

    // Verify frontmatter
    assert_eq!(doc.frontmatter.get_string("title"), Some("Integration Test Document"));
    assert_eq!(doc.frontmatter.get_string("author"), Some("Test Suite"));
    assert_eq!(doc.frontmatter.summarize_model, Some("gpt-4".to_string()));

    // Verify content nodes
    assert!(!doc.content.is_empty());

    // Verify interpolation was recognized
    let has_interpolation = doc.content.iter().any(|n| matches!(n, DarkMatterNode::Interpolation { .. }));
    assert!(has_interpolation, "Should have found interpolation node");

    // Verify directives were recognized
    let has_file = doc.content.iter().any(|n| matches!(n, DarkMatterNode::File { .. }));
    assert!(has_file, "Should have found file directive");

    let has_summarize = doc.content.iter().any(|n| matches!(n, DarkMatterNode::Summarize { .. }));
    assert!(has_summarize, "Should have found summarize directive");

    let has_consolidate = doc.content.iter().any(|n| matches!(n, DarkMatterNode::Consolidate { .. }));
    assert!(has_consolidate, "Should have found consolidate directive");

    let has_topic = doc.content.iter().any(|n| matches!(n, DarkMatterNode::Topic { .. }));
    assert!(has_topic, "Should have found topic directive");

    let has_table = doc.content.iter().any(|n| matches!(n, DarkMatterNode::Table { .. }));
    assert!(has_table, "Should have found table directive");

    let has_chart = doc.content.iter().any(|n| matches!(n, DarkMatterNode::BarChart { .. }));
    assert!(has_chart, "Should have found bar chart directive");

    // Verify dependencies were collected
    assert!(!doc.dependencies.is_empty(), "Should have collected dependencies");

    // Count expected dependencies:
    // 1x file, 1x summarize, 2x consolidate, 1x topic, 1x table, 1x chart = 7 total
    assert!(doc.dependencies.len() >= 6, "Should have at least 6 dependencies, found {}", doc.dependencies.len());
}

#[test]
fn test_nested_interpolations() {
    let content = r#"---
name: John
greeting: Hello
---
{{greeting}} {{name}}, welcome to our {{name}}'s page!"#;

    let resource = Resource::local(PathBuf::from("test.md"));
    let doc = parse_document(content, resource).unwrap();

    // Count interpolations
    let interpolation_count = doc.content.iter()
        .filter(|n| matches!(n, DarkMatterNode::Interpolation { .. }))
        .count();

    assert_eq!(interpolation_count, 3, "Should have found 3 interpolations");
}

#[test]
fn test_resource_requirements() {
    let content = r#"
::file ./required.md!

::file ./optional.md?

::file ./default.md
"#;

    let resource = Resource::local(PathBuf::from("test.md"));
    let doc = parse_document(content, resource).unwrap();

    assert_eq!(doc.dependencies.len(), 3);

    // Check requirements
    use lib::types::ResourceRequirement;

    let required_count = doc.dependencies.iter()
        .filter(|r| matches!(r.requirement, ResourceRequirement::Required))
        .count();
    assert_eq!(required_count, 1);

    let optional_count = doc.dependencies.iter()
        .filter(|r| matches!(r.requirement, ResourceRequirement::Optional))
        .count();
    assert_eq!(optional_count, 1);

    let default_count = doc.dependencies.iter()
        .filter(|r| matches!(r.requirement, ResourceRequirement::Default))
        .count();
    assert_eq!(default_count, 1);
}

#[test]
fn test_remote_resources() {
    let content = r#"
::file https://example.com/remote.md

::summarize https://api.example.com/data.json!
"#;

    let resource = Resource::local(PathBuf::from("test.md"));
    let doc = parse_document(content, resource).unwrap();

    assert_eq!(doc.dependencies.len(), 2);

    use lib::types::ResourceSource;

    let remote_count = doc.dependencies.iter()
        .filter(|r| matches!(r.source, ResourceSource::Remote(_)))
        .count();

    assert_eq!(remote_count, 2, "Both resources should be remote");

    // All remote resources should have cache duration set
    for dep in &doc.dependencies {
        if matches!(dep.source, ResourceSource::Remote(_)) {
            assert!(dep.cache_duration.is_some(), "Remote resources should have cache duration");
        }
    }
}

#[test]
fn test_line_range_parsing() {
    let content = "::file ./code.rs 10-50";

    let resource = Resource::local(PathBuf::from("test.md"));
    let doc = parse_document(content, resource).unwrap();

    let file_node = doc.content.iter()
        .find_map(|n| match n {
            DarkMatterNode::File { range, .. } => Some(range),
            _ => None,
        })
        .expect("Should have found file node");

    let range = file_node.as_ref().expect("Should have range");
    assert_eq!(range.start, 10);
    assert_eq!(range.end, Some(50));
}

#[test]
fn test_gfm_extensions() {
    let content = r#"
# Strikethrough Test

This has ~~strikethrough~~ text.

# Table Test

| Name | Value |
|------|-------|
| A    | 1     |
| B    | 2     |

# Task List Test

- [x] Completed task
- [ ] Pending task
"#;

    let resource = Resource::local(PathBuf::from("test.md"));
    let result = parse_document(content, resource);

    assert!(result.is_ok(), "GFM extensions should parse without error");
}

#[test]
fn test_complex_frontmatter() {
    let content = r#"---
title: Complex Document
tags:
  - rust
  - markdown
  - testing
metadata:
  version: 1.0
  status: draft
list_expansion: collapsed
replace:
  LLM: Large Language Model
  AI: Artificial Intelligence
breakpoints:
  sm: 640
  md: 768
  lg: 1024
---
Content here"#;

    let resource = Resource::local(PathBuf::from("test.md"));
    let doc = parse_document(content, resource).unwrap();

    assert_eq!(doc.frontmatter.get_string("title"), Some("Complex Document"));

    // Check replace map
    let replace = doc.frontmatter.replace.as_ref().expect("Should have replace map");
    assert_eq!(replace.get("LLM"), Some(&"Large Language Model".to_string()));
    assert_eq!(replace.get("AI"), Some(&"Artificial Intelligence".to_string()));

    // Check breakpoints
    let breakpoints = doc.frontmatter.breakpoints.as_ref().expect("Should have breakpoints");
    assert_eq!(breakpoints.sm, Some(640));
    assert_eq!(breakpoints.md, Some(768));
    assert_eq!(breakpoints.lg, Some(1024));
}
