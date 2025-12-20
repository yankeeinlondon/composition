use crate::error::RenderError;
use crate::types::{Resource, ResourceSource, TableSource};
use std::fs;
use tracing::instrument;

/// Render a table to HTML
///
/// Supports both inline table data and external CSV files
#[instrument]
pub fn render_table(source: &TableSource, has_heading: bool) -> Result<String, RenderError> {
    match source {
        TableSource::Inline(rows) => render_inline_table(rows, has_heading),
        TableSource::External(resource) => {
            let csv_data = load_csv(resource)?;
            render_csv_table(&csv_data, has_heading)
        }
    }
}

/// Render inline table data to HTML
fn render_inline_table(rows: &[Vec<String>], has_heading: bool) -> Result<String, RenderError> {
    if rows.is_empty() {
        return Ok(String::from("<table></table>"));
    }

    let mut html = String::from("<table>\n");

    // Handle heading row if specified
    if has_heading && !rows.is_empty() {
        html.push_str("  <thead>\n    <tr>\n");
        for cell in &rows[0] {
            html.push_str(&format!("      <th>{}</th>\n", escape_html(cell)));
        }
        html.push_str("    </tr>\n  </thead>\n");

        // Render remaining rows as body
        if rows.len() > 1 {
            html.push_str("  <tbody>\n");
            for row in &rows[1..] {
                html.push_str("    <tr>\n");
                for cell in row {
                    html.push_str(&format!("      <td>{}</td>\n", escape_html(cell)));
                }
                html.push_str("    </tr>\n");
            }
            html.push_str("  </tbody>\n");
        }
    } else {
        // All rows are body rows
        html.push_str("  <tbody>\n");
        for row in rows {
            html.push_str("    <tr>\n");
            for cell in row {
                html.push_str(&format!("      <td>{}</td>\n", escape_html(cell)));
            }
            html.push_str("    </tr>\n");
        }
        html.push_str("  </tbody>\n");
    }

    html.push_str("</table>");
    Ok(html)
}

/// Load CSV data from a resource
fn load_csv(resource: &Resource) -> Result<Vec<Vec<String>>, RenderError> {
    let content = match &resource.source {
        ResourceSource::Local(path) => {
            fs::read_to_string(path)
                .map_err(|e| RenderError::ResourceNotFound(
                    path.display().to_string(),
                    e.to_string()
                ))?
        }
        ResourceSource::Remote(url) => {
            // For remote CSV, we'd need to fetch it
            // Using blocking reqwest for simplicity
            reqwest::blocking::get(url.clone())
                .map_err(|e| RenderError::RemoteFetchError(
                    url.to_string(),
                    e.to_string()
                ))?
                .text()
                .map_err(|e| RenderError::RemoteFetchError(
                    url.to_string(),
                    e.to_string()
                ))?
        }
    };

    parse_csv(&content)
}

/// Parse CSV content into rows
fn parse_csv(content: &str) -> Result<Vec<Vec<String>>, RenderError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false) // Don't treat first row as headers
        .from_reader(content.as_bytes());
    let mut rows = Vec::new();

    for result in reader.records() {
        let record = result.map_err(|e| RenderError::CsvError(e.to_string()))?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        rows.push(row);
    }

    Ok(rows)
}

/// Render CSV data to HTML table
fn render_csv_table(rows: &[Vec<String>], has_heading: bool) -> Result<String, RenderError> {
    // CSV rendering is the same as inline table rendering
    render_inline_table(rows, has_heading)
}

/// Escape HTML special characters
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_inline_table_simple() {
        let rows = vec![
            vec!["A".to_string(), "B".to_string()],
            vec!["1".to_string(), "2".to_string()],
        ];

        let source = TableSource::Inline(rows);
        let html = render_table(&source, false).unwrap();

        assert!(html.contains("<table>"));
        assert!(html.contains("<td>A</td>"));
        assert!(html.contains("<td>1</td>"));
        assert!(html.contains("</table>"));
    }

    #[test]
    fn test_render_inline_table_with_heading() {
        let rows = vec![
            vec!["Name".to_string(), "Age".to_string()],
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];

        let source = TableSource::Inline(rows);
        let html = render_table(&source, true).unwrap();

        assert!(html.contains("<thead>"));
        assert!(html.contains("<th>Name</th>"));
        assert!(html.contains("<th>Age</th>"));
        assert!(html.contains("<tbody>"));
        assert!(html.contains("<td>Alice</td>"));
    }

    #[test]
    fn test_render_empty_table() {
        let rows: Vec<Vec<String>> = vec![];
        let source = TableSource::Inline(rows);
        let html = render_table(&source, false).unwrap();

        assert_eq!(html, "<table></table>");
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("Hello"), "Hello");
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("A & B"), "A &amp; B");
        assert_eq!(escape_html("\"quote\""), "&quot;quote&quot;");
    }

    #[test]
    fn test_parse_csv_simple() {
        let csv = "a,b,c\n1,2,3";
        let rows = parse_csv(csv).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["a", "b", "c"]);
        assert_eq!(rows[1], vec!["1", "2", "3"]);
    }

    #[test]
    fn test_parse_csv_with_quotes() {
        let csv = r#"name,description
"Alice","Has a ""quote"""#;
        let rows = parse_csv(csv).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["name", "description"]);
        assert_eq!(rows[1], vec!["Alice", r#"Has a "quote""#]);
    }

    #[test]
    fn test_parse_csv_empty() {
        let csv = "";
        let rows = parse_csv(csv).unwrap();
        assert_eq!(rows.len(), 0);
    }
}
