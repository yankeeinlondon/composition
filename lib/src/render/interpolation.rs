use crate::error::RenderError;
use crate::types::{DarkMatterNode, Frontmatter};
use chrono::{Datelike, Local, Utc, Weekday};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::instrument;

/// Regex pattern for matching {{variable}} interpolation syntax
static INTERPOLATION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*)\}\}").expect("Invalid regex pattern")
});

/// Generate utility variables that are always available
///
/// Returns a HashMap of utility variable names to their JSON values.
/// These variables provide date/time information and can be overridden
/// by custom frontmatter variables.
fn generate_utility_variables() -> HashMap<String, serde_json::Value> {
    use serde_json::json;

    let now_local = Local::now();
    let now_utc = Utc::now();
    let today = now_local.date_naive();
    let yesterday = today - chrono::Days::new(1);
    let tomorrow = today + chrono::Days::new(1);

    // Calculate season (Northern Hemisphere)
    let month = now_local.month();
    let season = match month {
        3..=5 => "Spring",
        6..=8 => "Summer",
        9..=11 => "Fall",
        12 | 1 | 2 => "Winter",
        _ => "Unknown",
    };

    // Get day of week
    let weekday = now_local.weekday();
    let day_of_week = match weekday {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    };

    let day_of_week_abbr = match weekday {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    };

    // Get month names
    let month_name = match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    };

    let month_abbr = match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "Unk",
    };

    // Check if today is last day of month
    let next_day = today + chrono::Days::new(1);
    let is_last_day = next_day.month() != today.month();

    // Get timezone - use offset format since IANA names aren't available
    let tz_offset = now_local.offset();
    let timezone = format!("{}", tz_offset);

    let mut vars = HashMap::new();

    // Date variables
    vars.insert("today".to_string(), json!(today.format("%Y-%m-%d").to_string()));
    vars.insert("yesterday".to_string(), json!(yesterday.format("%Y-%m-%d").to_string()));
    vars.insert("tomorrow".to_string(), json!(tomorrow.format("%Y-%m-%d").to_string()));
    vars.insert("year".to_string(), json!(now_local.year().to_string()));
    vars.insert("month".to_string(), json!(month_name));
    vars.insert("month_abbr".to_string(), json!(month_abbr));
    vars.insert("month_numeric".to_string(), json!(format!("{:02}", month)));
    vars.insert("day".to_string(), json!(format!("{:02}", now_local.day())));

    // Day of week
    vars.insert("day_of_week".to_string(), json!(day_of_week));
    vars.insert("day_of_week_abbr".to_string(), json!(day_of_week_abbr));

    // Season
    vars.insert("season".to_string(), json!(season));

    // Week number (ISO week)
    vars.insert("week_number".to_string(), json!(now_local.iso_week().week().to_string()));

    // Time variables
    vars.insert("timestamp".to_string(), json!(now_utc.timestamp().to_string()));
    vars.insert("iso_timestamp".to_string(), json!(now_utc.to_rfc3339()));
    vars.insert("now".to_string(), json!(now_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string()));
    vars.insert("now_utc".to_string(), json!(now_utc.to_rfc3339()));
    vars.insert("now_local".to_string(), json!(now_local.to_rfc3339()));

    // Timezone
    vars.insert("timezone".to_string(), json!(timezone));

    // Last day of month flag
    vars.insert("last_day_in_month".to_string(), json!(is_last_day));

    vars
}

/// Process frontmatter interpolation in content
///
/// This function:
/// 1. Generates utility variables (dates, times, etc.)
/// 2. Merges with custom frontmatter (custom overrides utilities)
/// 3. Replaces {{variable}} patterns with values
/// 4. Applies text replacements defined in frontmatter.replace
/// 5. Returns the processed content
#[instrument(skip(frontmatter))]
pub fn process_interpolation(content: &str, frontmatter: &Frontmatter) -> Result<String, RenderError> {
    let mut result = content.to_string();

    // Generate utility variables
    let utilities = generate_utility_variables();

    // Merge: custom frontmatter overrides utilities
    let all_vars: HashMap<String, serde_json::Value> = utilities
        .into_iter()
        .chain(frontmatter.custom.clone())
        .collect();

    // Process {{variable}} patterns
    for cap in INTERPOLATION_REGEX.captures_iter(content) {
        let var_name = &cap[1];
        if let Some(value) = all_vars.get(var_name) {
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

    // Utility variable tests

    #[test]
    fn test_utility_today() {
        let fm = Frontmatter::default();
        let content = "Date: {{today}}";
        let result = process_interpolation(content, &fm).unwrap();

        // Verify format is YYYY-MM-DD
        assert!(result.starts_with("Date: "));
        let date_part = result.strip_prefix("Date: ").unwrap();
        assert_eq!(date_part.len(), 10);
        assert_eq!(&date_part[4..5], "-");
        assert_eq!(&date_part[7..8], "-");
    }

    #[test]
    fn test_utility_yesterday_tomorrow() {
        let fm = Frontmatter::default();
        let content = "{{yesterday}} {{today}} {{tomorrow}}";
        let result = process_interpolation(content, &fm).unwrap();

        // All should be in YYYY-MM-DD format
        let parts: Vec<&str> = result.split_whitespace().collect();
        assert_eq!(parts.len(), 3);
        for part in parts {
            assert_eq!(part.len(), 10);
            assert_eq!(&part[4..5], "-");
            assert_eq!(&part[7..8], "-");
        }
    }

    #[test]
    fn test_utility_year() {
        let fm = Frontmatter::default();
        let content = "Year: {{year}}";
        let result = process_interpolation(content, &fm).unwrap();

        // Verify it's a 4-digit year
        let year = result.strip_prefix("Year: ").unwrap();
        assert_eq!(year.len(), 4);
        assert!(year.parse::<u32>().unwrap() >= 2024);
    }

    #[test]
    fn test_utility_month() {
        let fm = Frontmatter::default();
        let content = "{{month}} {{month_abbr}} {{month_numeric}}";
        let result = process_interpolation(content, &fm).unwrap();

        // Verify month name is present
        let valid_months = [
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December"
        ];
        assert!(valid_months.iter().any(|m| result.contains(m)));

        // Verify abbreviated month
        let valid_abbrs = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        assert!(valid_abbrs.iter().any(|a| result.contains(a)));
    }

    #[test]
    fn test_utility_day_of_week() {
        let fm = Frontmatter::default();
        let content = "{{day_of_week}} {{day_of_week_abbr}}";
        let result = process_interpolation(content, &fm).unwrap();

        // Verify day name
        let valid_days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
        assert!(valid_days.iter().any(|d| result.contains(d)));

        // Verify abbreviated day
        let valid_abbrs = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        assert!(valid_abbrs.iter().any(|a| result.contains(a)));
    }

    #[test]
    fn test_utility_season() {
        let fm = Frontmatter::default();
        let content = "Season: {{season}}";
        let result = process_interpolation(content, &fm).unwrap();

        // Verify it's a valid season
        let valid_seasons = ["Spring", "Summer", "Fall", "Winter"];
        assert!(valid_seasons.iter().any(|s| result.contains(s)));
    }

    #[test]
    fn test_utility_week_number() {
        let fm = Frontmatter::default();
        let content = "Week: {{week_number}}";
        let result = process_interpolation(content, &fm).unwrap();

        // Verify it's a number between 1 and 53
        let week = result.strip_prefix("Week: ").unwrap();
        let week_num: u32 = week.parse().unwrap();
        assert!((1..=53).contains(&week_num));
    }

    #[test]
    fn test_utility_timestamp() {
        let fm = Frontmatter::default();
        let content = "Timestamp: {{timestamp}}";
        let result = process_interpolation(content, &fm).unwrap();

        // Verify it's a valid Unix timestamp
        let ts = result.strip_prefix("Timestamp: ").unwrap();
        let timestamp: i64 = ts.parse().unwrap();
        assert!(timestamp > 1700000000); // After 2023
    }

    #[test]
    fn test_utility_iso_timestamp() {
        let fm = Frontmatter::default();
        let content = "ISO: {{iso_timestamp}}";
        let result = process_interpolation(content, &fm).unwrap();

        // Verify it contains expected ISO format characters
        assert!(result.contains("T"));
        assert!(result.contains("-") || result.contains("+"));
    }

    #[test]
    fn test_utility_now_utc_and_local() {
        let fm = Frontmatter::default();
        let content = "UTC: {{now_utc}} Local: {{now_local}}";
        let result = process_interpolation(content, &fm).unwrap();

        // Both should contain ISO format indicators
        assert!(result.contains("UTC:"));
        assert!(result.contains("Local:"));
        assert!(result.contains("T"));
    }

    #[test]
    fn test_utility_timezone() {
        let fm = Frontmatter::default();
        let content = "TZ: {{timezone}}";
        let result = process_interpolation(content, &fm).unwrap();

        // Verify timezone format (contains + or -)
        assert!(result.contains("TZ:"));
        assert!(result.contains("+") || result.contains("-"));
    }

    #[test]
    fn test_utility_last_day_in_month() {
        let fm = Frontmatter::default();
        let content = "Last day: {{last_day_in_month}}";
        let result = process_interpolation(content, &fm).unwrap();

        // Should be true or false
        assert!(result.contains("true") || result.contains("false"));
    }

    #[test]
    fn test_custom_overrides_utility() {
        let mut fm = Frontmatter::default();
        // Override the "today" utility variable
        fm.custom.insert(
            "today".to_string(),
            serde_json::Value::String("CUSTOM_DATE".to_string()),
        );

        let content = "Date: {{today}}";
        let result = process_interpolation(content, &fm).unwrap();

        // Custom value should override utility
        assert_eq!(result, "Date: CUSTOM_DATE");
    }

    #[test]
    fn test_utility_and_custom_mixed() {
        let mut fm = Frontmatter::default();
        fm.custom.insert(
            "author".to_string(),
            serde_json::Value::String("Alice".to_string()),
        );

        let content = "Written by {{author}} on {{today}}";
        let result = process_interpolation(content, &fm).unwrap();

        // Should have custom author and utility today
        assert!(result.starts_with("Written by Alice on "));
        assert!(result.contains("-")); // Date should have dashes
    }

    #[test]
    fn test_season_calculation() {
        // Directly test the utility generation logic for season
        let utilities = generate_utility_variables();
        let season = utilities.get("season").unwrap().as_str().unwrap();

        // Should be one of the four seasons
        assert!(["Spring", "Summer", "Fall", "Winter"].contains(&season));
    }
}
