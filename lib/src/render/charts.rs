use crate::types::{ChartData, DataPoint};
use crate::error::RenderError;

/// Render a bar chart to SVG
pub fn render_bar_chart(data: &ChartData, width: u32, height: u32) -> Result<String, RenderError> {
    let points = extract_data_points(data)?;

    if points.is_empty() {
        return Ok(String::from("<svg></svg>"));
    }

    let max_value = points.iter()
        .map(|p| p.value)
        .fold(f64::NEG_INFINITY, f64::max);

    let bar_width = (width as f64 * 0.8) / points.len() as f64;
    let margin = width as f64 * 0.1;
    let chart_height = height as f64 * 0.8;
    let margin_top = height as f64 * 0.1;

    let mut svg = format!(
        r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg" class="composition-bar-chart">"#,
        width, height
    );

    // Draw bars
    for (i, point) in points.iter().enumerate() {
        let bar_height = (point.value / max_value) * chart_height;
        let x = margin + (i as f64 * bar_width);
        let y = margin_top + (chart_height - bar_height);

        svg.push_str(&format!(
            r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#3b82f6" class="bar"/>"##,
            x, y, bar_width * 0.8, bar_height
        ));

        // Add label
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" text-anchor="middle" font-size="12" class="label">{}</text>"#,
            x + (bar_width * 0.4),
            height - 5,
            point.label
        ));
    }

    svg.push_str("</svg>");
    Ok(svg)
}

/// Render a line chart to SVG
pub fn render_line_chart(data: &ChartData, width: u32, height: u32) -> Result<String, RenderError> {
    let points = extract_data_points(data)?;

    if points.is_empty() {
        return Ok(String::from("<svg></svg>"));
    }

    let max_value = points.iter()
        .map(|p| p.value)
        .fold(f64::NEG_INFINITY, f64::max);

    let margin = 40.0;
    let chart_width = width as f64 - (2.0 * margin);
    let chart_height = height as f64 - (2.0 * margin);

    let mut svg = format!(
        r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg" class="composition-line-chart">"#,
        width, height
    );

    // Build path data
    let mut path_data = String::from("M");
    for (i, point) in points.iter().enumerate() {
        let x = margin + (i as f64 * chart_width / (points.len() - 1).max(1) as f64);
        let y = margin + (chart_height - (point.value / max_value) * chart_height);

        if i > 0 {
            path_data.push_str(&format!(" L{},{}", x, y));
        } else {
            path_data.push_str(&format!("{},{}", x, y));
        }
    }

    // Draw line
    svg.push_str(&format!(
        r##"<path d="{}" fill="none" stroke="#3b82f6" stroke-width="2" class="line"/>"##,
        path_data
    ));

    // Draw points
    for (i, point) in points.iter().enumerate() {
        let x = margin + (i as f64 * chart_width / (points.len() - 1).max(1) as f64);
        let y = margin + (chart_height - (point.value / max_value) * chart_height);

        svg.push_str(&format!(
            r##"<circle cx="{}" cy="{}" r="4" fill="#3b82f6" class="point"/>"##,
            x, y
        ));
    }

    svg.push_str("</svg>");
    Ok(svg)
}

/// Render a pie chart to SVG
pub fn render_pie_chart(data: &ChartData, width: u32, height: u32) -> Result<String, RenderError> {
    let points = extract_data_points(data)?;

    if points.is_empty() {
        return Ok(String::from("<svg></svg>"));
    }

    let total: f64 = points.iter().map(|p| p.value).sum();
    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;
    let radius = (width.min(height) as f64 / 2.0) * 0.8;

    let mut svg = format!(
        r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg" class="composition-pie-chart">"#,
        width, height
    );

    let colors = ["#3b82f6", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6", "#ec4899"];
    let mut current_angle = -90.0; // Start at top

    for (i, point) in points.iter().enumerate() {
        let slice_angle = (point.value / total) * 360.0;
        let end_angle = current_angle + slice_angle;

        let start_rad = current_angle.to_radians();
        let end_rad = end_angle.to_radians();

        let x1 = center_x + radius * start_rad.cos();
        let y1 = center_y + radius * start_rad.sin();
        let x2 = center_x + radius * end_rad.cos();
        let y2 = center_y + radius * end_rad.sin();

        let large_arc = if slice_angle > 180.0 { 1 } else { 0 };

        svg.push_str(&format!(
            r#"<path d="M{},{} L{},{} A{},{} 0 {},{} {},{} Z" fill="{}" class="slice"/>"#,
            center_x, center_y,
            x1, y1,
            radius, radius,
            large_arc,
            1,
            x2, y2,
            colors[i % colors.len()]
        ));

        current_angle = end_angle;
    }

    svg.push_str("</svg>");
    Ok(svg)
}

/// Render an area chart to SVG
pub fn render_area_chart(data: &ChartData, width: u32, height: u32) -> Result<String, RenderError> {
    let points = extract_data_points(data)?;

    if points.is_empty() {
        return Ok(String::from("<svg></svg>"));
    }

    let max_value = points.iter()
        .map(|p| p.value)
        .fold(f64::NEG_INFINITY, f64::max);

    let margin = 40.0;
    let chart_width = width as f64 - (2.0 * margin);
    let chart_height = height as f64 - (2.0 * margin);

    let mut svg = format!(
        r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg" class="composition-area-chart">"#,
        width, height
    );

    // Build path data for area
    let mut path_data = String::from("M");
    let baseline_y = margin + chart_height;

    // Start at baseline
    path_data.push_str(&format!("{},{}", margin, baseline_y));

    // Draw top line
    for (i, point) in points.iter().enumerate() {
        let x = margin + (i as f64 * chart_width / (points.len() - 1).max(1) as f64);
        let y = margin + (chart_height - (point.value / max_value) * chart_height);
        path_data.push_str(&format!(" L{},{}", x, y));
    }

    // Return to baseline
    let last_x = margin + chart_width;
    path_data.push_str(&format!(" L{},{} Z", last_x, baseline_y));

    // Draw filled area
    svg.push_str(&format!(
        r##"<path d="{}" fill="#3b82f6" fill-opacity="0.3" stroke="#3b82f6" stroke-width="2" class="area"/>"##,
        path_data
    ));

    svg.push_str("</svg>");
    Ok(svg)
}

/// Render a bubble chart to SVG
pub fn render_bubble_chart(data: &ChartData, width: u32, height: u32) -> Result<String, RenderError> {
    let points = extract_data_points(data)?;

    if points.is_empty() {
        return Ok(String::from("<svg></svg>"));
    }

    let max_value = points.iter()
        .map(|p| p.value)
        .fold(f64::NEG_INFINITY, f64::max);

    let margin = 40.0;
    let chart_width = width as f64 - (2.0 * margin);
    let chart_height = height as f64 - (2.0 * margin);

    let mut svg = format!(
        r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg" class="composition-bubble-chart">"#,
        width, height
    );

    let colors = ["#3b82f6", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6", "#ec4899"];

    // Draw bubbles
    for (i, point) in points.iter().enumerate() {
        let x = margin + (i as f64 * chart_width / (points.len() - 1).max(1) as f64);
        let y = margin + (chart_height - (point.value / max_value) * chart_height);
        let radius = (point.value / max_value) * 30.0 + 10.0;

        svg.push_str(&format!(
            r#"<circle cx="{}" cy="{}" r="{}" fill="{}" fill-opacity="0.6" stroke="{}" stroke-width="2" class="bubble"/>"#,
            x, y, radius, colors[i % colors.len()], colors[i % colors.len()]
        ));
    }

    svg.push_str("</svg>");
    Ok(svg)
}

/// Extract data points from ChartData
fn extract_data_points(data: &ChartData) -> Result<Vec<DataPoint>, RenderError> {
    match data {
        ChartData::Inline(points) => Ok(points.clone()),
        ChartData::External(_resource) => {
            // TODO: In a full implementation, this would read and parse the external resource
            // For now, return an error
            Err(RenderError::ChartError("External chart data not yet supported".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    fn sample_data() -> Vec<DataPoint> {
        vec![
            DataPoint {
                label: "A".to_string(),
                value: 10.0,
                metadata: None,
            },
            DataPoint {
                label: "B".to_string(),
                value: 20.0,
                metadata: None,
            },
            DataPoint {
                label: "C".to_string(),
                value: 15.0,
                metadata: None,
            },
        ]
    }

    #[test]
    fn test_render_bar_chart() {
        let data = ChartData::Inline(sample_data());
        let result = render_bar_chart(&data, 800, 400).unwrap();

        assert!(result.contains("<svg"));
        assert!(result.contains("composition-bar-chart"));
        assert!(result.contains("<rect"));
    }

    #[test]
    fn test_render_line_chart() {
        let data = ChartData::Inline(sample_data());
        let result = render_line_chart(&data, 800, 400).unwrap();

        assert!(result.contains("<svg"));
        assert!(result.contains("composition-line-chart"));
        assert!(result.contains("<path"));
        assert!(result.contains("<circle"));
    }

    #[test]
    fn test_render_pie_chart() {
        let data = ChartData::Inline(sample_data());
        let result = render_pie_chart(&data, 400, 400).unwrap();

        assert!(result.contains("<svg"));
        assert!(result.contains("composition-pie-chart"));
        assert!(result.contains("<path"));
    }

    #[test]
    fn test_render_area_chart() {
        let data = ChartData::Inline(sample_data());
        let result = render_area_chart(&data, 800, 400).unwrap();

        assert!(result.contains("<svg"));
        assert!(result.contains("composition-area-chart"));
        assert!(result.contains("<path"));
    }

    #[test]
    fn test_render_bubble_chart() {
        let data = ChartData::Inline(sample_data());
        let result = render_bubble_chart(&data, 800, 400).unwrap();

        assert!(result.contains("<svg"));
        assert!(result.contains("composition-bubble-chart"));
        assert!(result.contains("<circle"));
    }

    #[test]
    fn test_empty_data() {
        let data = ChartData::Inline(vec![]);
        let result = render_bar_chart(&data, 800, 400).unwrap();

        assert!(result.contains("<svg></svg>"));
    }
}
