use crate::error::RenderError;
use crate::types::{DarkMatterNode, Frontmatter};
use regex::Regex;
use std::sync::LazyLock;
use tracing::instrument;

/// Regex pattern for matching {{variable}} interpolation syntax
static INTERPOLATION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*)\}\}").expect("Invalid regex pattern")
});

/// Process frontmatter interpolation in content
///
/// This function:
/// 1. Replaces {{variable}} patterns with values from frontmatter
/// 2. Applies text replacements defined in frontmatter.replace
/// 3. Returns the processed content
#[instrument(skip(frontmatter))]
pub fn process_interpolation(content: &str, frontmatter: &Frontmatter) -> Result<String, RenderError> {
    let mut result = content.to_string();

    // Process {{variable}} patterns
    for cap in INTERPOLATION_REGEX.captures_iter(content) {
        let var_name = &cap[1];
        if let Some(value) = frontmatter.custom.get(var_name) {
            // Convert JSON value to string
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => String::new(),
                _ => {
                    // For complex values (arrays, objects), use JSON representation
                    serde_json::to_string(value)
                        .map_err(|_e| RenderError::InterpolationFailed {
                            variable: var_name.to_string(),
                        })?
                }
            };
            result = result.replace(&cap[0], &replacement);
        }
        // If variable not found, leave it as-is (or could error based on strictness setting)
    }

    // Process text replacements from frontmatter
    if let Some(replacements) = &frontmatter.replace {
        for (from, to) in replacements {
            result = result.replace(from, to);
        }
    }

    Ok(result)
}

/// Recursively process interpolation in all text nodes
pub fn process_nodes_interpolation(
    nodes: &[DarkMatterNode],
    frontmatter: &Frontmatter,
) -> Result<Vec<DarkMatterNode>, RenderError> {
    let mut result = Vec::new();

    for node in nodes {
        let processed = match node {
            DarkMatterNode::Text(text) => {
                DarkMatterNode::Text(process_interpolation(text, frontmatter)?)
            }
            DarkMatterNode::Markdown(content) => {
                let mut new_content = content.clone();
                new_content.raw = process_interpolation(&content.raw, frontmatter)?;
                DarkMatterNode::Markdown(new_content)
            }
            DarkMatterNode::Popover { trigger, content } => {
                let processed_trigger = Box::new(
                    process_nodes_interpolation(&[*trigger.clone()], frontmatter)?
                        .into_iter()
                        .next()
                        .unwrap_or(DarkMatterNode::Text(String::new())),
                );
                let processed_content = process_nodes_interpolation(content, frontmatter)?;
                DarkMatterNode::Popover {
                    trigger: processed_trigger,
                    content: processed_content,
                }
            }
            DarkMatterNode::Columns { breakpoints, sections } => {
                let processed_sections = sections
                    .iter()
                    .map(|section| process_nodes_interpolation(section, frontmatter))
                    .collect::<Result<Vec<_>, _>>()?;
                DarkMatterNode::Columns {
                    breakpoints: breakpoints.clone(),
                    sections: processed_sections,
                }
            }
            DarkMatterNode::Disclosure { summary, details } => {
                let processed_summary = process_nodes_interpolation(summary, frontmatter)?;
                let processed_details = process_nodes_interpolation(details, frontmatter)?;
                DarkMatterNode::Disclosure {
                    summary: processed_summary,
                    details: processed_details,
                }
            }
            // Other node types pass through unchanged
            other => other.clone(),
        };
        result.push(processed);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_interpolation_simple() {
        let mut fm = Frontmatter::default();
        fm.custom.insert(
            "title".to_string(),
            serde_json::Value::String("My Title".to_string()),
        );

        let content = "# {{title}}";
        let result = process_interpolation(content, &fm).unwrap();
        assert_eq!(result, "# My Title");
    }

    #[test]
    fn test_interpolation_multiple() {
        let mut fm = Frontmatter::default();
        fm.custom.insert(
            "author".to_string(),
            serde_json::Value::String("John Doe".to_string()),
        );
        fm.custom.insert(
            "year".to_string(),
            serde_json::Value::Number(2024.into()),
        );

        let content = "Written by {{author}} in {{year}}";
        let result = process_interpolation(content, &fm).unwrap();
        assert_eq!(result, "Written by John Doe in 2024");
    }

    #[test]
    fn test_interpolation_missing_variable() {
        let fm = Frontmatter::default();
        let content = "{{missing}} should remain";
        let result = process_interpolation(content, &fm).unwrap();
        assert_eq!(result, "{{missing}} should remain");
    }

    #[test]
    fn test_interpolation_with_replacements() {
        let mut fm = Frontmatter::default();
        let mut replacements = HashMap::new();
        replacements.insert("old".to_string(), "new".to_string());
        replacements.insert("foo".to_string(), "bar".to_string());
        fm.replace = Some(replacements);

        let content = "This is old text with foo";
        let result = process_interpolation(content, &fm).unwrap();
        assert_eq!(result, "This is new text with bar");
    }

    #[test]
    fn test_interpolation_with_both() {
        let mut fm = Frontmatter::default();
        fm.custom.insert(
            "name".to_string(),
            serde_json::Value::String("Alice".to_string()),
        );
        let mut replacements = HashMap::new();
        replacements.insert("Hello".to_string(), "Hi".to_string());
        fm.replace = Some(replacements);

        let content = "Hello {{name}}!";
        let result = process_interpolation(content, &fm).unwrap();
        assert_eq!(result, "Hi Alice!");
    }

    #[test]
    fn test_interpolation_bool() {
        let mut fm = Frontmatter::default();
        fm.custom.insert(
            "enabled".to_string(),
            serde_json::Value::Bool(true),
        );

        let content = "Enabled: {{enabled}}";
        let result = process_interpolation(content, &fm).unwrap();
        assert_eq!(result, "Enabled: true");
    }

    #[test]
    fn test_interpolation_null() {
        let mut fm = Frontmatter::default();
        fm.custom.insert(
            "empty".to_string(),
            serde_json::Value::Null,
        );

        let content = "Value: {{empty}}";
        let result = process_interpolation(content, &fm).unwrap();
        assert_eq!(result, "Value: ");
    }
}
