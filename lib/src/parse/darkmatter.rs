use crate::error::ParseError;
use crate::types::{DarkMatterNode, LineRange};
use crate::parse::resource::{parse_resource, parse_resources};
use regex::Regex;
use std::sync::LazyLock;

// Regex patterns for DarkMatter directives
static FILE_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^::file\s+(.+?)(?:\s+(\d+)-(\d+)?)?$").unwrap()
});

static SUMMARIZE_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^::summarize\s+(.+)$").unwrap()
});

static CONSOLIDATE_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^::consolidate\s+(.+)$").unwrap()
});

static TOPIC_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^::topic\s+"([^"]+)"\s+(.+?)(?:\s+--review)?$"#).unwrap()
});

static TABLE_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^::table(?:\s+(.+?))?(?:\s+--with-heading-row)?$").unwrap()
});

static CHART_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^::(bar-chart|line-chart|pie-chart|area-chart|bubble-chart)\s+(.+)$").unwrap()
});

static COLUMNS_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^::columns(?:\s+(.+))?$").unwrap()
});

static POPOVER_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[([^\]]+)\]\(popover:([^)]+)\)").unwrap()
});

static INTERPOLATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{\{(\w+)\}\}").unwrap()
});

/// Parse a DarkMatter block directive
pub fn parse_directive(line: &str, line_num: usize) -> Result<Option<DarkMatterNode>, ParseError> {
    let trimmed = line.trim();

    // Check for various directive types
    if let Some(caps) = FILE_DIRECTIVE.captures(trimmed) {
        let resource = parse_resource(caps.get(1).unwrap().as_str())?;

        let range = if let (Some(start), Some(end)) = (caps.get(2), caps.get(3)) {
            let start_num = start.as_str().parse::<usize>()
                .map_err(|_| ParseError::InvalidDirective {
                    line: line_num,
                    directive: line.to_string(),
                })?;
            let end_num = end.as_str().parse::<usize>()
                .map_err(|_| ParseError::InvalidDirective {
                    line: line_num,
                    directive: line.to_string(),
                })?;

            Some(LineRange {
                start: start_num,
                end: Some(end_num),
            })
        } else if let Some(start) = caps.get(2) {
            let start_num = start.as_str().parse::<usize>()
                .map_err(|_| ParseError::InvalidDirective {
                    line: line_num,
                    directive: line.to_string(),
                })?;

            Some(LineRange {
                start: start_num,
                end: None,
            })
        } else {
            None
        };

        return Ok(Some(DarkMatterNode::File { resource, range }));
    }

    if let Some(caps) = SUMMARIZE_DIRECTIVE.captures(trimmed) {
        let resource = parse_resource(caps.get(1).unwrap().as_str())?;
        return Ok(Some(DarkMatterNode::Summarize { resource }));
    }

    if let Some(caps) = CONSOLIDATE_DIRECTIVE.captures(trimmed) {
        let resources = parse_resources(caps.get(1).unwrap().as_str())?;
        return Ok(Some(DarkMatterNode::Consolidate { resources }));
    }

    if let Some(caps) = TOPIC_DIRECTIVE.captures(trimmed) {
        let topic = caps.get(1).unwrap().as_str().to_string();
        let resources = parse_resources(caps.get(2).unwrap().as_str())?;
        let review = trimmed.contains("--review");

        return Ok(Some(DarkMatterNode::Topic {
            topic,
            resources,
            review,
        }));
    }

    if let Some(caps) = TABLE_DIRECTIVE.captures(trimmed) {
        let has_heading = trimmed.contains("--with-heading-row");

        let source = if let Some(path_match) = caps.get(1) {
            let resource = parse_resource(path_match.as_str())?;
            crate::types::TableSource::External(resource)
        } else {
            // Inline table - will be populated later when parsing table content
            crate::types::TableSource::Inline(Vec::new())
        };

        return Ok(Some(DarkMatterNode::Table {
            source,
            has_heading,
        }));
    }

    if let Some(caps) = CHART_DIRECTIVE.captures(trimmed) {
        let chart_type = caps.get(1).unwrap().as_str();
        let resource = parse_resource(caps.get(2).unwrap().as_str())?;
        let data = crate::types::ChartData::External(resource);

        return Ok(Some(match chart_type {
            "bar-chart" => DarkMatterNode::BarChart { data },
            "line-chart" => DarkMatterNode::LineChart { data },
            "pie-chart" => DarkMatterNode::PieChart { data },
            "area-chart" => DarkMatterNode::AreaChart { data },
            "bubble-chart" => DarkMatterNode::BubbleChart { data },
            _ => return Err(ParseError::InvalidDirective {
                line: line_num,
                directive: line.to_string(),
            }),
        }));
    }

    // Check for summary/details directives
    if trimmed == "::summary" {
        // This will be handled by the parser context
        return Ok(None);
    }

    if trimmed == "::details" {
        // This will be handled by the parser context
        return Ok(None);
    }

    if trimmed == "::break" {
        // Column break - handled by parser context
        return Ok(None);
    }

    if COLUMNS_DIRECTIVE.is_match(trimmed) {
        // Columns - will be handled by parser context
        return Ok(None);
    }

    // Not a recognized directive
    Ok(None)
}

/// Process inline DarkMatter syntax in text
pub fn process_inline_syntax(text: &str) -> Vec<DarkMatterNode> {
    let mut nodes = Vec::new();
    let mut current_pos = 0;

    // TODO: Handle popover links in future phase
    // For now, we'll just handle interpolations

    // Find all popover links (placeholder for future implementation)
    for caps in POPOVER_LINK.captures_iter(text) {
        let _full_match = caps.get(0).unwrap();
        let _trigger_text = caps.get(1).unwrap().as_str();
        let _popover_content = caps.get(2).unwrap().as_str();

        // For now, we'll create a simplified representation
        // In a full implementation, this would create proper popover nodes
    }

    // Find all interpolations
    for caps in INTERPOLATION.captures_iter(text) {
        let full_match = caps.get(0).unwrap();
        let var_name = caps.get(1).unwrap().as_str();

        let match_start = full_match.start();

        // Add text before interpolation
        if match_start > current_pos {
            let text_before = &text[current_pos..match_start];
            if !text_before.is_empty() {
                nodes.push(DarkMatterNode::Text(text_before.to_string()));
            }
        }

        // Add interpolation node
        nodes.push(DarkMatterNode::Interpolation {
            variable: var_name.to_string(),
        });

        current_pos = full_match.end();
    }

    // Add remaining text
    if current_pos < text.len() {
        let remaining = &text[current_pos..];
        if !remaining.is_empty() {
            nodes.push(DarkMatterNode::Text(remaining.to_string()));
        }
    }

    // If no inline syntax was found, return the original text as a single node
    if nodes.is_empty() {
        nodes.push(DarkMatterNode::Text(text.to_string()));
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_directive() {
        let node = parse_directive("::file ./path/to/file.md", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::File { resource, range } => {
                assert!(matches!(resource.source, crate::types::ResourceSource::Local(_)));
                assert!(range.is_none());
            }
            _ => panic!("Expected File node"),
        }
    }

    #[test]
    fn test_parse_file_directive_with_range() {
        let node = parse_directive("::file ./file.md 10-20", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::File { resource: _, range } => {
                let range = range.unwrap();
                assert_eq!(range.start, 10);
                assert_eq!(range.end, Some(20));
            }
            _ => panic!("Expected File node"),
        }
    }

    #[test]
    fn test_parse_summarize_directive() {
        let node = parse_directive("::summarize ./doc.md", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Summarize { resource: _ } => {
                // Success
            }
            _ => panic!("Expected Summarize node"),
        }
    }

    #[test]
    fn test_parse_consolidate_directive() {
        let node = parse_directive("::consolidate ./a.md ./b.md", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Consolidate { resources } => {
                assert_eq!(resources.len(), 2);
            }
            _ => panic!("Expected Consolidate node"),
        }
    }

    #[test]
    fn test_parse_topic_directive() {
        let node = parse_directive(r#"::topic "testing" ./a.md ./b.md"#, 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Topic { topic, resources, review } => {
                assert_eq!(topic, "testing");
                assert_eq!(resources.len(), 2);
                assert!(!review);
            }
            _ => panic!("Expected Topic node"),
        }
    }

    #[test]
    fn test_parse_topic_directive_with_review() {
        let node = parse_directive(r#"::topic "testing" ./a.md --review"#, 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Topic { topic: _, resources: _, review } => {
                assert!(review);
            }
            _ => panic!("Expected Topic node"),
        }
    }

    #[test]
    fn test_parse_table_directive() {
        let node = parse_directive("::table ./data.csv --with-heading-row", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Table { source, has_heading } => {
                assert!(matches!(source, crate::types::TableSource::External(_)));
                assert!(has_heading);
            }
            _ => panic!("Expected Table node"),
        }
    }

    #[test]
    fn test_parse_chart_directive() {
        let node = parse_directive("::bar-chart ./data.csv", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::BarChart { data: _ } => {
                // Success
            }
            _ => panic!("Expected BarChart node"),
        }
    }

    #[test]
    fn test_process_interpolation() {
        let nodes = process_inline_syntax("Hello {{name}}, welcome!");

        assert_eq!(nodes.len(), 3);
        assert!(matches!(nodes[0], DarkMatterNode::Text(_)));
        assert!(matches!(nodes[1], DarkMatterNode::Interpolation { .. }));
        assert!(matches!(nodes[2], DarkMatterNode::Text(_)));
    }

    #[test]
    fn test_process_plain_text() {
        let nodes = process_inline_syntax("Just plain text");

        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            DarkMatterNode::Text(t) => assert_eq!(t, "Just plain text"),
            _ => panic!("Expected Text node"),
        }
    }
}
