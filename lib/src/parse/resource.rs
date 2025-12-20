use crate::error::ParseError;
use crate::types::{Resource, ResourceRequirement};
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

/// Parse a resource reference string into a Resource struct
///
/// Handles:
/// - Local file paths (relative/absolute)
/// - URLs (http/https)
/// - Required (!) and optional (?) suffixes
/// - Cache duration overrides
pub fn parse_resource(input: &str) -> Result<Resource, ParseError> {
    let trimmed = input.trim();

    // Check for requirement suffix
    let (path_str, requirement) = if let Some(stripped) = trimmed.strip_suffix('!') {
        (stripped, ResourceRequirement::Required)
    } else if let Some(stripped) = trimmed.strip_suffix('?') {
        (stripped, ResourceRequirement::Optional)
    } else {
        (trimmed, ResourceRequirement::Default)
    };

    // Try to parse as URL first
    if path_str.starts_with("http://") || path_str.starts_with("https://") {
        let url = Url::parse(path_str)?;
        Ok(Resource::remote(url)
            .with_requirement(requirement))
    } else {
        // Treat as local path
        let path = PathBuf::from(path_str);
        Ok(Resource::local(path)
            .with_requirement(requirement))
    }
}

/// Parse multiple resources from a space-separated string
pub fn parse_resources(input: &str) -> Result<Vec<Resource>, ParseError> {
    input
        .split_whitespace()
        .map(parse_resource)
        .collect()
}

/// Parse a resource with optional cache duration override
///
/// Format: "path [cache:duration]"
/// Duration examples: "1h", "30m", "1d"
pub fn parse_resource_with_cache(input: &str) -> Result<Resource, ParseError> {
    let parts: Vec<&str> = input.splitn(2, " cache:").collect();

    let mut resource = parse_resource(parts[0])?;

    if parts.len() == 2 {
        let duration = parse_duration(parts[1])?;
        resource = resource.with_cache_duration(Some(duration));
    }

    Ok(resource)
}

/// Parse duration string (e.g., "1h", "30m", "1d")
fn parse_duration(s: &str) -> Result<Duration, ParseError> {
    let s = s.trim();

    if s.is_empty() {
        return Err(ParseError::InvalidResource("Empty duration".into()));
    }

    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: u64 = num_str.parse()
        .map_err(|_| ParseError::InvalidResource(format!("Invalid duration number: {}", num_str)))?;

    let seconds = match unit {
        "s" => num,
        "m" => num * 60,
        "h" => num * 3600,
        "d" => num * 86400,
        _ => return Err(ParseError::InvalidResource(format!("Invalid duration unit: {}", unit))),
    };

    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ResourceSource;

    #[test]
    fn test_parse_local_path() {
        let resource = parse_resource("./path/to/file.md").unwrap();

        match resource.source {
            ResourceSource::Local(path) => {
                assert_eq!(path, PathBuf::from("./path/to/file.md"));
            }
            _ => panic!("Expected local resource"),
        }
        assert!(matches!(resource.requirement, ResourceRequirement::Default));
    }

    #[test]
    fn test_parse_required_resource() {
        let resource = parse_resource("./file.md!").unwrap();

        assert!(matches!(resource.requirement, ResourceRequirement::Required));
    }

    #[test]
    fn test_parse_optional_resource() {
        let resource = parse_resource("./file.md?").unwrap();

        assert!(matches!(resource.requirement, ResourceRequirement::Optional));
    }

    #[test]
    fn test_parse_url() {
        let resource = parse_resource("https://example.com/doc.md").unwrap();

        match resource.source {
            ResourceSource::Remote(url) => {
                assert_eq!(url.as_str(), "https://example.com/doc.md");
            }
            _ => panic!("Expected remote resource"),
        }

        // Remote resources should have default cache duration
        assert!(resource.cache_duration.is_some());
    }

    #[test]
    fn test_parse_url_with_requirement() {
        let resource = parse_resource("https://example.com/doc.md!").unwrap();

        assert!(matches!(resource.source, ResourceSource::Remote(_)));
        assert!(matches!(resource.requirement, ResourceRequirement::Required));
    }

    #[test]
    fn test_parse_multiple_resources() {
        let resources = parse_resources("./a.md ./b.md https://example.com/c.md").unwrap();

        assert_eq!(resources.len(), 3);
        assert!(matches!(resources[0].source, ResourceSource::Local(_)));
        assert!(matches!(resources[1].source, ResourceSource::Local(_)));
        assert!(matches!(resources[2].source, ResourceSource::Remote(_)));
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("1d").unwrap(), Duration::from_secs(86400));
    }

    #[test]
    fn test_parse_resource_with_cache() {
        let resource = parse_resource_with_cache("https://example.com/doc.md cache:2h").unwrap();

        match resource.source {
            ResourceSource::Remote(_) => {
                assert_eq!(resource.cache_duration, Some(Duration::from_secs(7200)));
            }
            _ => panic!("Expected remote resource"),
        }
    }
}
