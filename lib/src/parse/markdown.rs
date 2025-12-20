use crate::error::ParseError;
use crate::types::{DarkMatterNode, MarkdownContent};
use crate::parse::darkmatter::{parse_directive, process_inline_syntax};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// Parse markdown content with GFM extensions
pub fn parse_markdown(content: &str) -> Result<Vec<DarkMatterNode>, ParseError> {
    // Split content into lines and process directives separately
    let mut nodes = Vec::new();
    let mut markdown_buffer = String::new();
    let mut line_num = 1;

    for line in content.lines() {
        let trimmed = line.trim();

        // Check if this is a DarkMatter directive
        if trimmed.starts_with("::") {
            // Flush any accumulated markdown first
            if !markdown_buffer.is_empty() {
                nodes.push(DarkMatterNode::Markdown(MarkdownContent {
                    raw: markdown_buffer.clone(),
                    frontmatter: None,
                }));
                markdown_buffer.clear();
            }

            // Parse the directive
            if let Some(node) = parse_directive(trimmed, line_num)? {
                nodes.push(node);
            }
        } else {
            // Accumulate markdown content
            if !markdown_buffer.is_empty() {
                markdown_buffer.push('\n');
            }
            markdown_buffer.push_str(line);
        }

        line_num += 1;
    }

    // Flush any remaining markdown
    if !markdown_buffer.is_empty() {
        nodes.push(DarkMatterNode::Markdown(MarkdownContent {
            raw: markdown_buffer,
            frontmatter: None,
        }));
    }

    Ok(nodes)
}

/// Parse markdown content with GFM extensions (old detailed parser - keeping for reference)
#[allow(dead_code)]
fn parse_markdown_detailed(content: &str) -> Result<Vec<DarkMatterNode>, ParseError> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(content, options);
    let mut nodes = Vec::new();
    let mut current_text = String::new();
    let mut line_num = 1;
    let mut in_paragraph = false;

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {
                in_paragraph = true;
                current_text.clear();
            }

            Event::End(TagEnd::Paragraph) => {
                in_paragraph = false;

                // Check if this paragraph is a DarkMatter directive
                let trimmed = current_text.trim();
                if trimmed.starts_with("::") {
                    if let Some(node) = parse_directive(trimmed, line_num)? {
                        nodes.push(node);
                    }
                } else if !current_text.is_empty() {
                    // Keep as markdown content
                    nodes.push(DarkMatterNode::Markdown(MarkdownContent {
                        raw: current_text.clone(),
                        frontmatter: None,
                    }));
                }

                current_text.clear();
            }

            Event::Text(text) => {
                if in_paragraph {
                    current_text.push_str(&text);
                } else {
                    // Text outside paragraph - process inline syntax
                    let inline_nodes = process_inline_syntax(&text);
                    nodes.extend(inline_nodes);
                }

                // Count newlines for line tracking
                line_num += text.matches('\n').count();
            }

            Event::Code(code) => {
                if in_paragraph {
                    current_text.push('`');
                    current_text.push_str(&code);
                    current_text.push('`');
                } else {
                    nodes.push(DarkMatterNode::Text(format!("`{}`", code)));
                }
            }

            Event::Start(Tag::CodeBlock(kind)) => {
                // Check if this is a special DarkMatter code block (e.g., ```table)
                let lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                    pulldown_cmark::CodeBlockKind::Indented => String::new(),
                };

                if lang.starts_with("table") {
                    // This is a table code block - we'll handle it when we get the text
                    current_text.clear();
                    current_text.push_str(&format!("::table {}\n", lang.strip_prefix("table").unwrap_or("")));
                } else {
                    // Regular code block - we'll wrap it as markdown
                    current_text.clear();
                    current_text.push_str(&format!("```{}\n", lang));
                }
            }

            Event::End(TagEnd::CodeBlock) => {
                if !current_text.is_empty() {
                    // If it was a table directive, parse it
                    if current_text.starts_with("::table") {
                        let first_line = current_text.lines().next().unwrap_or("");
                        if let Some(node) = parse_directive(first_line, line_num)? {
                            nodes.push(node);
                        }
                    } else {
                        // Regular code block - store as markdown
                        current_text.push_str("\n```");
                        nodes.push(DarkMatterNode::Markdown(MarkdownContent {
                            raw: current_text.clone(),
                            frontmatter: None,
                        }));
                    }

                    current_text.clear();
                }
            }

            Event::Start(Tag::Heading { .. }) => {
                current_text.clear();
            }

            Event::End(TagEnd::Heading(_)) => {
                if !current_text.is_empty() {
                    // Process inline syntax in heading text (for interpolations, etc.)
                    let inline_nodes = process_inline_syntax(&current_text);
                    nodes.extend(inline_nodes);
                    current_text.clear();
                }
            }

            Event::SoftBreak => {
                current_text.push('\n');
                line_num += 1;
            }

            Event::HardBreak => {
                current_text.push_str("  \n");
                line_num += 1;
            }

            // For other events, we'll add basic handling
            Event::Start(_) | Event::End(_) => {
                // Structure events - we'll handle more of these in future phases
            }

            Event::Html(html) => {
                nodes.push(DarkMatterNode::Markdown(MarkdownContent {
                    raw: html.to_string(),
                    frontmatter: None,
                }));
            }

            Event::InlineHtml(html) => {
                if in_paragraph {
                    current_text.push_str(&html);
                } else {
                    nodes.push(DarkMatterNode::Text(html.to_string()));
                }
            }

            _ => {
                // Other events not yet handled
            }
        }
    }

    // Handle any remaining text
    if !current_text.is_empty() {
        nodes.push(DarkMatterNode::Text(current_text));
    }

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_markdown() {
        let content = "# Hello\n\nThis is a paragraph.";
        let nodes = parse_markdown(content).unwrap();

        assert!(!nodes.is_empty());
    }

    #[test]
    fn test_parse_directive_in_markdown() {
        let content = "# Document\n\n::file ./some-file.md\n\nMore content.";
        let nodes = parse_markdown(content).unwrap();

        // Should find the directive
        let has_file_directive = nodes.iter().any(|n| matches!(n, DarkMatterNode::File { .. }));
        assert!(has_file_directive);
    }

    #[test]
    fn test_parse_interpolation_in_markdown() {
        let content = "Hello {{name}}, welcome!";
        let nodes = parse_markdown(content).unwrap();

        // Should find interpolation node
        let has_interpolation = nodes.iter().any(|n| matches!(n, DarkMatterNode::Interpolation { .. }));
        assert!(has_interpolation);
    }

    #[test]
    fn test_parse_gfm_table() {
        let content = r#"
| Name | Age |
|------|-----|
| John | 30  |
| Jane | 25  |
"#;

        // Should parse without error
        let result = parse_markdown(content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_code_block() {
        let content = "```rust\nfn main() {}\n```";
        let nodes = parse_markdown(content).unwrap();

        assert!(!nodes.is_empty());
    }
}
