use crate::types::DarkMatterNode;
use crate::error::RenderError;

/// Render a disclosure block (details/summary) to HTML
pub fn render_disclosure(summary: &[DarkMatterNode], details: &[DarkMatterNode]) -> Result<String, RenderError> {
    let summary_html = render_nodes_to_html(summary)?;
    let details_html = render_nodes_to_html(details)?;

    let html = format!(
        r#"<details class="composition-disclosure">
  <summary class="composition-disclosure-summary">
    {}
  </summary>
  <div class="composition-disclosure-content">
    {}
  </div>
</details>"#,
        summary_html,
        details_html
    );

    Ok(html)
}

/// Render disclosure with custom open state
pub fn render_disclosure_open(summary: &[DarkMatterNode], details: &[DarkMatterNode], open: bool) -> Result<String, RenderError> {
    let summary_html = render_nodes_to_html(summary)?;
    let details_html = render_nodes_to_html(details)?;

    let open_attr = if open { " open" } else { "" };

    let html = format!(
        r#"<details class="composition-disclosure"{}>
  <summary class="composition-disclosure-summary">
    {}
  </summary>
  <div class="composition-disclosure-content">
    {}
  </div>
</details>"#,
        open_attr,
        summary_html,
        details_html
    );

    Ok(html)
}

/// Generate disclosure CSS styles
pub fn generate_disclosure_styles() -> String {
    r#"
.composition-disclosure {
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  margin: 1rem 0;
  overflow: hidden;
}

.composition-disclosure-summary {
  padding: 1rem;
  cursor: pointer;
  background-color: #f9fafb;
  font-weight: 600;
  user-select: none;
  list-style: none;
  transition: background-color 0.2s ease;
}

.composition-disclosure-summary::-webkit-details-marker {
  display: none;
}

.composition-disclosure-summary::marker {
  display: none;
}

.composition-disclosure-summary::before {
  content: '▶';
  display: inline-block;
  margin-right: 0.5rem;
  transition: transform 0.2s ease;
}

.composition-disclosure[open] .composition-disclosure-summary::before {
  transform: rotate(90deg);
}

.composition-disclosure-summary:hover {
  background-color: #f3f4f6;
}

.composition-disclosure-content {
  padding: 1rem;
  background-color: white;
}

.composition-disclosure[open] .composition-disclosure-summary {
  border-bottom: 1px solid #e5e7eb;
}
"#.to_string()
}

// Helper functions

fn render_nodes_to_html(nodes: &[DarkMatterNode]) -> Result<String, RenderError> {
    let mut html = String::new();

    for node in nodes {
        match node {
            DarkMatterNode::Text(text) => html.push_str(&escape_html(text)),
            DarkMatterNode::Markdown(content) => {
                // For now, just escape the raw content
                // In a full implementation, this would parse markdown to HTML
                html.push_str(&escape_html(&content.raw));
            }
            DarkMatterNode::Interpolation { variable } => {
                // Placeholder - would need frontmatter context
                html.push_str(&format!("{{{{{}}}}}", variable));
            }
            _ => {
                // For other node types, attempt to render as text
                html.push_str(&format!("[Unsupported node type: {:?}]", node));
            }
        }
    }

    Ok(html)
}

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
    fn test_render_disclosure() {
        let summary = vec![DarkMatterNode::Text("Click to expand".to_string())];
        let details = vec![
            DarkMatterNode::Text("This is the hidden content ".to_string()),
            DarkMatterNode::Text("that appears when expanded.".to_string()),
        ];

        let result = render_disclosure(&summary, &details).unwrap();

        assert!(result.contains("<details"));
        assert!(result.contains("<summary"));
        assert!(result.contains("Click to expand"));
        assert!(result.contains("This is the hidden content"));
        assert!(result.contains("composition-disclosure"));
    }

    #[test]
    fn test_render_disclosure_open() {
        let summary = vec![DarkMatterNode::Text("Summary".to_string())];
        let details = vec![DarkMatterNode::Text("Details".to_string())];

        let result_closed = render_disclosure_open(&summary, &details, false).unwrap();
        let result_open = render_disclosure_open(&summary, &details, true).unwrap();

        assert!(!result_closed.contains(" open"));
        assert!(result_open.contains(" open"));
    }

    #[test]
    fn test_disclosure_with_interpolation() {
        let summary = vec![DarkMatterNode::Interpolation {
            variable: "title".to_string(),
        }];
        let details = vec![DarkMatterNode::Text("Content".to_string())];

        let result = render_disclosure(&summary, &details).unwrap();

        assert!(result.contains("{{title}}"));
    }

    #[test]
    fn test_html_escaping_in_disclosure() {
        let summary = vec![DarkMatterNode::Text("<script>alert('xss')</script>".to_string())];
        let details = vec![DarkMatterNode::Text("Safe & sound".to_string())];

        let result = render_disclosure(&summary, &details).unwrap();

        assert!(result.contains("&lt;script&gt;"));
        assert!(result.contains("&amp;"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_generate_disclosure_styles() {
        let styles = generate_disclosure_styles();

        assert!(styles.contains(".composition-disclosure"));
        assert!(styles.contains(".composition-disclosure-summary"));
        assert!(styles.contains(".composition-disclosure-content"));
        assert!(styles.contains("cursor: pointer"));
    }

    #[test]
    fn test_empty_disclosure() {
        let summary = vec![];
        let details = vec![];

        let result = render_disclosure(&summary, &details).unwrap();

        assert!(result.contains("<details"));
        assert!(result.contains("<summary"));
    }

    #[test]
    fn test_multiple_nodes_in_summary() {
        let summary = vec![
            DarkMatterNode::Text("Part 1 ".to_string()),
            DarkMatterNode::Text("Part 2 ".to_string()),
            DarkMatterNode::Text("Part 3".to_string()),
        ];
        let details = vec![DarkMatterNode::Text("Details".to_string())];

        let result = render_disclosure(&summary, &details).unwrap();

        assert!(result.contains("Part 1 Part 2 Part 3"));
    }
}
