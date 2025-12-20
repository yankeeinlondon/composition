use crate::error::RenderError;
use crate::types::{DarkMatterNode, MarkdownContent};
use pulldown_cmark::{html, Options, Parser};
use tracing::instrument;

use super::table::render_table;
use super::charts::{render_bar_chart, render_line_chart, render_pie_chart, render_area_chart, render_bubble_chart};
use super::popover::render_popover as render_popover_component;
use super::disclosure::render_disclosure as render_disclosure_component;
use super::columns::render_columns as render_columns_component;

/// Convert DarkMatter nodes to HTML
///
/// This function processes all node types and generates self-contained HTML output
#[instrument(skip(nodes))]
pub fn to_html(nodes: &[DarkMatterNode]) -> Result<String, RenderError> {
    let mut html = String::new();

    for node in nodes {
        let node_html = render_node(node)?;
        html.push_str(&node_html);
    }

    Ok(html)
}

/// Render a single DarkMatter node to HTML
fn render_node(node: &DarkMatterNode) -> Result<String, RenderError> {
    match node {
        DarkMatterNode::Markdown(content) => render_markdown(content),
        DarkMatterNode::Text(text) => Ok(escape_html(text)),
        DarkMatterNode::Table { source, has_heading } => render_table(source, *has_heading),
        DarkMatterNode::Popover { trigger, content } => render_popover(trigger, content),
        DarkMatterNode::Disclosure { summary, details } => render_disclosure(summary, details),
        DarkMatterNode::Columns { breakpoints, sections } => render_columns(breakpoints, sections),

        // AI operations would be resolved before HTML generation
        DarkMatterNode::Summarize { .. } |
        DarkMatterNode::Consolidate { .. } |
        DarkMatterNode::Topic { .. } => {
            Err(RenderError::HtmlGenerationFailed(
                "AI operations must be resolved before HTML generation".to_string()
            ))
        }

        // Transclusion should be resolved before HTML generation
        DarkMatterNode::File { .. } => {
            Err(RenderError::HtmlGenerationFailed(
                "File transclusions must be resolved before HTML generation".to_string()
            ))
        }

        // Charts
        DarkMatterNode::BarChart { data } => {
            render_bar_chart(data, 800, 400)
        }
        DarkMatterNode::LineChart { data } => {
            render_line_chart(data, 800, 400)
        }
        DarkMatterNode::PieChart { data } => {
            render_pie_chart(data, 400, 400)
        }
        DarkMatterNode::AreaChart { data } => {
            render_area_chart(data, 800, 400)
        }
        DarkMatterNode::BubbleChart { data } => {
            render_bubble_chart(data, 800, 400)
        }

        // Interpolation should be processed before HTML generation
        DarkMatterNode::Interpolation { variable } => {
            Ok(format!("{{{{{}}}}}", variable)) // Return as-is if not processed
        }
    }
}

/// Render markdown content to HTML using pulldown-cmark
fn render_markdown(content: &MarkdownContent) -> Result<String, RenderError> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(&content.raw, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Ok(html_output)
}

/// Render a popover to HTML
fn render_popover(trigger: &DarkMatterNode, content: &[DarkMatterNode]) -> Result<String, RenderError> {
    render_popover_component(trigger, content)
}

/// Render disclosure (details/summary) to HTML
fn render_disclosure(summary: &[DarkMatterNode], details: &[DarkMatterNode]) -> Result<String, RenderError> {
    render_disclosure_component(summary, details)
}

/// Render columns to HTML with responsive grid
fn render_columns(
    breakpoints: &std::collections::HashMap<crate::types::Breakpoint, u32>,
    sections: &[Vec<DarkMatterNode>],
) -> Result<String, RenderError> {
    render_columns_component(breakpoints, sections)
}

/// Escape HTML special characters
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Frontmatter;

    #[test]
    fn test_render_markdown_simple() {
        let content = MarkdownContent {
            raw: "# Hello World\n\nThis is a **test**.".to_string(),
            frontmatter: None,
        };

        let html = render_markdown(&content).unwrap();
        assert!(html.contains("<h1>Hello World</h1>"));
        assert!(html.contains("<strong>test</strong>"));
    }

    #[test]
    fn test_render_markdown_with_table() {
        let content = MarkdownContent {
            raw: "| A | B |\n|---|---|\n| 1 | 2 |".to_string(),
            frontmatter: None,
        };

        let html = render_markdown(&content).unwrap();
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>A</th>"));
        assert!(html.contains("<td>1</td>"));
    }

    #[test]
    fn test_render_text() {
        let node = DarkMatterNode::Text("Plain text".to_string());
        let html = render_node(&node).unwrap();
        assert_eq!(html, "Plain text");
    }

    #[test]
    fn test_render_text_escapes_html() {
        let node = DarkMatterNode::Text("<script>alert('xss')</script>".to_string());
        let html = render_node(&node).unwrap();
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_render_disclosure() {
        let summary = vec![DarkMatterNode::Text("Click me".to_string())];
        let details = vec![DarkMatterNode::Text("Hidden content".to_string())];

        let html = render_disclosure(&summary, &details).unwrap();
        assert!(html.contains("<details"));
        assert!(html.contains("composition-disclosure"));
        assert!(html.contains("Click me"));
        assert!(html.contains("Hidden content"));
    }

    #[test]
    fn test_to_html_multiple_nodes() {
        let nodes = vec![
            DarkMatterNode::Text("First".to_string()),
            DarkMatterNode::Text("Second".to_string()),
        ];

        let html = to_html(&nodes).unwrap();
        assert!(html.contains("First"));
        assert!(html.contains("Second"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("Hello"), "Hello");
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("A & B"), "A &amp; B");
        assert_eq!(escape_html("\"quote\""), "&quot;quote&quot;");
    }
}
