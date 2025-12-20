use crate::types::DarkMatterNode;
use crate::error::RenderError;

/// Render a popover to HTML with CSS classes
pub fn render_popover(trigger: &DarkMatterNode, content: &[DarkMatterNode]) -> Result<String, RenderError> {
    let trigger_html = render_node_to_text(trigger)?;
    let content_html = render_nodes_to_html(content)?;

    // Generate unique ID for this popover
    let popover_id = format!("popover-{}", generate_id());

    let html = format!(
        r#"<span class="composition-popover-wrapper">
  <button class="composition-popover-trigger" data-popover-target="{}">
    {}
  </button>
  <div id="{}" class="composition-popover-content" role="tooltip">
    <div class="composition-popover-arrow"></div>
    <div class="composition-popover-body">
      {}
    </div>
  </div>
</span>"#,
        popover_id,
        trigger_html,
        popover_id,
        content_html
    );

    Ok(html)
}

/// Render inline popover link syntax [text](popover:content)
pub fn render_inline_popover(trigger_text: &str, popover_content: &str) -> Result<String, RenderError> {
    let popover_id = format!("popover-{}", generate_id());

    let html = format!(
        r#"<span class="composition-popover-wrapper">
  <button class="composition-popover-trigger" data-popover-target="{}">
    {}
  </button>
  <div id="{}" class="composition-popover-content" role="tooltip">
    <div class="composition-popover-arrow"></div>
    <div class="composition-popover-body">
      {}
    </div>
  </div>
</span>"#,
        popover_id,
        escape_html(trigger_text),
        popover_id,
        escape_html(popover_content)
    );

    Ok(html)
}

/// Generate popover CSS styles
pub fn generate_popover_styles() -> String {
    r#"
.composition-popover-wrapper {
  position: relative;
  display: inline-block;
}

.composition-popover-trigger {
  background: none;
  border: none;
  color: #3b82f6;
  text-decoration: underline;
  text-decoration-style: dotted;
  cursor: pointer;
  padding: 0;
  font: inherit;
}

.composition-popover-trigger:hover {
  color: #2563eb;
}

.composition-popover-content {
  display: none;
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-bottom: 8px;
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
  padding: 12px;
  max-width: 300px;
  z-index: 1000;
}

.composition-popover-content.show {
  display: block;
}

.composition-popover-arrow {
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  border-top: 8px solid white;
}

.composition-popover-arrow::before {
  content: '';
  position: absolute;
  top: -9px;
  left: -8px;
  width: 0;
  height: 0;
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  border-top: 8px solid #e5e7eb;
}

.composition-popover-body {
  font-size: 14px;
  line-height: 1.5;
}
"#.to_string()
}

/// Generate JavaScript for popover interactivity
pub fn generate_popover_script() -> String {
    r#"
document.addEventListener('DOMContentLoaded', function() {
  // Handle popover triggers
  document.querySelectorAll('.composition-popover-trigger').forEach(function(trigger) {
    trigger.addEventListener('click', function(e) {
      e.preventDefault();
      const targetId = this.getAttribute('data-popover-target');
      const popover = document.getElementById(targetId);

      // Close other popovers
      document.querySelectorAll('.composition-popover-content.show').forEach(function(other) {
        if (other.id !== targetId) {
          other.classList.remove('show');
        }
      });

      // Toggle this popover
      popover.classList.toggle('show');
    });
  });

  // Close popover when clicking outside
  document.addEventListener('click', function(e) {
    if (!e.target.closest('.composition-popover-wrapper')) {
      document.querySelectorAll('.composition-popover-content.show').forEach(function(popover) {
        popover.classList.remove('show');
      });
    }
  });
});
"#.to_string()
}

// Helper functions

fn render_node_to_text(node: &DarkMatterNode) -> Result<String, RenderError> {
    match node {
        DarkMatterNode::Text(text) => Ok(escape_html(text)),
        DarkMatterNode::Markdown(content) => Ok(escape_html(&content.raw)),
        _ => Err(RenderError::PopoverError("Unsupported node type for popover trigger".to_string())),
    }
}

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
            _ => return Err(RenderError::PopoverError("Unsupported node type in popover content".to_string())),
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

use std::sync::atomic::{AtomicUsize, Ordering};

static POPOVER_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn generate_id() -> usize {
    POPOVER_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_inline_popover() {
        let result = render_inline_popover("Click me", "This is the popover content").unwrap();

        assert!(result.contains("composition-popover-wrapper"));
        assert!(result.contains("composition-popover-trigger"));
        assert!(result.contains("Click me"));
        assert!(result.contains("This is the popover content"));
        assert!(result.contains("data-popover-target="));
    }

    #[test]
    fn test_render_popover_with_nodes() {
        let trigger = DarkMatterNode::Text("Trigger text".to_string());
        let content = vec![
            DarkMatterNode::Text("Popover ".to_string()),
            DarkMatterNode::Text("content".to_string()),
        ];

        let result = render_popover(&trigger, &content).unwrap();

        assert!(result.contains("Trigger text"));
        assert!(result.contains("Popover content"));
        assert!(result.contains("composition-popover-wrapper"));
    }

    #[test]
    fn test_html_escaping() {
        let result = render_inline_popover("<script>alert('xss')</script>", "Content & stuff").unwrap();

        assert!(result.contains("&lt;script&gt;"));
        assert!(result.contains("&amp;"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_generate_popover_styles() {
        let styles = generate_popover_styles();

        assert!(styles.contains(".composition-popover-wrapper"));
        assert!(styles.contains(".composition-popover-trigger"));
        assert!(styles.contains(".composition-popover-content"));
        assert!(styles.contains(".composition-popover-arrow"));
    }

    #[test]
    fn test_generate_popover_script() {
        let script = generate_popover_script();

        assert!(script.contains("addEventListener"));
        assert!(script.contains("composition-popover-trigger"));
        assert!(script.contains("data-popover-target"));
    }

    #[test]
    fn test_unique_ids() {
        let result1 = render_inline_popover("A", "B").unwrap();
        let result2 = render_inline_popover("C", "D").unwrap();

        // Extract IDs from results
        assert_ne!(result1, result2);
        assert!(result1.contains("popover-"));
        assert!(result2.contains("popover-"));
    }
}
