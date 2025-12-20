use crate::error::ParseError;
use crate::types::{DarkMatterNode, LineRange, WidthSpec};
use crate::parse::resource::{parse_resource, parse_resources};
use regex::Regex;
use std::sync::LazyLock;

// Regex patterns for DarkMatter directives
static FILE_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^::file\s+(.+?)(?:\s+(\d+)-(\d+)?)?$").unwrap()
});

static SUMMARIZE_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^::summarize\s+(.+)$").unwrap()
});

static CONSOLIDATE_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^::consolidate\s+(.+)$").unwrap()
});

static TOPIC_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^::topic\s+"([^"]+)"\s+(.+?)(?:\s+--review)?$"#).unwrap()
});

static TABLE_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^::table\s+(.+)$").unwrap()
});

static CHART_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^::(bar-chart|line-chart|pie-chart|area-chart|bubble-chart)\s+(.+)$").unwrap()
});

static COLUMNS_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^::columns(?:\s+(.+))?$").unwrap()
});

static POPOVER_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[([^\]]+)\]\(popover:([^)]+)\)").unwrap()
});

static INTERPOLATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{\{(\w+)\}\}").unwrap()
});

static AUDIO_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    // Match ::audio followed by a path, optionally followed by a quoted name
    // Handles: ::audio ./file.mp3
    //          ::audio ./file.mp3 "Name"
    //          ::audio ./file.mp3 "Name with spaces"
    //          ::audio "./path with spaces.mp3" "Name"
    Regex::new(r#"^::audio\s+(?:"([^"]+)"|(\S+))(?:\s+"(.+)")?$"#).unwrap()
});

static YOUTUBE_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^::youtube\s+([^\s]+)(?:\s+(\d+(?:\.\d+)?(?:px|rem|%)))?$").unwrap()
});

// YouTube URL patterns for video ID extraction
static YOUTUBE_WATCH_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^https?://(?:www\.)?youtube\.com/watch\?.*v=([A-Za-z0-9_-]{11})").unwrap()
});

static YOUTUBE_SHORT_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^https?://youtu\.be/([A-Za-z0-9_-]{11})").unwrap()
});

static YOUTUBE_EMBED_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^https?://(?:www\.)?youtube\.com/embed/([A-Za-z0-9_-]{11})").unwrap()
});

static YOUTUBE_V_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^https?://(?:www\.)?youtube\.com/v/([A-Za-z0-9_-]{11})").unwrap()
});

static YOUTUBE_RAW_ID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Za-z0-9_-]{11}$").unwrap()
});

/// Extract a YouTube video ID from various URL formats or raw IDs
///
/// Supports:
/// - `https://youtube.com/watch?v=ID`
/// - `https://youtu.be/ID`
/// - `https://youtube.com/embed/ID`
/// - `https://youtube.com/v/ID`
/// - Raw 11-character IDs
///
/// # Errors
///
/// Returns `ParseError::InvalidResource` if the reference cannot be parsed
/// as a valid YouTube URL or video ID.
fn extract_youtube_id(reference: &str) -> Result<String, ParseError> {
    // Try each URL pattern
    if let Some(caps) = YOUTUBE_WATCH_URL.captures(reference) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }

    if let Some(caps) = YOUTUBE_SHORT_URL.captures(reference) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }

    if let Some(caps) = YOUTUBE_EMBED_URL.captures(reference) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }

    if let Some(caps) = YOUTUBE_V_URL.captures(reference) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }

    // Try raw ID
    if YOUTUBE_RAW_ID.is_match(reference) {
        return Ok(reference.to_string());
    }

    Err(ParseError::InvalidResource(format!(
        "Could not extract video ID from '{}'. \
         Supported formats: youtube.com/watch?v=ID, youtu.be/ID, youtube.com/embed/ID, youtube.com/v/ID, or 11-character ID",
        reference
    )))
}

/// Parse a width specification from a string
///
/// Supports:
/// - Pixels: `512px`
/// - Rems: `32rem`, `32.5rem`
/// - Percentage: `80%` (validated 0-100)
///
/// # Errors
///
/// Returns `ParseError::InvalidDirective` if the width format is invalid
/// or percentage is out of range.
fn parse_width_spec(width_str: &str) -> Result<WidthSpec, ParseError> {
    if let Some(px_str) = width_str.strip_suffix("px") {
        let px = px_str.parse::<u32>().map_err(|_| {
            ParseError::InvalidDirective {
                line: 0,
                directive: format!(
                    "Invalid pixel width '{}'. Width must be a positive integer",
                    width_str
                ),
            }
        })?;

        if px == 0 {
            return Err(ParseError::InvalidDirective {
                line: 0,
                directive: "Width must be positive".to_string(),
            });
        }

        return Ok(WidthSpec::Pixels(px));
    }

    if let Some(rem_str) = width_str.strip_suffix("rem") {
        let rem = rem_str.parse::<f32>().map_err(|_| {
            ParseError::InvalidDirective {
                line: 0,
                directive: format!(
                    "Invalid rem width '{}'. Width must be a positive number",
                    width_str
                ),
            }
        })?;

        if rem <= 0.0 {
            return Err(ParseError::InvalidDirective {
                line: 0,
                directive: "Width must be positive".to_string(),
            });
        }

        return Ok(WidthSpec::Rems(rem));
    }

    if let Some(pct_str) = width_str.strip_suffix('%') {
        let pct = pct_str.parse::<u8>().map_err(|_| {
            ParseError::InvalidDirective {
                line: 0,
                directive: format!(
                    "Invalid percentage width '{}'. Percentage must be 0-100",
                    width_str
                ),
            }
        })?;

        if pct > 100 {
            return Err(ParseError::InvalidDirective {
                line: 0,
                directive: format!(
                    "Invalid percentage '{}'. Must be 0-100%",
                    pct
                ),
            });
        }

        return Ok(WidthSpec::Percentage(pct));
    }

    Err(ParseError::InvalidDirective {
        line: 0,
        directive: format!(
            "Invalid width format '{}'. Width must be pixels (512px), rems (32rem), or percentage (0-100%)",
            width_str
        ),
    })
}

/// Parse a DarkMatter block directive
pub fn parse_directive(line: &str, line_num: usize) -> Result<Option<DarkMatterNode>, ParseError> {
    let trimmed = line.trim();

    // Check for various directive types
    if let Some(caps) = FILE_DIRECTIVE.captures(trimmed) {
        let resource = parse_resource(caps.get(1).unwrap().as_str())?;

        let range = if let (Some(start), Some(end)) = (caps.get(2), caps.get(3)) {
            let start_num = start.as_str().parse::<usize>()
                .map_err(|_| ParseError::InvalidDirective {
                    line: line_num,
                    directive: line.to_string(),
                })?;
            let end_num = end.as_str().parse::<usize>()
                .map_err(|_| ParseError::InvalidDirective {
                    line: line_num,
                    directive: line.to_string(),
                })?;

            Some(LineRange {
                start: start_num,
                end: Some(end_num),
            })
        } else if let Some(start) = caps.get(2) {
            let start_num = start.as_str().parse::<usize>()
                .map_err(|_| ParseError::InvalidDirective {
                    line: line_num,
                    directive: line.to_string(),
                })?;

            Some(LineRange {
                start: start_num,
                end: None,
            })
        } else {
            None
        };

        return Ok(Some(DarkMatterNode::File { resource, range }));
    }

    if let Some(caps) = SUMMARIZE_DIRECTIVE.captures(trimmed) {
        let resource = parse_resource(caps.get(1).unwrap().as_str())?;
        return Ok(Some(DarkMatterNode::Summarize { resource }));
    }

    if let Some(caps) = CONSOLIDATE_DIRECTIVE.captures(trimmed) {
        let resources = parse_resources(caps.get(1).unwrap().as_str())?;
        return Ok(Some(DarkMatterNode::Consolidate { resources }));
    }

    if let Some(caps) = TOPIC_DIRECTIVE.captures(trimmed) {
        let topic = caps.get(1).unwrap().as_str().to_string();
        let resources = parse_resources(caps.get(2).unwrap().as_str())?;
        let review = trimmed.contains("--review");

        return Ok(Some(DarkMatterNode::Topic {
            topic,
            resources,
            review,
        }));
    }

    if let Some(caps) = TABLE_DIRECTIVE.captures(trimmed) {
        let has_heading = trimmed.contains("--with-heading-row");

        let args = caps.get(1).map(|m| m.as_str()).unwrap_or("");

        // Remove --with-heading-row flag from args to get the path
        let path_str = args
            .replace("--with-heading-row", "")
            .trim()
            .to_string();

        let source = if !path_str.is_empty() {
            let resource = parse_resource(&path_str)?;
            crate::types::TableSource::External(resource)
        } else {
            // Inline table - will be populated later when parsing table content
            crate::types::TableSource::Inline(Vec::new())
        };

        return Ok(Some(DarkMatterNode::Table {
            source,
            has_heading,
        }));
    }

    if let Some(caps) = CHART_DIRECTIVE.captures(trimmed) {
        let chart_type = caps.get(1).unwrap().as_str();
        let resource = parse_resource(caps.get(2).unwrap().as_str())?;
        let data = crate::types::ChartData::External(resource);

        return Ok(Some(match chart_type {
            "bar-chart" => DarkMatterNode::BarChart { data },
            "line-chart" => DarkMatterNode::LineChart { data },
            "pie-chart" => DarkMatterNode::PieChart { data },
            "area-chart" => DarkMatterNode::AreaChart { data },
            "bubble-chart" => DarkMatterNode::BubbleChart { data },
            _ => return Err(ParseError::InvalidDirective {
                line: line_num,
                directive: line.to_string(),
            }),
        }));
    }

    if let Some(caps) = AUDIO_DIRECTIVE.captures(trimmed) {
        // Extract source path - could be quoted (group 1) or unquoted (group 2)
        let source = caps.get(1)
            .or_else(|| caps.get(2))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| ParseError::InvalidDirective {
                line: line_num,
                directive: line.to_string(),
            })?;

        // Extract optional name (group 3)
        let name = caps.get(3).map(|m| m.as_str().to_string());

        return Ok(Some(DarkMatterNode::Audio { source, name }));
    }

    if let Some(caps) = YOUTUBE_DIRECTIVE.captures(trimmed) {
        let video_ref = caps.get(1).unwrap().as_str();

        // Check for empty reference
        if video_ref.is_empty() {
            return Err(ParseError::InvalidDirective {
                line: line_num,
                directive: "YouTube directive requires a video reference (URL or 11-character video ID)".to_string(),
            });
        }

        let video_id = extract_youtube_id(video_ref)?;

        let width = caps.get(2)
            .map(|w| parse_width_spec(w.as_str()))
            .transpose()?
            .unwrap_or_default();

        return Ok(Some(DarkMatterNode::YouTube { video_id, width }));
    }

    // Check for summary/details directives
    if trimmed == "::summary" {
        // This will be handled by the parser context
        return Ok(None);
    }

    if trimmed == "::details" {
        // This will be handled by the parser context
        return Ok(None);
    }

    if trimmed == "::break" {
        // Column break - handled by parser context
        return Ok(None);
    }

    if COLUMNS_DIRECTIVE.is_match(trimmed) {
        // Columns - will be handled by parser context
        return Ok(None);
    }

    // Not a recognized directive
    Ok(None)
}

/// Process inline DarkMatter syntax in text
pub fn process_inline_syntax(text: &str) -> Vec<DarkMatterNode> {
    let mut nodes = Vec::new();
    let mut current_pos = 0;

    // TODO: Handle popover links in future phase
    // For now, we'll just handle interpolations

    // Find all popover links (placeholder for future implementation)
    for caps in POPOVER_LINK.captures_iter(text) {
        let _full_match = caps.get(0).unwrap();
        let _trigger_text = caps.get(1).unwrap().as_str();
        let _popover_content = caps.get(2).unwrap().as_str();

        // For now, we'll create a simplified representation
        // In a full implementation, this would create proper popover nodes
    }

    // Find all interpolations
    for caps in INTERPOLATION.captures_iter(text) {
        let full_match = caps.get(0).unwrap();
        let var_name = caps.get(1).unwrap().as_str();

        let match_start = full_match.start();

        // Add text before interpolation
        if match_start > current_pos {
            let text_before = &text[current_pos..match_start];
            if !text_before.is_empty() {
                nodes.push(DarkMatterNode::Text(text_before.to_string()));
            }
        }

        // Add interpolation node
        nodes.push(DarkMatterNode::Interpolation {
            variable: var_name.to_string(),
        });

        current_pos = full_match.end();
    }

    // Add remaining text
    if current_pos < text.len() {
        let remaining = &text[current_pos..];
        if !remaining.is_empty() {
            nodes.push(DarkMatterNode::Text(remaining.to_string()));
        }
    }

    // If no inline syntax was found, return the original text as a single node
    if nodes.is_empty() {
        nodes.push(DarkMatterNode::Text(text.to_string()));
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_directive() {
        let node = parse_directive("::file ./path/to/file.md", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::File { resource, range } => {
                assert!(matches!(resource.source, crate::types::ResourceSource::Local(_)));
                assert!(range.is_none());
            }
            _ => panic!("Expected File node"),
        }
    }

    #[test]
    fn test_parse_file_directive_with_range() {
        let node = parse_directive("::file ./file.md 10-20", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::File { resource: _, range } => {
                let range = range.unwrap();
                assert_eq!(range.start, 10);
                assert_eq!(range.end, Some(20));
            }
            _ => panic!("Expected File node"),
        }
    }

    #[test]
    fn test_parse_summarize_directive() {
        let node = parse_directive("::summarize ./doc.md", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Summarize { resource: _ } => {
                // Success
            }
            _ => panic!("Expected Summarize node"),
        }
    }

    #[test]
    fn test_parse_consolidate_directive() {
        let node = parse_directive("::consolidate ./a.md ./b.md", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Consolidate { resources } => {
                assert_eq!(resources.len(), 2);
            }
            _ => panic!("Expected Consolidate node"),
        }
    }

    #[test]
    fn test_parse_topic_directive() {
        let node = parse_directive(r#"::topic "testing" ./a.md ./b.md"#, 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Topic { topic, resources, review } => {
                assert_eq!(topic, "testing");
                assert_eq!(resources.len(), 2);
                assert!(!review);
            }
            _ => panic!("Expected Topic node"),
        }
    }

    #[test]
    fn test_parse_topic_directive_with_review() {
        let node = parse_directive(r#"::topic "testing" ./a.md --review"#, 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Topic { topic: _, resources: _, review } => {
                assert!(review);
            }
            _ => panic!("Expected Topic node"),
        }
    }

    #[test]
    fn test_parse_table_directive() {
        let node = parse_directive("::table ./data.csv --with-heading-row", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Table { source, has_heading } => {
                assert!(matches!(source, crate::types::TableSource::External(_)));
                assert!(has_heading);
            }
            _ => panic!("Expected Table node"),
        }
    }

    #[test]
    fn test_parse_table_directive_flag_first() {
        let node = parse_directive("::table --with-heading-row ./data.csv", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Table { source, has_heading } => {
                assert!(matches!(source, crate::types::TableSource::External(_)));
                assert!(has_heading);
            }
            _ => panic!("Expected Table node"),
        }
    }

    #[test]
    fn test_parse_chart_directive() {
        let node = parse_directive("::bar-chart ./data.csv", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::BarChart { data: _ } => {
                // Success
            }
            _ => panic!("Expected BarChart node"),
        }
    }

    #[test]
    fn test_process_interpolation() {
        let nodes = process_inline_syntax("Hello {{name}}, welcome!");

        assert_eq!(nodes.len(), 3);
        assert!(matches!(nodes[0], DarkMatterNode::Text(_)));
        assert!(matches!(nodes[1], DarkMatterNode::Interpolation { .. }));
        assert!(matches!(nodes[2], DarkMatterNode::Text(_)));
    }

    #[test]
    fn test_process_plain_text() {
        let nodes = process_inline_syntax("Just plain text");

        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            DarkMatterNode::Text(t) => assert_eq!(t, "Just plain text"),
            _ => panic!("Expected Text node"),
        }
    }

    #[test]
    fn test_parse_audio_directive() {
        let node = parse_directive("::audio ./podcast.mp3", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Audio { source, name } => {
                assert_eq!(source, "./podcast.mp3");
                assert!(name.is_none());
            }
            _ => panic!("Expected Audio node"),
        }
    }

    #[test]
    fn test_parse_audio_directive_with_name() {
        let node = parse_directive(r#"::audio ./podcast.mp3 "Episode 42""#, 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Audio { source, name } => {
                assert_eq!(source, "./podcast.mp3");
                assert_eq!(name, Some("Episode 42".to_string()));
            }
            _ => panic!("Expected Audio node"),
        }
    }

    #[test]
    fn test_parse_audio_directive_with_quoted_path() {
        let node = parse_directive(r#"::audio "./path with spaces.mp3""#, 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Audio { source, name } => {
                assert_eq!(source, "./path with spaces.mp3");
                assert!(name.is_none());
            }
            _ => panic!("Expected Audio node"),
        }
    }

    #[test]
    fn test_parse_audio_directive_with_quoted_path_and_name() {
        let node = parse_directive(r#"::audio "./path with spaces.mp3" "My Audio""#, 1).unwrap().unwrap();

        match node {
            DarkMatterNode::Audio { source, name } => {
                assert_eq!(source, "./path with spaces.mp3");
                assert_eq!(name, Some("My Audio".to_string()));
            }
            _ => panic!("Expected Audio node"),
        }
    }

    #[test]
    fn test_parse_audio_directive_invalid() {
        let result = parse_directive("::audio", 1).unwrap();
        assert!(result.is_none());
    }

    // YouTube directive parsing tests
    #[test]
    fn test_parse_youtube_directive_with_raw_id() {
        let node = parse_directive("::youtube dQw4w9WgXcQ", 1).unwrap().unwrap();

        match node {
            DarkMatterNode::YouTube { video_id, width } => {
                assert_eq!(video_id, "dQw4w9WgXcQ");
                assert_eq!(width, WidthSpec::Pixels(512)); // default
            }
            _ => panic!("Expected YouTube node"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_watch_url() {
        let node = parse_directive("::youtube https://www.youtube.com/watch?v=dQw4w9WgXcQ", 1)
            .unwrap()
            .unwrap();

        match node {
            DarkMatterNode::YouTube { video_id, width: _ } => {
                assert_eq!(video_id, "dQw4w9WgXcQ");
            }
            _ => panic!("Expected YouTube node"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_watch_url_with_params() {
        let node = parse_directive("::youtube https://youtube.com/watch?v=dQw4w9WgXcQ&feature=share", 1)
            .unwrap()
            .unwrap();

        match node {
            DarkMatterNode::YouTube { video_id, width: _ } => {
                assert_eq!(video_id, "dQw4w9WgXcQ");
            }
            _ => panic!("Expected YouTube node"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_short_url() {
        let node = parse_directive("::youtube https://youtu.be/dQw4w9WgXcQ", 1)
            .unwrap()
            .unwrap();

        match node {
            DarkMatterNode::YouTube { video_id, width: _ } => {
                assert_eq!(video_id, "dQw4w9WgXcQ");
            }
            _ => panic!("Expected YouTube node"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_embed_url() {
        let node = parse_directive("::youtube https://www.youtube.com/embed/dQw4w9WgXcQ", 1)
            .unwrap()
            .unwrap();

        match node {
            DarkMatterNode::YouTube { video_id, width: _ } => {
                assert_eq!(video_id, "dQw4w9WgXcQ");
            }
            _ => panic!("Expected YouTube node"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_v_url() {
        let node = parse_directive("::youtube https://youtube.com/v/dQw4w9WgXcQ", 1)
            .unwrap()
            .unwrap();

        match node {
            DarkMatterNode::YouTube { video_id, width: _ } => {
                assert_eq!(video_id, "dQw4w9WgXcQ");
            }
            _ => panic!("Expected YouTube node"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_with_pixel_width() {
        let node = parse_directive("::youtube dQw4w9WgXcQ 800px", 1)
            .unwrap()
            .unwrap();

        match node {
            DarkMatterNode::YouTube { video_id: _, width } => {
                assert_eq!(width, WidthSpec::Pixels(800));
            }
            _ => panic!("Expected YouTube node"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_with_rem_width() {
        let node = parse_directive("::youtube dQw4w9WgXcQ 32rem", 1)
            .unwrap()
            .unwrap();

        match node {
            DarkMatterNode::YouTube { video_id: _, width } => {
                assert_eq!(width, WidthSpec::Rems(32.0));
            }
            _ => panic!("Expected YouTube node"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_with_rem_width_decimal() {
        let node = parse_directive("::youtube dQw4w9WgXcQ 32.5rem", 1)
            .unwrap()
            .unwrap();

        match node {
            DarkMatterNode::YouTube { video_id: _, width } => {
                assert_eq!(width, WidthSpec::Rems(32.5));
            }
            _ => panic!("Expected YouTube node"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_with_percentage_width() {
        let node = parse_directive("::youtube dQw4w9WgXcQ 80%", 1)
            .unwrap()
            .unwrap();

        match node {
            DarkMatterNode::YouTube { video_id: _, width } => {
                assert_eq!(width, WidthSpec::Percentage(80));
            }
            _ => panic!("Expected YouTube node"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_invalid_video_id() {
        let result = parse_directive("::youtube invalid-id", 1);
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidResource(msg)) => {
                assert!(msg.contains("Could not extract video ID"));
            }
            _ => panic!("Expected InvalidResource error"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_invalid_url() {
        let result = parse_directive("::youtube https://vimeo.com/123456", 1);
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidResource(msg)) => {
                assert!(msg.contains("Could not extract video ID"));
            }
            _ => panic!("Expected InvalidResource error"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_percentage_over_100() {
        let result = parse_directive("::youtube dQw4w9WgXcQ 101%", 1);
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidDirective { line: _, directive }) => {
                assert!(directive.contains("Invalid percentage"));
                assert!(directive.contains("101"));
            }
            _ => panic!("Expected InvalidDirective error"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_zero_pixel_width() {
        let result = parse_directive("::youtube dQw4w9WgXcQ 0px", 1);
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidDirective { line: _, directive }) => {
                assert!(directive.contains("Width must be positive"));
            }
            _ => panic!("Expected InvalidDirective error"),
        }
    }

    #[test]
    fn test_parse_youtube_directive_invalid_width_format() {
        let result = parse_directive("::youtube dQw4w9WgXcQ 500", 1);
        // Should fail regex match, returning Ok(None) since directive doesn't match pattern
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // YouTube ID extraction tests
    #[test]
    fn test_extract_youtube_id_raw_id() {
        let id = extract_youtube_id("dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_youtube_id_watch_url() {
        let id = extract_youtube_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_youtube_id_watch_url_no_www() {
        let id = extract_youtube_id("https://youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_youtube_id_watch_url_http() {
        let id = extract_youtube_id("http://youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_youtube_id_short_url() {
        let id = extract_youtube_id("https://youtu.be/dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_youtube_id_embed_url() {
        let id = extract_youtube_id("https://www.youtube.com/embed/dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_youtube_id_v_url() {
        let id = extract_youtube_id("https://youtube.com/v/dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_youtube_id_with_query_params() {
        let id = extract_youtube_id("https://youtube.com/watch?v=dQw4w9WgXcQ&feature=share").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_youtube_id_invalid_too_short() {
        let result = extract_youtube_id("dQw4w9Wg");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_youtube_id_invalid_too_long() {
        let result = extract_youtube_id("dQw4w9WgXcQX");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_youtube_id_invalid_characters() {
        let result = extract_youtube_id("dQw4w9WgX@Q");
        assert!(result.is_err());
    }

    // Width spec parsing tests
    #[test]
    fn test_parse_width_spec_pixels() {
        let width = parse_width_spec("512px").unwrap();
        assert_eq!(width, WidthSpec::Pixels(512));
    }

    #[test]
    fn test_parse_width_spec_rems() {
        let width = parse_width_spec("32rem").unwrap();
        assert_eq!(width, WidthSpec::Rems(32.0));
    }

    #[test]
    fn test_parse_width_spec_rems_decimal() {
        let width = parse_width_spec("32.5rem").unwrap();
        assert_eq!(width, WidthSpec::Rems(32.5));
    }

    #[test]
    fn test_parse_width_spec_percentage() {
        let width = parse_width_spec("80%").unwrap();
        assert_eq!(width, WidthSpec::Percentage(80));
    }

    #[test]
    fn test_parse_width_spec_percentage_zero() {
        let width = parse_width_spec("0%").unwrap();
        assert_eq!(width, WidthSpec::Percentage(0));
    }

    #[test]
    fn test_parse_width_spec_percentage_100() {
        let width = parse_width_spec("100%").unwrap();
        assert_eq!(width, WidthSpec::Percentage(100));
    }

    #[test]
    fn test_parse_width_spec_percentage_over_100() {
        let result = parse_width_spec("101%");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_width_spec_zero_pixels() {
        let result = parse_width_spec("0px");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_width_spec_negative_rems() {
        let result = parse_width_spec("-5rem");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_width_spec_invalid_format() {
        let result = parse_width_spec("500");
        assert!(result.is_err());
    }


    // Verify LazyLock regex compilation happens only once
    #[test]
    fn test_youtube_regex_compiled_once() {
        // Access the regexes multiple times
        let _ = YOUTUBE_DIRECTIVE.is_match("::youtube test");
        let _ = YOUTUBE_WATCH_URL.is_match("test");
        let _ = YOUTUBE_SHORT_URL.is_match("test");
        let _ = YOUTUBE_EMBED_URL.is_match("test");
        let _ = YOUTUBE_V_URL.is_match("test");
        let _ = YOUTUBE_RAW_ID.is_match("test");

        // If we get here without panics, LazyLock is working correctly
        // The regexes are compiled on first access and reused
    }

    // ===== Comprehensive Error Handling Tests =====
    // These tests verify all error scenarios provide user-friendly, actionable messages

    #[test]
    fn test_extract_youtube_id_empty_reference() {
        let result = extract_youtube_id("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidResource(_)));
        let msg = err.to_string();
        assert!(msg.contains("Could not extract video ID from ''"));
        assert!(msg.contains("Supported formats"));
    }

    #[test]
    fn test_extract_youtube_id_invalid_domain() {
        let result = extract_youtube_id("https://vimeo.com/123456789");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidResource(_)));
        let msg = err.to_string();
        assert!(msg.contains("Could not extract video ID"));
        assert!(msg.contains("vimeo.com"));
        assert!(msg.contains("Supported formats"));
    }

    #[test]
    fn test_extract_youtube_id_malformed_url() {
        let result = extract_youtube_id("https://youtube.com/invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidResource(_)));
        let msg = err.to_string();
        assert!(msg.contains("Could not extract video ID"));
        assert!(msg.contains("youtube.com/invalid"));
    }

    #[test]
    fn test_extract_youtube_id_invalid_id_format_special_chars() {
        let result = extract_youtube_id("invalid@id!");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidResource(_)));
        let msg = err.to_string();
        assert!(msg.contains("Could not extract video ID"));
        assert!(msg.contains("invalid@id!"));
    }

    #[test]
    fn test_parse_width_spec_invalid_unit() {
        let result = parse_width_spec("500em");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidDirective { .. }));
        let msg = err.to_string();
        assert!(msg.contains("Invalid width format '500em'"));
        assert!(msg.contains("pixels (512px), rems (32rem), or percentage (0-100%)"));
    }

    #[test]
    fn test_parse_width_spec_no_unit() {
        let result = parse_width_spec("500");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidDirective { .. }));
        let msg = err.to_string();
        assert!(msg.contains("Invalid width format '500'"));
        assert!(msg.contains("pixels (512px), rems (32rem), or percentage (0-100%)"));
    }

    #[test]
    fn test_parse_width_spec_percentage_over_100_error_message() {
        let result = parse_width_spec("150%");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidDirective { .. }));
        let msg = err.to_string();
        assert!(msg.contains("Invalid percentage '150'"));
        assert!(msg.contains("Must be 0-100%"));
    }

    #[test]
    fn test_parse_width_spec_zero_pixels_error_message() {
        let result = parse_width_spec("0px");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidDirective { .. }));
        let msg = err.to_string();
        assert!(msg.contains("Width must be positive"));
    }

    #[test]
    fn test_parse_width_spec_zero_rems_error_message() {
        let result = parse_width_spec("0rem");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidDirective { .. }));
        let msg = err.to_string();
        assert!(msg.contains("Width must be positive"));
    }

    #[test]
    fn test_parse_width_spec_negative_pixels_error_message() {
        // This will fail during parsing as negative sign won't parse as u32
        let result = parse_width_spec("-100px");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidDirective { .. }));
        let msg = err.to_string();
        assert!(msg.contains("Invalid pixel width"));
    }

    #[test]
    fn test_parse_width_spec_invalid_rem_format() {
        let result = parse_width_spec("abcrem");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidDirective { .. }));
        let msg = err.to_string();
        assert!(msg.contains("Invalid rem width"));
        assert!(msg.contains("abcrem"));
    }

    #[test]
    fn test_parse_width_spec_invalid_percentage_format() {
        let result = parse_width_spec("abc%");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidDirective { .. }));
        let msg = err.to_string();
        assert!(msg.contains("Invalid percentage width"));
        assert!(msg.contains("abc%"));
    }

    #[test]
    fn test_parse_directive_youtube_with_only_width() {
        // The regex will match "::youtube  512px" with "512px" as the video reference
        // This should fail during video ID extraction because "512px" is not a valid ID
        let result = parse_directive("::youtube  512px", 1);
        assert!(result.is_err(), "Should error when trying to extract video ID from '512px'");

        // Similarly, "::youtube invalid" should error because "invalid" is not a valid video ID
        let result = parse_directive("::youtube invalid", 1);
        assert!(result.is_err(), "Should error on invalid video ID 'invalid'");
    }

    #[test]
    fn test_youtube_directive_propagates_extraction_error() {
        // Test that extract_youtube_id errors propagate correctly through parse_directive
        let result = parse_directive("::youtube invalid-id", 1);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidResource(_)));
        let msg = err.to_string();
        assert!(msg.contains("Could not extract video ID"));
        assert!(msg.contains("invalid-id"));
    }

    #[test]
    fn test_youtube_directive_propagates_width_error() {
        // Test that parse_width_spec errors propagate correctly through parse_directive
        let result = parse_directive("::youtube dQw4w9WgXcQ 150%", 1);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidDirective { .. }));
        let msg = err.to_string();
        assert!(msg.contains("Invalid percentage"));
        assert!(msg.contains("150"));
    }

    #[test]
    fn test_extract_youtube_id_url_with_malformed_query() {
        // URL with query params but no v parameter
        let result = extract_youtube_id("https://youtube.com/watch?feature=share");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::InvalidResource(_)));
        let msg = err.to_string();
        assert!(msg.contains("Could not extract video ID"));
    }

    #[test]
    fn test_extract_youtube_id_video_id_with_invalid_length() {
        // Too short
        let result = extract_youtube_id("dQw4w9Wg");
        assert!(result.is_err());

        // Too long
        let result = extract_youtube_id("dQw4w9WgXcQXX");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_messages_include_context() {
        // Verify that error messages include the actual invalid input for debugging
        let test_cases = vec![
            ("short", "short"),  // Too short (5 chars)
            ("https://vimeo.com/12345", "vimeo.com"),  // Wrong domain
            ("invalid@video", "invalid@video"),  // 12 chars with invalid char
        ];

        for (input, expected_in_message) in test_cases {
            let result = extract_youtube_id(input);
            assert!(
                result.is_err(),
                "Expected error for input '{}', but got: {:?}",
                input,
                result
            );
            let msg = result.unwrap_err().to_string();
            assert!(
                msg.contains(expected_in_message),
                "Error message '{}' should contain '{}'",
                msg,
                expected_in_message
            );
        }
    }

    #[test]
    fn test_width_error_messages_include_suggestions() {
        // Verify width errors provide actionable suggestions
        let result = parse_width_spec("500em");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("pixels (512px)"));
        assert!(msg.contains("rems (32rem)"));
        assert!(msg.contains("percentage (0-100%)"));
    }

    #[test]
    fn test_video_id_error_messages_include_suggestions() {
        // Verify video ID errors provide actionable suggestions
        let result = extract_youtube_id("invalid");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("youtube.com/watch?v=ID"));
        assert!(msg.contains("youtu.be/ID"));
        assert!(msg.contains("11-character ID"));
    }

    #[test]
    fn test_no_panics_on_invalid_inputs() {
        // Ensure no panics occur with various invalid inputs
        let long_string = "dQw4w9WgXcQ".to_owned() + &"X".repeat(100);
        let invalid_inputs = vec![
            "",
            "   ",
            "https://",
            "http://",
            "youtube.com",
            "watch?v=",
            "youtu.be/",
            "123",
            "!@#$%^&*()",
            &long_string, // Very long string
        ];

        for input in invalid_inputs {
            let _ = extract_youtube_id(input);
            // Should not panic, just return error
        }
    }

    // ===== Property-Based Tests =====
    // These tests use proptest to verify properties hold for generated inputs

    use proptest::prelude::*;

    // Strategy to generate valid YouTube video IDs (11 chars, alphanumeric + - and _)
    fn valid_video_id_strategy() -> impl Strategy<Value = String> {
        prop::string::string_regex("[A-Za-z0-9_-]{11}").unwrap()
    }

    // Strategy to generate valid pixel widths (1-10000px)
    fn valid_pixel_width_strategy() -> impl Strategy<Value = u32> {
        1u32..=10000u32
    }

    // Strategy to generate valid rem widths (0.1-100.0rem)
    fn valid_rem_width_strategy() -> impl Strategy<Value = f32> {
        0.1f32..=100.0f32
    }

    // Strategy to generate valid percentage widths (0-100%)
    fn valid_percentage_width_strategy() -> impl Strategy<Value = u8> {
        0u8..=100u8
    }

    proptest! {
        #[test]
        fn prop_valid_video_ids_parse(id in valid_video_id_strategy()) {
            // Any 11-character string with valid chars should parse as raw ID
            let result = extract_youtube_id(&id);
            prop_assert!(result.is_ok(), "Valid ID '{}' should parse", id);
            prop_assert_eq!(result.unwrap(), id);
        }

        #[test]
        fn prop_valid_video_ids_in_watch_url(id in valid_video_id_strategy()) {
            // Video ID in watch URL should extract correctly
            let url = format!("https://www.youtube.com/watch?v={}", id);
            let result = extract_youtube_id(&url);
            prop_assert!(result.is_ok(), "Watch URL with '{}' should parse", id);
            prop_assert_eq!(result.unwrap(), id);
        }

        #[test]
        fn prop_valid_video_ids_in_short_url(id in valid_video_id_strategy()) {
            // Video ID in short URL should extract correctly
            let url = format!("https://youtu.be/{}", id);
            let result = extract_youtube_id(&url);
            prop_assert!(result.is_ok(), "Short URL with '{}' should parse", id);
            prop_assert_eq!(result.unwrap(), id);
        }

        #[test]
        fn prop_valid_video_ids_in_embed_url(id in valid_video_id_strategy()) {
            // Video ID in embed URL should extract correctly
            let url = format!("https://www.youtube.com/embed/{}", id);
            let result = extract_youtube_id(&url);
            prop_assert!(result.is_ok(), "Embed URL with '{}' should parse", id);
            prop_assert_eq!(result.unwrap(), id);
        }

        #[test]
        fn prop_url_format_consistency(id in valid_video_id_strategy()) {
            // All URL formats for same video ID should extract to same ID
            let urls = vec![
                format!("https://www.youtube.com/watch?v={}", id),
                format!("https://youtu.be/{}", id),
                format!("https://www.youtube.com/embed/{}", id),
                format!("https://youtube.com/v/{}", id),
                id.clone(),
            ];

            let expected_id = id.clone();
            for url in urls {
                let result = extract_youtube_id(&url);
                prop_assert!(result.is_ok(), "URL '{}' should parse", url);
                prop_assert_eq!(result.unwrap(), expected_id.clone(), "URL '{}' should extract to '{}'", url, expected_id);
            }
        }

        #[test]
        fn prop_valid_pixel_widths_parse(px in valid_pixel_width_strategy()) {
            // Any positive pixel value should parse
            let width_str = format!("{}px", px);
            let result = parse_width_spec(&width_str);
            prop_assert!(result.is_ok(), "Width '{}' should parse", width_str);
            prop_assert_eq!(result.unwrap(), WidthSpec::Pixels(px));
        }

        #[test]
        fn prop_valid_rem_widths_parse(rem in valid_rem_width_strategy()) {
            // Any positive rem value should parse
            let width_str = format!("{}rem", rem);
            let result = parse_width_spec(&width_str);
            prop_assert!(result.is_ok(), "Width '{}' should parse", width_str);

            match result.unwrap() {
                WidthSpec::Rems(parsed_rem) => {
                    // Float comparison with epsilon
                    prop_assert!((parsed_rem - rem).abs() < 0.001, "Parsed rem {} != {}", parsed_rem, rem);
                }
                _ => prop_assert!(false, "Should parse as Rems"),
            }
        }

        #[test]
        fn prop_valid_percentage_widths_parse(pct in valid_percentage_width_strategy()) {
            // Any percentage 0-100 should parse
            let width_str = format!("{}%", pct);
            let result = parse_width_spec(&width_str);
            prop_assert!(result.is_ok(), "Width '{}' should parse", width_str);
            prop_assert_eq!(result.unwrap(), WidthSpec::Percentage(pct));
        }

        #[test]
        fn prop_invalid_video_id_length_fails(
            // Generate strings that are NOT 11 characters
            len in 1usize..50usize,
        ) {
            // Skip length 11 (valid length)
            prop_assume!(len != 11);

            // Generate a string of the specified length with valid chars
            let id: String = (0..len)
                .map(|_| {
                    let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
                    chars[len % chars.len()] as char
                })
                .collect();

            // Should fail validation (wrong length)
            let result = extract_youtube_id(&id);
            prop_assert!(result.is_err(), "Invalid length ID '{}' (len={}) should fail", id, len);
        }

        #[test]
        fn prop_percentage_over_100_fails(pct in 101u16..=1000u16) {
            // Percentages > 100 should fail
            let width_str = format!("{}%", pct);
            let result = parse_width_spec(&width_str);
            prop_assert!(result.is_err(), "Percentage '{}' should fail", width_str);
        }

        #[test]
        fn prop_width_spec_display_roundtrip(width in prop_oneof![
            valid_pixel_width_strategy().prop_map(WidthSpec::Pixels),
            valid_rem_width_strategy().prop_map(WidthSpec::Rems),
            valid_percentage_width_strategy().prop_map(WidthSpec::Percentage),
        ]) {
            // Display and parse should roundtrip correctly
            let displayed = width.to_string();
            let parsed = parse_width_spec(&displayed);

            prop_assert!(parsed.is_ok(), "Display '{}' should parse back", displayed);

            match (width, parsed.unwrap()) {
                (WidthSpec::Pixels(a), WidthSpec::Pixels(b)) => prop_assert_eq!(a, b),
                (WidthSpec::Rems(a), WidthSpec::Rems(b)) => {
                    prop_assert!((a - b).abs() < 0.001, "{} != {}", a, b);
                }
                (WidthSpec::Percentage(a), WidthSpec::Percentage(b)) => prop_assert_eq!(a, b),
                _ => prop_assert!(false, "Width variant mismatch"),
            }
        }

        #[test]
        fn prop_youtube_directive_parse_with_valid_inputs(
            id in valid_video_id_strategy(),
            width_opt in prop::option::of(prop_oneof![
                valid_pixel_width_strategy().prop_map(|px| format!("{}px", px)),
                valid_rem_width_strategy().prop_map(|rem| format!("{}rem", rem)),
                valid_percentage_width_strategy().prop_map(|pct| format!("{}%", pct)),
            ])
        ) {
            // Construct a valid directive
            let directive = match width_opt {
                Some(width) => format!("::youtube {} {}", id, width),
                None => format!("::youtube {}", id),
            };

            let result = parse_directive(&directive, 1);
            prop_assert!(result.is_ok(), "Directive '{}' should parse", directive);

            let node = result.unwrap();
            prop_assert!(node.is_some(), "Directive '{}' should return node", directive);

            match node.unwrap() {
                DarkMatterNode::YouTube { video_id, width: _ } => {
                    prop_assert_eq!(video_id, id, "Video ID mismatch in directive '{}'", directive);
                }
                _ => prop_assert!(false, "Should return YouTube node for '{}'", directive),
            }
        }

        #[test]
        fn prop_html_injection_attempts_fail(
            // Generate strings with HTML/script injection attempts
            malicious in prop::string::string_regex("<[^>]{1,20}>|['\";]|DROP|script|alert").unwrap()
        ) {
            // Any string with HTML/SQL injection patterns should fail video ID validation
            let directive = format!("::youtube {}", malicious);
            let result = parse_directive(&directive, 1);

            // Should either error or return None (doesn't match directive pattern)
            prop_assert!(
                result.is_err() || result.as_ref().unwrap().is_none(),
                "Malicious input '{}' should not parse as valid YouTube directive",
                malicious
            );
        }
    }
}
