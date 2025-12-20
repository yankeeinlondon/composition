use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Frontmatter metadata for DarkMatter documents
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Frontmatter {
    /// User-defined key-values
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,

    /// Reserved darkmatter properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_expansion: Option<ListExpansion>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summarize_model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub consolidate_model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakpoints: Option<Breakpoints>,
}

/// List expansion behavior
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum ListExpansion {
    Expanded,
    Collapsed,
    #[default]
    None,
}

/// Responsive breakpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoints {
    pub xs: Option<u32>,
    pub sm: Option<u32>,
    pub md: Option<u32>,
    pub lg: Option<u32>,
    pub xl: Option<u32>,
    pub xxl: Option<u32>,
}

impl Frontmatter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: Frontmatter) {
        // Merge custom fields (other takes precedence)
        for (key, value) in other.custom {
            self.custom.insert(key, value);
        }

        // Merge reserved fields (other takes precedence if Some)
        if other.list_expansion.is_some() {
            self.list_expansion = other.list_expansion;
        }
        if other.replace.is_some() {
            self.replace = other.replace;
        }
        if other.summarize_model.is_some() {
            self.summarize_model = other.summarize_model;
        }
        if other.consolidate_model.is_some() {
            self.consolidate_model = other.consolidate_model;
        }
        if other.breakpoints.is_some() {
            self.breakpoints = other.breakpoints;
        }
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.custom.get(key).and_then(|v| v.as_str())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.custom.get(key).and_then(|v| v.as_bool())
    }
}
