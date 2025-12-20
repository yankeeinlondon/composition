use super::{Resource, Frontmatter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DarkMatter AST node representing various DSL elements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DarkMatterNode {
    // Transclusion
    File {
        resource: Resource,
        range: Option<LineRange>,
    },

    // AI operations
    Summarize {
        resource: Resource,
    },
    Consolidate {
        resources: Vec<Resource>,
    },
    Topic {
        topic: String,
        resources: Vec<Resource>,
        review: bool,
    },

    // Tables & Charts
    Table {
        source: TableSource,
        has_heading: bool,
    },
    BarChart {
        data: ChartData,
    },
    LineChart {
        data: ChartData,
    },
    PieChart {
        data: ChartData,
    },
    AreaChart {
        data: ChartData,
    },
    BubbleChart {
        data: ChartData,
    },

    // Layout
    Popover {
        trigger: Box<DarkMatterNode>,
        content: Vec<DarkMatterNode>,
    },
    Columns {
        breakpoints: HashMap<Breakpoint, u32>,
        sections: Vec<Vec<DarkMatterNode>>,
    },
    Disclosure {
        summary: Vec<DarkMatterNode>,
        details: Vec<DarkMatterNode>,
    },

    // Media
    Audio {
        source: String,
        name: Option<String>,
    },
    YouTube {
        video_id: String,
        width: super::youtube::WidthSpec,
    },

    // Text/content
    Text(String),
    Interpolation {
        variable: String,
    },
    Markdown(MarkdownContent),
}

/// Line range for partial file transclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineRange {
    pub start: usize,
    pub end: Option<usize>,
}

/// Source for table data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableSource {
    Inline(Vec<Vec<String>>),
    External(Resource),
}

/// Chart data source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartData {
    Inline(Vec<DataPoint>),
    External(Resource),
}

/// Data point for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub label: String,
    pub value: f64,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Responsive breakpoints (Tailwind-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Breakpoint {
    Micro, // 320px - mobile portrait
    Xs,    // 640px - mobile landscape (matches sm)
    Sm,    // 640px
    Md,    // 768px
    Lg,    // 1024px
    Xl,    // 1280px
    Xxl,   // 1536px
}

/// Markdown content wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownContent {
    pub raw: String,
    pub frontmatter: Option<Frontmatter>,
}
