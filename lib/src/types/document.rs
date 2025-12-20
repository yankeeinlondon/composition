use super::{DarkMatterNode, Frontmatter, Resource};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A parsed DarkMatter document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub resource: Resource,
    pub frontmatter: Frontmatter,
    pub content: Vec<DarkMatterNode>,
    pub dependencies: Vec<Resource>,
    pub parsed_at: DateTime<Utc>,
}

impl Document {
    pub fn new(resource: Resource) -> Self {
        Self {
            resource,
            frontmatter: Frontmatter::default(),
            content: Vec::new(),
            dependencies: Vec::new(),
            parsed_at: Utc::now(),
        }
    }

    pub fn with_frontmatter(mut self, frontmatter: Frontmatter) -> Self {
        self.frontmatter = frontmatter;
        self
    }

    pub fn with_content(mut self, content: Vec<DarkMatterNode>) -> Self {
        self.content = content;
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<Resource>) -> Self {
        self.dependencies = dependencies;
        self
    }
}
