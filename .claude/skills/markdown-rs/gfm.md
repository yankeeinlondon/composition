# GitHub Flavored Markdown (GFM) with markdown-rs

GFM extends CommonMark with features commonly used on GitHub: tables, task lists, strikethrough, autolinks, and footnotes.

## Enabling GFM

### All GFM Features

```rust
use markdown::{to_html_with_options, Options};

let html = to_html_with_options(input, &Options::gfm())?;
```

### Selective Features

```rust
use markdown::{to_html_with_options, Options, ParseOptions, Constructs};

let options = Options {
    parse: ParseOptions {
        constructs: Constructs {
            gfm_table: true,
            gfm_task_list_item: true,
            gfm_strikethrough: true,
            gfm_autolink_literal: true,
            // Disable footnotes if not needed
            gfm_footnote_definition: false,
            gfm_footnote_label: false,
            gfm_footnote_reference: false,
            ..Constructs::default()
        },
        ..ParseOptions::default()
    },
    ..Options::default()
};

let html = to_html_with_options(input, &options)?;
```

## Feature Reference

### Tables

```markdown
| Left | Center | Right |
|:-----|:------:|------:|
| L    | C      | R     |
```

**Construct flag**: `gfm_table: true`

**Generated HTML**:
```html
<table>
<thead><tr><th align="left">Left</th><th align="center">Center</th><th align="right">Right</th></tr></thead>
<tbody><tr><td align="left">L</td><td align="center">C</td><td align="right">R</td></tr></tbody>
</table>
```

**AST Node**: `Node::Table` with `align` vector and `Node::TableRow`/`Node::TableCell` children.

### Task Lists

```markdown
- [x] Completed task
- [ ] Incomplete task
- Regular list item
```

**Construct flag**: `gfm_task_list_item: true`

**Generated HTML**:
```html
<ul>
<li><input type="checkbox" disabled checked /> Completed task</li>
<li><input type="checkbox" disabled /> Incomplete task</li>
<li>Regular list item</li>
</ul>
```

**AST Node**: `Node::ListItem` with `checked: Option<bool>` field.

### Strikethrough

```markdown
~~deleted text~~
```

**Construct flag**: `gfm_strikethrough: true`

**Generated HTML**: `<del>deleted text</del>`

**AST Node**: `Node::Delete` with children.

### Autolink Literals

```markdown
Visit https://example.com or email user@example.com
```

**Construct flag**: `gfm_autolink_literal: true`

URLs and emails are automatically linked without requiring `<angle brackets>`.

### Footnotes

```markdown
Here's a statement with a footnote[^1].

[^1]: This is the footnote content.
```

**Construct flags**:
- `gfm_footnote_reference: true` - for `[^1]` references
- `gfm_footnote_definition: true` - for `[^1]: content` definitions
- `gfm_footnote_label: true` - for label parsing

## Working with AST

### Extracting Task List Status

```rust
use markdown::{to_mdast, ParseOptions, Constructs, mdast::{Node, ListItem}};

fn extract_tasks(markdown: &str) -> Vec<(String, bool)> {
    let options = ParseOptions {
        constructs: Constructs::gfm(),
        ..ParseOptions::default()
    };
    let ast = to_mdast(markdown, &options).unwrap();
    let mut tasks = Vec::new();

    fn walk(node: &Node, tasks: &mut Vec<(String, bool)>) {
        if let Node::ListItem(ListItem { checked: Some(done), children, .. }) = node {
            // Extract text from children
            let text = extract_text(children);
            tasks.push((text, *done));
        }
        if let Some(children) = node.children() {
            for child in children {
                walk(child, tasks);
            }
        }
    }

    fn extract_text(nodes: &[Node]) -> String {
        nodes.iter()
            .map(|n| match n {
                Node::Text(t) => t.value.clone(),
                Node::Paragraph(p) => extract_text(&p.children),
                _ => String::new(),
            })
            .collect()
    }

    walk(&ast, &mut tasks);
    tasks
}
```

### Extracting Table Data

```rust
use markdown::{to_mdast, ParseOptions, Constructs};
use markdown::mdast::{Node, Table, TableRow, TableCell, AlignKind};

struct TableData {
    headers: Vec<String>,
    alignments: Vec<AlignKind>,
    rows: Vec<Vec<String>>,
}

fn extract_table(markdown: &str) -> Option<TableData> {
    let options = ParseOptions {
        constructs: Constructs::gfm(),
        ..ParseOptions::default()
    };
    let ast = to_mdast(markdown, &options).ok()?;

    fn find_table(node: &Node) -> Option<&Table> {
        match node {
            Node::Table(t) => Some(t),
            _ => node.children()?.iter().find_map(find_table),
        }
    }

    let table = find_table(&ast)?;
    let alignments = table.align.clone();

    let mut rows_iter = table.children.iter();
    let header_row = rows_iter.next()?;
    let headers = extract_row_text(header_row);

    let rows: Vec<Vec<String>> = rows_iter.map(|row| extract_row_text(row)).collect();

    Some(TableData { headers, alignments, rows })
}

fn extract_row_text(row: &Node) -> Vec<String> {
    if let Node::TableRow(TableRow { children, .. }) = row {
        children.iter().map(|cell| {
            if let Node::TableCell(TableCell { children, .. }) = cell {
                cell_to_text(children)
            } else {
                String::new()
            }
        }).collect()
    } else {
        vec![]
    }
}

fn cell_to_text(nodes: &[Node]) -> String {
    nodes.iter()
        .filter_map(|n| if let Node::Text(t) = n { Some(t.value.as_str()) } else { None })
        .collect()
}
```

## Constructs Reference

| Construct | Flag | Markdown Syntax |
|-----------|------|-----------------|
| Tables | `gfm_table` | `\| cell \|` with `---` separators |
| Task Lists | `gfm_task_list_item` | `- [x]` or `- [ ]` |
| Strikethrough | `gfm_strikethrough` | `~~text~~` |
| Autolink Literals | `gfm_autolink_literal` | bare `https://...` or `email@...` |
| Footnote References | `gfm_footnote_reference` | `[^label]` |
| Footnote Definitions | `gfm_footnote_definition` | `[^label]: content` |
| Footnote Labels | `gfm_footnote_label` | Label parsing for footnotes |
| Tag Filter | `gfm_tagfilter` | Filters dangerous HTML tags |

## Best Practices

1. **Use `Options::gfm()` for full compatibility** with GitHub rendering
2. **Enable only needed constructs** for better performance
3. **Use task lists for interactive features** - check state is in AST
4. **Tables support alignment** - useful for numeric data
5. **Footnotes generate IDs** - may need CSS styling
6. **Autolinks are security-safe** - javascript: URLs are blocked by default
