use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

/// A resource that can be referenced in DarkMatter documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub source: ResourceSource,
    pub requirement: ResourceRequirement,
    pub cache_duration: Option<Duration>,
}

/// The source location of a resource
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceSource {
    Local(PathBuf),
    Remote(Url),
}

/// Requirement level for a resource (based on suffix syntax)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum ResourceRequirement {
    /// Required - `!` suffix - error if missing
    Required,
    /// Optional - `?` suffix - silent if missing
    Optional,
    /// Default - no suffix - warning if missing
    #[default]
    Default,
}

impl Resource {
    pub fn local(path: PathBuf) -> Self {
        Self {
            source: ResourceSource::Local(path),
            requirement: ResourceRequirement::Default,
            cache_duration: None,
        }
    }

    pub fn remote(url: Url) -> Self {
        Self {
            source: ResourceSource::Remote(url),
            requirement: ResourceRequirement::Default,
            cache_duration: Some(Duration::from_secs(86400)), // 1 day default
        }
    }

    pub fn with_requirement(mut self, requirement: ResourceRequirement) -> Self {
        self.requirement = requirement;
        self
    }

    pub fn with_cache_duration(mut self, duration: Option<Duration>) -> Self {
        self.cache_duration = duration;
        self
    }
}

/// Hash type for resource identification
pub type ResourceHash = u64;
