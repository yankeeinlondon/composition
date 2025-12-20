use crate::error::RenderError;
use crate::types::{DarkMatterNode, MarkdownContent};
use pulldown_cmark::{html, Options, Parser};
use tracing::instrument;

use super::table::render_table;
use super::charts::{render_bar_chart, render_line_chart, render_pie_chart, render_area_chart, render_bubble_chart};
use super::popover::render_popover as render_popover_component;
use super::disclosure::render_disclosure as render_disclosure_component;
use super::columns::render_columns as render_columns_component;
use super::youtube::render_youtube_embed;

/// Convert DarkMatter nodes to HTML
///
/// This function processes all node types and generates self-contained HTML output
#[instrument(skip(nodes))]
pub fn to_html(nodes: &[DarkMatterNode]) -> Result<String, RenderError> {
    let mut html = String::new();
    let mut youtube_assets_included = false;

    for node in nodes {
        let node_html = render_node(node)?;
        html.push_str(&node_html);

        // Include YouTube assets on first occurrence
        if matches!(node, DarkMatterNode::YouTube { .. }) && !youtube_assets_included {
            html.push_str(&format!(
                "\n<style id=\"dm-youtube\">{}</style>",
                super::youtube::youtube_css()
            ));
            html.push_str(&format!(
                "\n<script id=\"dm-youtube\">{}</script>",
                super::youtube::youtube_js()
            ));
            youtube_assets_included = true;
        }
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

        // Audio should be processed before HTML generation
        DarkMatterNode::Audio { .. } => {
            Err(RenderError::HtmlGenerationFailed(
                "Audio directives must be processed before HTML generation".to_string()
            ))
        }

        // YouTube rendering
        DarkMatterNode::YouTube { video_id, width } => {
            Ok(render_youtube_embed(video_id, width))
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

    // YouTube asset deduplication tests
    #[test]
    fn test_youtube_single_embed_includes_assets() {
        use crate::types::WidthSpec;

        let nodes = vec![
            DarkMatterNode::YouTube {
                video_id: "dQw4w9WgXcQ".to_string(),
                width: WidthSpec::Pixels(512),
            },
        ];

        let html = to_html(&nodes).unwrap();

        // Verify embed HTML is present
        assert!(html.contains("dm-youtube-container"));
        assert!(html.contains("dQw4w9WgXcQ"));

        // Verify CSS is included exactly once
        assert!(html.contains(r#"<style id="dm-youtube">"#));
        let css_count = html.matches(r#"<style id="dm-youtube">"#).count();
        assert_eq!(css_count, 1, "CSS should be included exactly once");

        // Verify JS is included exactly once
        assert!(html.contains(r#"<script id="dm-youtube">"#));
        let js_count = html.matches(r#"<script id="dm-youtube">"#).count();
        assert_eq!(js_count, 1, "JS should be included exactly once");
    }

    #[test]
    fn test_youtube_multiple_embeds_assets_once() {
        use crate::types::WidthSpec;

        let nodes = vec![
            DarkMatterNode::YouTube {
                video_id: "dQw4w9WgXcQ".to_string(),
                width: WidthSpec::Pixels(512),
            },
            DarkMatterNode::YouTube {
                video_id: "jNQXAC9IVRw".to_string(),
                width: WidthSpec::Pixels(800),
            },
            DarkMatterNode::YouTube {
                video_id: "9bZkp7q19f0".to_string(),
                width: WidthSpec::Rems(32.0),
            },
        ];

        let html = to_html(&nodes).unwrap();

        // Verify all embeds are present
        assert!(html.contains("dQw4w9WgXcQ"));
        assert!(html.contains("jNQXAC9IVRw"));
        assert!(html.contains("9bZkp7q19f0"));

        // Verify each embed has its container (count opening div tags with the class)
        let container_count = html.matches(r#"<div class="dm-youtube-container""#).count();
        assert_eq!(container_count, 3, "Should have 3 embed containers");

        // Verify CSS is included exactly once
        let css_count = html.matches(r#"<style id="dm-youtube">"#).count();
        assert_eq!(css_count, 1, "CSS should be included exactly once despite multiple embeds");

        // Verify JS is included exactly once
        let js_count = html.matches(r#"<script id="dm-youtube">"#).count();
        assert_eq!(js_count, 1, "JS should be included exactly once despite multiple embeds");
    }

    #[test]
    fn test_youtube_mixed_with_other_nodes() {
        use crate::types::WidthSpec;

        let nodes = vec![
            DarkMatterNode::Text("Introduction text".to_string()),
            DarkMatterNode::YouTube {
                video_id: "dQw4w9WgXcQ".to_string(),
                width: WidthSpec::Pixels(512),
            },
            DarkMatterNode::Text("Middle text".to_string()),
            DarkMatterNode::YouTube {
                video_id: "jNQXAC9IVRw".to_string(),
                width: WidthSpec::Pixels(800),
            },
            DarkMatterNode::Text("Conclusion text".to_string()),
        ];

        let html = to_html(&nodes).unwrap();

        // Verify all content is present
        assert!(html.contains("Introduction text"));
        assert!(html.contains("Middle text"));
        assert!(html.contains("Conclusion text"));
        assert!(html.contains("dQw4w9WgXcQ"));
        assert!(html.contains("jNQXAC9IVRw"));

        // Verify assets included only once
        let css_count = html.matches(r#"<style id="dm-youtube">"#).count();
        assert_eq!(css_count, 1);
        let js_count = html.matches(r#"<script id="dm-youtube">"#).count();
        assert_eq!(js_count, 1);
    }

    #[test]
    fn test_youtube_no_embeds_no_assets() {
        let nodes = vec![
            DarkMatterNode::Text("Just text".to_string()),
            DarkMatterNode::Text("More text".to_string()),
        ];

        let html = to_html(&nodes).unwrap();

        // Verify no YouTube assets are included
        assert!(!html.contains(r#"<style id="dm-youtube">"#));
        assert!(!html.contains(r#"<script id="dm-youtube">"#));
        assert!(!html.contains("dm-youtube-container"));
    }

    #[test]
    fn test_youtube_assets_order() {
        use crate::types::WidthSpec;

        let nodes = vec![
            DarkMatterNode::YouTube {
                video_id: "dQw4w9WgXcQ".to_string(),
                width: WidthSpec::Pixels(512),
            },
        ];

        let html = to_html(&nodes).unwrap();

        // Find positions of embed, CSS, and JS
        let embed_pos = html.find("dm-youtube-container").unwrap();
        let css_pos = html.find(r#"<style id="dm-youtube">"#).unwrap();
        let js_pos = html.find(r#"<script id="dm-youtube">"#).unwrap();

        // Assets should come after the first embed
        assert!(css_pos > embed_pos, "CSS should come after the first embed");
        assert!(js_pos > embed_pos, "JS should come after the first embed");

        // CSS should come before JS
        assert!(css_pos < js_pos, "CSS should come before JS");
    }

    #[test]
    fn test_youtube_different_widths_single_assets() {
        use crate::types::WidthSpec;

        let nodes = vec![
            DarkMatterNode::YouTube {
                video_id: "video1".to_string(),
                width: WidthSpec::Pixels(512),
            },
            DarkMatterNode::YouTube {
                video_id: "video2".to_string(),
                width: WidthSpec::Rems(32.0),
            },
            DarkMatterNode::YouTube {
                video_id: "video3".to_string(),
                width: WidthSpec::Percentage(80),
            },
        ];

        let html = to_html(&nodes).unwrap();

        // Verify all embeds present with different widths
        assert!(html.contains("video1"));
        assert!(html.contains("video2"));
        assert!(html.contains("video3"));

        // Verify assets only included once despite different width configurations
        let css_count = html.matches(r#"<style id="dm-youtube">"#).count();
        assert_eq!(css_count, 1);
        let js_count = html.matches(r#"<script id="dm-youtube">"#).count();
        assert_eq!(js_count, 1);
    }
}
