mod frontmatter;
mod resource;
pub mod darkmatter;
mod markdown;

pub use frontmatter::extract_frontmatter;
pub use resource::{parse_resource, parse_resources};
pub use darkmatter::{parse_directive, process_inline_syntax};
pub use markdown::parse_markdown;

use crate::error::ParseError;
use crate::types::{Document, Resource, DarkMatterNode};
use chrono::Utc;

/// Parse a DarkMatter document from source content
///
/// This is the main entry point for parsing. It:
/// 1. Extracts frontmatter
/// 2. Parses markdown with DarkMatter DSL extensions
/// 3. Collects resource dependencies
/// 4. Returns a complete Document
pub fn parse_document(content: &str, source: Resource) -> Result<Document, ParseError> {
    // 1. Extract frontmatter
    let (frontmatter, body) = extract_frontmatter(content)?;

    // 2. Parse markdown and DarkMatter
    let nodes = parse_markdown(body)?;

    // 3. Collect dependencies from nodes
    let dependencies = collect_dependencies(&nodes);

    Ok(Document {
        resource: source,
        frontmatter,
        content: nodes,
        dependencies,
        parsed_at: Utc::now(),
    })
}

/// Collect all resource dependencies from parsed nodes
fn collect_dependencies(nodes: &[DarkMatterNode]) -> Vec<Resource> {
    let mut deps = Vec::new();

    for node in nodes {
        match node {
            DarkMatterNode::File { resource, .. } => {
                deps.push(resource.clone());
            }
            DarkMatterNode::Summarize { resource } => {
                deps.push(resource.clone());
            }
            DarkMatterNode::Consolidate { resources } => {
                deps.extend(resources.clone());
            }
            DarkMatterNode::Topic { resources, .. } => {
                deps.extend(resources.clone());
            }
            DarkMatterNode::Table { source: crate::types::TableSource::External(resource), .. } => {
                deps.push(resource.clone());
            }
            DarkMatterNode::Table { .. } => {
                // Inline table, no external dependencies
            }
            DarkMatterNode::BarChart { data } |
            DarkMatterNode::LineChart { data } |
            DarkMatterNode::PieChart { data } |
            DarkMatterNode::AreaChart { data } |
            DarkMatterNode::BubbleChart { data } => {
                if let crate::types::ChartData::External(resource) = data {
                    deps.push(resource.clone());
                }
            }
            DarkMatterNode::Popover { content, .. } => {
                // Recursively collect from popover content
                deps.extend(collect_dependencies(content));
            }
            DarkMatterNode::Columns { sections, .. } => {
                // Recursively collect from all sections
                for section in sections {
                    deps.extend(collect_dependencies(section));
                }
            }
            DarkMatterNode::Disclosure { summary, details } => {
                // Recursively collect from summary and details
                deps.extend(collect_dependencies(summary));
                deps.extend(collect_dependencies(details));
            }
            _ => {
                // Other node types don't have dependencies
            }
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_document_simple() {
        let content = "# Hello World\n\nThis is a test.";
        let resource = Resource::local(PathBuf::from("test.md"));

        let doc = parse_document(content, resource).unwrap();

        assert!(!doc.content.is_empty());
        assert!(doc.dependencies.is_empty());
    }

    #[test]
    fn test_parse_document_with_frontmatter() {
        let content = r#"---
title: Test Document
author: John Doe
---
# Hello

Content here"#;

        let resource = Resource::local(PathBuf::from("test.md"));
        let doc = parse_document(content, resource).unwrap();

        assert_eq!(doc.frontmatter.get_string("title"), Some("Test Document"));
        assert_eq!(doc.frontmatter.get_string("author"), Some("John Doe"));
    }

    #[test]
    fn test_parse_document_with_dependencies() {
        let content = "# Document\n\n::file ./other.md\n\n::summarize ./data.md";
        let resource = Resource::local(PathBuf::from("test.md"));

        let doc = parse_document(content, resource).unwrap();

        assert_eq!(doc.dependencies.len(), 2);
    }

    #[test]
    fn test_collect_dependencies() {
        let nodes = vec![
            DarkMatterNode::File {
                resource: Resource::local(PathBuf::from("a.md")),
                range: None,
            },
            DarkMatterNode::Summarize {
                resource: Resource::local(PathBuf::from("b.md")),
            },
        ];

        let deps = collect_dependencies(&nodes);
        assert_eq!(deps.len(), 2);
    }
}

