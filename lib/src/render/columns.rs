use crate::types::{DarkMatterNode, Breakpoint};
use crate::error::RenderError;
use std::collections::HashMap;

/// Render a multi-column layout with responsive breakpoints
pub fn render_columns(
    breakpoints: &HashMap<Breakpoint, u32>,
    sections: &[Vec<DarkMatterNode>],
) -> Result<String, RenderError> {
    if sections.is_empty() {
        return Ok(String::new());
    }

    let column_class = generate_column_class(breakpoints);

    let mut html = format!(r#"<div class="composition-columns {}">"#, column_class);

    for section in sections {
        html.push_str(r#"<div class="composition-column">"#);

        let section_html = render_nodes_to_html(section)?;
        html.push_str(&section_html);

        html.push_str(r#"</div>"#);
    }

    html.push_str(r#"</div>"#);

    Ok(html)
}

/// Generate CSS class name for column configuration
fn generate_column_class(breakpoints: &HashMap<Breakpoint, u32>) -> String {
    if breakpoints.is_empty() {
        return "composition-columns-default".to_string();
    }

    let mut classes = Vec::new();

    // Sort breakpoints by size for consistent ordering
    let mut breakpoint_list: Vec<_> = breakpoints.iter().collect();
    breakpoint_list.sort_by_key(|(bp, _)| breakpoint_order(bp));

    for (bp, cols) in breakpoint_list {
        let bp_name = breakpoint_name(bp);
        classes.push(format!("{}-{}", bp_name, cols));
    }

    format!("composition-columns-{}", classes.join("-"))
}

/// Generate CSS styles for column layouts with breakpoints
pub fn generate_columns_styles(breakpoints: &HashMap<Breakpoint, u32>) -> String {
    let mut styles = String::from(
        r#"
.composition-columns {
  display: grid;
  gap: 2rem;
  margin: 1rem 0;
}

.composition-column {
  min-width: 0;
}
"#,
    );

    if breakpoints.is_empty() {
        // Default 2-column layout
        styles.push_str(
            r#"
.composition-columns-default {
  grid-template-columns: 1fr;
}

@media (min-width: 768px) {
  .composition-columns-default {
    grid-template-columns: repeat(2, 1fr);
  }
}
"#,
        );
        return styles;
    }

    let class_name = generate_column_class(breakpoints);

    // Generate styles for each breakpoint
    let mut breakpoint_list: Vec<_> = breakpoints.iter().collect();
    breakpoint_list.sort_by_key(|(bp, _)| breakpoint_order(bp));

    for (i, (bp, cols)) in breakpoint_list.iter().enumerate() {
        let bp_px = breakpoint_pixels(bp);

        if i == 0 && **bp == Breakpoint::Xs {
            // Base styles (no media query for xs)
            styles.push_str(&format!(
                r#"
.{} {{
  grid-template-columns: repeat({}, 1fr);
}}
"#,
                class_name, cols
            ));
        } else {
            // Media query for larger breakpoints
            styles.push_str(&format!(
                r#"
@media (min-width: {}px) {{
  .{} {{
    grid-template-columns: repeat({}, 1fr);
  }}
}}
"#,
                bp_px, class_name, cols
            ));
        }
    }

    styles
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

fn breakpoint_name(bp: &Breakpoint) -> &'static str {
    match bp {
        Breakpoint::Xs => "xs",
        Breakpoint::Sm => "sm",
        Breakpoint::Md => "md",
        Breakpoint::Lg => "lg",
        Breakpoint::Xl => "xl",
        Breakpoint::Xxl => "xxl",
    }
}

fn breakpoint_pixels(bp: &Breakpoint) -> u32 {
    match bp {
        Breakpoint::Xs => 0,
        Breakpoint::Sm => 640,
        Breakpoint::Md => 768,
        Breakpoint::Lg => 1024,
        Breakpoint::Xl => 1280,
        Breakpoint::Xxl => 1536,
    }
}

fn breakpoint_order(bp: &Breakpoint) -> u8 {
    match bp {
        Breakpoint::Xs => 0,
        Breakpoint::Sm => 1,
        Breakpoint::Md => 2,
        Breakpoint::Lg => 3,
        Breakpoint::Xl => 4,
        Breakpoint::Xxl => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_columns_basic() {
        let breakpoints = HashMap::new();
        let sections = vec![
            vec![DarkMatterNode::Text("Column 1".to_string())],
            vec![DarkMatterNode::Text("Column 2".to_string())],
        ];

        let result = render_columns(&breakpoints, &sections).unwrap();

        assert!(result.contains("composition-columns"));
        assert!(result.contains("composition-column"));
        assert!(result.contains("Column 1"));
        assert!(result.contains("Column 2"));
    }

    #[test]
    fn test_render_columns_with_breakpoints() {
        let mut breakpoints = HashMap::new();
        breakpoints.insert(Breakpoint::Xs, 1);
        breakpoints.insert(Breakpoint::Md, 2);
        breakpoints.insert(Breakpoint::Lg, 3);

        let sections = vec![
            vec![DarkMatterNode::Text("A".to_string())],
            vec![DarkMatterNode::Text("B".to_string())],
            vec![DarkMatterNode::Text("C".to_string())],
        ];

        let result = render_columns(&breakpoints, &sections).unwrap();

        assert!(result.contains("composition-columns"));
        assert!(result.contains("A"));
        assert!(result.contains("B"));
        assert!(result.contains("C"));
    }

    #[test]
    fn test_generate_column_class() {
        let mut breakpoints = HashMap::new();
        breakpoints.insert(Breakpoint::Md, 2);
        breakpoints.insert(Breakpoint::Lg, 3);

        let class = generate_column_class(&breakpoints);

        assert!(class.contains("composition-columns"));
        assert!(class.contains("md-2"));
        assert!(class.contains("lg-3"));
    }

    #[test]
    fn test_generate_column_class_empty() {
        let breakpoints = HashMap::new();
        let class = generate_column_class(&breakpoints);

        assert_eq!(class, "composition-columns-default");
    }

    #[test]
    fn test_generate_columns_styles_default() {
        let breakpoints = HashMap::new();
        let styles = generate_columns_styles(&breakpoints);

        assert!(styles.contains(".composition-columns"));
        assert!(styles.contains("grid-template-columns"));
        assert!(styles.contains("composition-columns-default"));
    }

    #[test]
    fn test_generate_columns_styles_with_breakpoints() {
        let mut breakpoints = HashMap::new();
        breakpoints.insert(Breakpoint::Md, 2);
        breakpoints.insert(Breakpoint::Lg, 3);

        let styles = generate_columns_styles(&breakpoints);

        assert!(styles.contains("@media (min-width: 768px)"));
        assert!(styles.contains("@media (min-width: 1024px)"));
        assert!(styles.contains("repeat(2, 1fr)"));
        assert!(styles.contains("repeat(3, 1fr)"));
    }

    #[test]
    fn test_breakpoint_pixels() {
        assert_eq!(breakpoint_pixels(&Breakpoint::Xs), 0);
        assert_eq!(breakpoint_pixels(&Breakpoint::Sm), 640);
        assert_eq!(breakpoint_pixels(&Breakpoint::Md), 768);
        assert_eq!(breakpoint_pixels(&Breakpoint::Lg), 1024);
        assert_eq!(breakpoint_pixels(&Breakpoint::Xl), 1280);
        assert_eq!(breakpoint_pixels(&Breakpoint::Xxl), 1536);
    }

    #[test]
    fn test_empty_sections() {
        let breakpoints = HashMap::new();
        let sections: Vec<Vec<DarkMatterNode>> = vec![];

        let result = render_columns(&breakpoints, &sections).unwrap();

        assert_eq!(result, "");
    }

    #[test]
    fn test_html_escaping_in_columns() {
        let breakpoints = HashMap::new();
        let sections = vec![vec![DarkMatterNode::Text("<script>alert('xss')</script>".to_string())]];

        let result = render_columns(&breakpoints, &sections).unwrap();

        assert!(result.contains("&lt;script&gt;"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_single_column() {
        let breakpoints = HashMap::new();
        let sections = vec![vec![DarkMatterNode::Text("Solo".to_string())]];

        let result = render_columns(&breakpoints, &sections).unwrap();

        assert!(result.contains("composition-columns"));
        assert!(result.contains("Solo"));
    }

    #[test]
    fn test_multiple_nodes_per_column() {
        let breakpoints = HashMap::new();
        let sections = vec![vec![
            DarkMatterNode::Text("First ".to_string()),
            DarkMatterNode::Text("Second ".to_string()),
            DarkMatterNode::Text("Third".to_string()),
        ]];

        let result = render_columns(&breakpoints, &sections).unwrap();

        assert!(result.contains("First Second Third"));
    }
}
