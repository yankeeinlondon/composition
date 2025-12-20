use crate::error::ParseError;
use crate::types::Frontmatter;
use yaml_rust2::{Yaml, YamlLoader};

/// Extract YAML frontmatter from markdown content
///
/// Returns (frontmatter, body) tuple where frontmatter is parsed
/// and body is the content after frontmatter delimiter
pub fn extract_frontmatter(content: &str) -> Result<(Frontmatter, &str), ParseError> {
    // Check for frontmatter delimiter
    if !content.starts_with("---") {
        return Ok((Frontmatter::default(), content));
    }

    // Find the closing delimiter
    let after_first_delimiter = &content[3..];

    // Find end of line after first ---
    let first_newline = after_first_delimiter
        .find('\n')
        .ok_or_else(|| ParseError::InvalidFrontmatter("No newline after opening ---".into()))?;

    let yaml_start = 3 + first_newline + 1;
    let remaining = &content[yaml_start..];

    // Check if closing --- is at the very start (empty frontmatter)
    if remaining.starts_with("---\n") {
        let body = &remaining[4..]; // Skip ---\n
        return Ok((Frontmatter::default(), body));
    }

    if remaining.starts_with("---") && remaining.len() == 3 {
        // Empty frontmatter at end of file
        return Ok((Frontmatter::default(), ""));
    }

    // Find closing ---
    if let Some(end_pos) = remaining.find("\n---\n") {
        let yaml_content = &remaining[..end_pos];
        let body = &remaining[end_pos + 5..]; // Skip \n---\n

        // Parse YAML
        let frontmatter = parse_yaml(yaml_content)?;

        Ok((frontmatter, body))
    } else if let Some(end_pos) = remaining.find("\n---") {
        // Check if --- is at end of file
        let potential_body = &remaining[end_pos + 4..];
        if potential_body.trim().is_empty() || potential_body.starts_with('\n') {
            let yaml_content = &remaining[..end_pos];
            let body = potential_body.trim_start_matches('\n');

            let frontmatter = parse_yaml(yaml_content)?;
            Ok((frontmatter, body))
        } else {
            // Not a valid closing delimiter
            Ok((Frontmatter::default(), content))
        }
    } else {
        // No closing delimiter found
        Ok((Frontmatter::default(), content))
    }
}

/// Parse YAML string into Frontmatter struct
fn parse_yaml(yaml_str: &str) -> Result<Frontmatter, ParseError> {
    let docs = YamlLoader::load_from_str(yaml_str)
        .map_err(|e| ParseError::YamlParse(e.to_string()))?;

    if docs.is_empty() {
        return Ok(Frontmatter::default());
    }

    let doc = &docs[0];

    if let Yaml::Hash(hash) = doc {
        let mut frontmatter = Frontmatter::default();

        for (key, value) in hash {
            if let Yaml::String(key_str) = key {
                match key_str.as_str() {
                    "list_expansion" => {
                        if let Some(val_str) = value.as_str() {
                            frontmatter.list_expansion = match val_str {
                                "expanded" => Some(crate::types::ListExpansion::Expanded),
                                "collapsed" => Some(crate::types::ListExpansion::Collapsed),
                                "none" => Some(crate::types::ListExpansion::None),
                                _ => None,
                            };
                        }
                    }
                    "replace" => {
                        if let Yaml::Hash(replace_hash) = value {
                            let mut replace_map = std::collections::HashMap::new();
                            for (k, v) in replace_hash {
                                if let (Yaml::String(k_str), Yaml::String(v_str)) = (k, v) {
                                    replace_map.insert(k_str.clone(), v_str.clone());
                                }
                            }
                            if !replace_map.is_empty() {
                                frontmatter.replace = Some(replace_map);
                            }
                        }
                    }
                    "summarize_model" => {
                        if let Some(val_str) = value.as_str() {
                            frontmatter.summarize_model = Some(val_str.to_string());
                        }
                    }
                    "consolidate_model" => {
                        if let Some(val_str) = value.as_str() {
                            frontmatter.consolidate_model = Some(val_str.to_string());
                        }
                    }
                    "breakpoints" => {
                        if let Yaml::Hash(bp_hash) = value {
                            let mut breakpoints = crate::types::Breakpoints {
                                xs: None,
                                sm: None,
                                md: None,
                                lg: None,
                                xl: None,
                                xxl: None,
                            };

                            for (k, v) in bp_hash {
                                if let Yaml::String(k_str) = k {
                                    if let Some(v_int) = v.as_i64() {
                                        let v_u32 = v_int as u32;
                                        match k_str.as_str() {
                                            "xs" => breakpoints.xs = Some(v_u32),
                                            "sm" => breakpoints.sm = Some(v_u32),
                                            "md" => breakpoints.md = Some(v_u32),
                                            "lg" => breakpoints.lg = Some(v_u32),
                                            "xl" => breakpoints.xl = Some(v_u32),
                                            "xxl" => breakpoints.xxl = Some(v_u32),
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            frontmatter.breakpoints = Some(breakpoints);
                        }
                    }
                    _ => {
                        // Custom field - convert to serde_json::Value
                        if let Ok(json_value) = yaml_to_json(value) {
                            frontmatter.custom.insert(key_str.clone(), json_value);
                        }
                    }
                }
            }
        }

        Ok(frontmatter)
    } else {
        Ok(Frontmatter::default())
    }
}

/// Convert YAML value to serde_json::Value
fn yaml_to_json(yaml: &Yaml) -> Result<serde_json::Value, ParseError> {
    match yaml {
        Yaml::Real(s) | Yaml::String(s) => Ok(serde_json::Value::String(s.clone())),
        Yaml::Integer(i) => Ok(serde_json::Value::Number((*i).into())),
        Yaml::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
        Yaml::Array(arr) => {
            let json_arr: Result<Vec<_>, _> = arr.iter().map(yaml_to_json).collect();
            Ok(serde_json::Value::Array(json_arr?))
        }
        Yaml::Hash(hash) => {
            let mut map = serde_json::Map::new();
            for (k, v) in hash {
                if let Yaml::String(key_str) = k {
                    map.insert(key_str.clone(), yaml_to_json(v)?);
                }
            }
            Ok(serde_json::Value::Object(map))
        }
        Yaml::Null => Ok(serde_json::Value::Null),
        _ => Err(ParseError::YamlParse("Unsupported YAML type".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_frontmatter() {
        let content = "# Hello\n\nThis is content.";
        let (fm, body) = extract_frontmatter(content).unwrap();

        assert_eq!(body, content);
        assert!(fm.custom.is_empty());
    }

    #[test]
    fn test_empty_frontmatter() {
        let content = "---\n---\n# Hello\n\nContent";
        let (fm, body) = extract_frontmatter(content).unwrap();

        assert_eq!(body, "# Hello\n\nContent");
        assert!(fm.custom.is_empty());
    }

    #[test]
    fn test_basic_frontmatter() {
        let content = r#"---
title: My Document
author: John Doe
---
# Hello

Content here"#;

        let (fm, body) = extract_frontmatter(content).unwrap();

        assert_eq!(body, "# Hello\n\nContent here");
        assert_eq!(fm.get_string("title"), Some("My Document"));
        assert_eq!(fm.get_string("author"), Some("John Doe"));
    }

    #[test]
    fn test_list_expansion_frontmatter() {
        let content = r#"---
list_expansion: expanded
---
Content"#;

        let (fm, _) = extract_frontmatter(content).unwrap();
        assert!(matches!(fm.list_expansion, Some(crate::types::ListExpansion::Expanded)));
    }

    #[test]
    fn test_replace_frontmatter() {
        let content = r#"---
replace:
  foo: bar
  hello: world
---
Content"#;

        let (fm, _) = extract_frontmatter(content).unwrap();
        let replace = fm.replace.unwrap();
        assert_eq!(replace.get("foo"), Some(&"bar".to_string()));
        assert_eq!(replace.get("hello"), Some(&"world".to_string()));
    }

    #[test]
    fn test_model_frontmatter() {
        let content = r#"---
summarize_model: gpt-4
consolidate_model: claude-3
---
Content"#;

        let (fm, _) = extract_frontmatter(content).unwrap();
        assert_eq!(fm.summarize_model, Some("gpt-4".to_string()));
        assert_eq!(fm.consolidate_model, Some("claude-3".to_string()));
    }
}
