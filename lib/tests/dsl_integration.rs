use lib::types::{ChartData, DataPoint, DarkMatterNode, Breakpoint};
use lib::render::{
    render_bar_chart, render_line_chart, render_pie_chart, render_area_chart, render_bubble_chart,
    render_popover, render_inline_popover,
    render_disclosure, render_disclosure_open,
    render_columns, generate_columns_styles,
};
use std::collections::HashMap;

// Chart Tests

#[test]
fn test_bar_chart_rendering() {
    let data = ChartData::Inline(vec![
        DataPoint {
            label: "Q1".to_string(),
            value: 100.0,
            metadata: None,
        },
        DataPoint {
            label: "Q2".to_string(),
            value: 150.0,
            metadata: None,
        },
        DataPoint {
            label: "Q3".to_string(),
            value: 120.0,
            metadata: None,
        },
    ]);

    let result = render_bar_chart(&data, 800, 400).unwrap();

    assert!(result.contains("<svg"));
    assert!(result.contains("viewBox=\"0 0 800 400\""));
    assert!(result.contains("composition-bar-chart"));
    assert!(result.contains("<rect"));
    assert!(result.contains("Q1"));
    assert!(result.contains("Q2"));
    assert!(result.contains("Q3"));
}

#[test]
fn test_line_chart_rendering() {
    let data = ChartData::Inline(vec![
        DataPoint {
            label: "Jan".to_string(),
            value: 50.0,
            metadata: None,
        },
        DataPoint {
            label: "Feb".to_string(),
            value: 75.0,
            metadata: None,
        },
        DataPoint {
            label: "Mar".to_string(),
            value: 60.0,
            metadata: None,
        },
    ]);

    let result = render_line_chart(&data, 800, 400).unwrap();

    assert!(result.contains("<svg"));
    assert!(result.contains("composition-line-chart"));
    assert!(result.contains("<path"));
    assert!(result.contains("<circle"));
}

#[test]
fn test_pie_chart_rendering() {
    let data = ChartData::Inline(vec![
        DataPoint {
            label: "A".to_string(),
            value: 30.0,
            metadata: None,
        },
        DataPoint {
            label: "B".to_string(),
            value: 50.0,
            metadata: None,
        },
        DataPoint {
            label: "C".to_string(),
            value: 20.0,
            metadata: None,
        },
    ]);

    let result = render_pie_chart(&data, 400, 400).unwrap();

    assert!(result.contains("<svg"));
    assert!(result.contains("composition-pie-chart"));
    assert!(result.contains("<path"));
    // Should have 3 slices
    assert_eq!(result.matches("<path").count(), 3);
}

#[test]
fn test_area_chart_rendering() {
    let data = ChartData::Inline(vec![
        DataPoint {
            label: "Week 1".to_string(),
            value: 100.0,
            metadata: None,
        },
        DataPoint {
            label: "Week 2".to_string(),
            value: 150.0,
            metadata: None,
        },
    ]);

    let result = render_area_chart(&data, 800, 400).unwrap();

    assert!(result.contains("<svg"));
    assert!(result.contains("composition-area-chart"));
    assert!(result.contains("<path"));
    assert!(result.contains("fill-opacity"));
}

#[test]
fn test_bubble_chart_rendering() {
    let data = ChartData::Inline(vec![
        DataPoint {
            label: "Series A".to_string(),
            value: 80.0,
            metadata: None,
        },
        DataPoint {
            label: "Series B".to_string(),
            value: 120.0,
            metadata: None,
        },
    ]);

    let result = render_bubble_chart(&data, 800, 400).unwrap();

    assert!(result.contains("<svg"));
    assert!(result.contains("composition-bubble-chart"));
    assert!(result.contains("<circle"));
    assert_eq!(result.matches("<circle").count(), 2);
}

#[test]
fn test_empty_chart_data() {
    let data = ChartData::Inline(vec![]);

    let result = render_bar_chart(&data, 800, 400).unwrap();
    assert_eq!(result, "<svg></svg>");

    let result = render_line_chart(&data, 800, 400).unwrap();
    assert_eq!(result, "<svg></svg>");

    let result = render_pie_chart(&data, 400, 400).unwrap();
    assert_eq!(result, "<svg></svg>");
}

// Popover Tests

#[test]
fn test_inline_popover_rendering() {
    let result = render_inline_popover("Click here", "This is helpful info").unwrap();

    assert!(result.contains("composition-popover-wrapper"));
    assert!(result.contains("composition-popover-trigger"));
    assert!(result.contains("composition-popover-content"));
    assert!(result.contains("Click here"));
    assert!(result.contains("This is helpful info"));
    assert!(result.contains("data-popover-target"));
}

#[test]
fn test_popover_with_nodes() {
    let trigger = DarkMatterNode::Text("Hover me".to_string());
    let content = vec![
        DarkMatterNode::Text("Popover ".to_string()),
        DarkMatterNode::Text("content!".to_string()),
    ];

    let result = render_popover(&trigger, &content).unwrap();

    assert!(result.contains("Hover me"));
    assert!(result.contains("Popover content!"));
    assert!(result.contains("composition-popover-wrapper"));
}

#[test]
fn test_popover_html_escaping() {
    let result = render_inline_popover("<script>alert('xss')</script>", "Safe content").unwrap();

    assert!(result.contains("&lt;script&gt;"));
    assert!(!result.contains("<script>"));
}

#[test]
fn test_popover_unique_ids() {
    let result1 = render_inline_popover("A", "B").unwrap();
    let result2 = render_inline_popover("C", "D").unwrap();

    // IDs should be unique
    assert_ne!(result1, result2);
}

// Disclosure Tests

#[test]
fn test_disclosure_rendering() {
    let summary = vec![DarkMatterNode::Text("Click to expand".to_string())];
    let details = vec![
        DarkMatterNode::Text("This is ".to_string()),
        DarkMatterNode::Text("hidden content".to_string()),
    ];

    let result = render_disclosure(&summary, &details).unwrap();

    assert!(result.contains("<details"));
    assert!(result.contains("<summary"));
    assert!(result.contains("Click to expand"));
    assert!(result.contains("This is hidden content"));
    assert!(result.contains("composition-disclosure"));
}

#[test]
fn test_disclosure_open_state() {
    let summary = vec![DarkMatterNode::Text("Summary".to_string())];
    let details = vec![DarkMatterNode::Text("Details".to_string())];

    let result_closed = render_disclosure_open(&summary, &details, false).unwrap();
    assert!(!result_closed.contains(" open"));

    let result_open = render_disclosure_open(&summary, &details, true).unwrap();
    assert!(result_open.contains(" open"));
}

#[test]
fn test_disclosure_html_escaping() {
    let summary = vec![DarkMatterNode::Text("<b>Bold</b>".to_string())];
    let details = vec![DarkMatterNode::Text("A & B".to_string())];

    let result = render_disclosure(&summary, &details).unwrap();

    assert!(result.contains("&lt;b&gt;"));
    assert!(result.contains("&amp;"));
}

#[test]
fn test_disclosure_empty_content() {
    let summary = vec![];
    let details = vec![];

    let result = render_disclosure(&summary, &details).unwrap();

    assert!(result.contains("<details"));
    assert!(result.contains("<summary"));
}

// Columns Tests

#[test]
fn test_columns_basic() {
    let breakpoints = HashMap::new();
    let sections = vec![
        vec![DarkMatterNode::Text("Column 1".to_string())],
        vec![DarkMatterNode::Text("Column 2".to_string())],
        vec![DarkMatterNode::Text("Column 3".to_string())],
    ];

    let result = render_columns(&breakpoints, &sections).unwrap();

    assert!(result.contains("composition-columns"));
    assert!(result.contains("composition-column"));
    assert!(result.contains("Column 1"));
    assert!(result.contains("Column 2"));
    assert!(result.contains("Column 3"));
}

#[test]
fn test_columns_with_breakpoints() {
    let mut breakpoints = HashMap::new();
    breakpoints.insert(Breakpoint::Micro, 1);
    breakpoints.insert(Breakpoint::Md, 2);
    breakpoints.insert(Breakpoint::Lg, 3);

    let sections = vec![
        vec![DarkMatterNode::Text("A".to_string())],
        vec![DarkMatterNode::Text("B".to_string())],
        vec![DarkMatterNode::Text("C".to_string())],
    ];

    let result = render_columns(&breakpoints, &sections).unwrap();

    assert!(result.contains("composition-columns"));
    assert!(result.contains("A"));
    assert!(result.contains("B"));
    assert!(result.contains("C"));
}

#[test]
fn test_columns_styles_generation() {
    let mut breakpoints = HashMap::new();
    breakpoints.insert(Breakpoint::Md, 2);
    breakpoints.insert(Breakpoint::Lg, 3);

    let styles = generate_columns_styles(&breakpoints);

    assert!(styles.contains(".composition-columns"));
    assert!(styles.contains("grid-template-columns"));
    assert!(styles.contains("@media (min-width: 768px)"));
    assert!(styles.contains("@media (min-width: 1024px)"));
    assert!(styles.contains("repeat(2, 1fr)"));
    assert!(styles.contains("repeat(3, 1fr)"));
}

#[test]
fn test_columns_default_styles() {
    let breakpoints = HashMap::new();
    let styles = generate_columns_styles(&breakpoints);

    assert!(styles.contains("composition-columns-default"));
    assert!(styles.contains("grid-template-columns"));
}

#[test]
fn test_columns_empty_sections() {
    let breakpoints = HashMap::new();
    let sections: Vec<Vec<DarkMatterNode>> = vec![];

    let result = render_columns(&breakpoints, &sections).unwrap();

    assert_eq!(result, "");
}

#[test]
fn test_columns_html_escaping() {
    let breakpoints = HashMap::new();
    let sections = vec![vec![DarkMatterNode::Text("<script>bad</script>".to_string())]];

    let result = render_columns(&breakpoints, &sections).unwrap();

    assert!(result.contains("&lt;script&gt;"));
    assert!(!result.contains("<script>bad</script>"));
}

// Integration Tests

#[test]
fn test_multiple_chart_types() {
    let data = ChartData::Inline(vec![
        DataPoint {
            label: "Data".to_string(),
            value: 100.0,
            metadata: None,
        },
    ]);

    // All chart types should render successfully
    assert!(render_bar_chart(&data, 800, 400).is_ok());
    assert!(render_line_chart(&data, 800, 400).is_ok());
    assert!(render_pie_chart(&data, 400, 400).is_ok());
    assert!(render_area_chart(&data, 800, 400).is_ok());
    assert!(render_bubble_chart(&data, 800, 400).is_ok());
}

#[test]
fn test_nested_disclosure_in_columns() {
    // Test that disclosure blocks can be used within columns
    let summary = vec![DarkMatterNode::Text("Summary".to_string())];
    let details = vec![DarkMatterNode::Text("Details".to_string())];

    let disclosure = render_disclosure(&summary, &details).unwrap();

    let breakpoints = HashMap::new();
    let sections = vec![vec![DarkMatterNode::Text(disclosure)]];

    let result = render_columns(&breakpoints, &sections).unwrap();

    assert!(result.contains("composition-columns"));
    assert!(result.contains("details"));
}

#[test]
fn test_responsive_breakpoint_order() {
    let mut breakpoints = HashMap::new();
    // Insert in random order
    breakpoints.insert(Breakpoint::Xxl, 7);
    breakpoints.insert(Breakpoint::Micro, 1);
    breakpoints.insert(Breakpoint::Xs, 2);
    breakpoints.insert(Breakpoint::Lg, 5);
    breakpoints.insert(Breakpoint::Md, 4);
    breakpoints.insert(Breakpoint::Sm, 3);
    breakpoints.insert(Breakpoint::Xl, 6);

    let sections = vec![vec![DarkMatterNode::Text("Test".to_string())]];
    let result = render_columns(&breakpoints, &sections).unwrap();

    // Should successfully render regardless of insertion order
    assert!(result.contains("composition-columns"));
}
