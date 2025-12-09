# Code Examples

## Basic Markdown to HTML

```rust
use pulldown_cmark::{Parser, html};

fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn main() {
    let md = "# Hello\n\nThis is **bold** and *italic*.";
    println!("{}", markdown_to_html(md));
}
```

## With Extensions

```rust
use pulldown_cmark::{Parser, html, Options};

fn markdown_to_html_extended(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
```

## Extract All Headings

```rust
use pulldown_cmark::{Parser, Event, Tag, HeadingLevel};

fn extract_headings(markdown: &str) -> Vec<(HeadingLevel, String)> {
    let parser = Parser::new(markdown);
    let mut headings = Vec::new();
    let mut current_level = None;
    let mut current_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_level = Some(level);
                current_text.clear();
            }
            Event::Text(text) if current_level.is_some() => {
                current_text.push_str(&text);
            }
            Event::End(pulldown_cmark::TagEnd::Heading(_)) => {
                if let Some(level) = current_level.take() {
                    headings.push((level, std::mem::take(&mut current_text)));
                }
            }
            _ => {}
        }
    }
    headings
}
```

## Build Table of Contents

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel};

#[derive(Debug)]
struct TocEntry {
    level: u8,
    id: String,
    title: String,
}

fn build_toc(markdown: &str) -> Vec<TocEntry> {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown, options);
    let mut toc = Vec::new();
    let mut current: Option<(HeadingLevel, Option<String>)> = None;
    let mut title = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, id, .. }) => {
                current = Some((level, id.map(|s| s.into_string())));
                title.clear();
            }
            Event::Text(text) if current.is_some() => {
                title.push_str(&text);
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((level, id)) = current.take() {
                    let slug = id.unwrap_or_else(|| slugify(&title));
                    toc.push(TocEntry {
                        level: level as u8,
                        id: slug,
                        title: std::mem::take(&mut title),
                    });
                }
            }
            _ => {}
        }
    }
    toc
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
```

## Extract All Links

```rust
use pulldown_cmark::{Parser, Event, Tag};

#[derive(Debug)]
struct Link {
    url: String,
    title: String,
    text: String,
}

fn extract_links(markdown: &str) -> Vec<Link> {
    let parser = Parser::new(markdown);
    let mut links = Vec::new();
    let mut current_link: Option<(String, String)> = None;
    let mut link_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Link { dest_url, title, .. }) => {
                current_link = Some((dest_url.into_string(), title.into_string()));
                link_text.clear();
            }
            Event::Text(text) if current_link.is_some() => {
                link_text.push_str(&text);
            }
            Event::End(pulldown_cmark::TagEnd::Link) => {
                if let Some((url, title)) = current_link.take() {
                    links.push(Link {
                        url,
                        title,
                        text: std::mem::take(&mut link_text),
                    });
                }
            }
            _ => {}
        }
    }
    links
}
```

## Add target="_blank" to External Links

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd, html, CowStr};

fn add_external_link_attrs(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut in_link = false;
    let mut link_url = String::new();
    let mut link_title = String::new();
    let mut link_content = Vec::new();

    let transformed = parser.flat_map(move |event| {
        match event {
            Event::Start(Tag::Link { dest_url, title, .. }) => {
                in_link = true;
                link_url = dest_url.into_string();
                link_title = title.into_string();
                link_content.clear();
                vec![]
            }
            Event::End(TagEnd::Link) if in_link => {
                in_link = false;
                let is_external = link_url.starts_with("http://") || link_url.starts_with("https://");

                // Render link content to HTML
                let mut inner_html = String::new();
                html::push_html(&mut inner_html, link_content.drain(..).into_iter());

                let html = if is_external {
                    format!(
                        r#"<a href="{}" title="{}" target="_blank" rel="noopener noreferrer">{}</a>"#,
                        html_escape(&link_url),
                        html_escape(&link_title),
                        inner_html
                    )
                } else {
                    format!(
                        r#"<a href="{}" title="{}">{}</a>"#,
                        html_escape(&link_url),
                        html_escape(&link_title),
                        inner_html
                    )
                };
                vec![Event::Html(html.into())]
            }
            other if in_link => {
                link_content.push(other);
                vec![]
            }
            other => vec![other],
        }
    });

    let mut output = String::new();
    html::push_html(&mut output, transformed);
    output
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
```

## Calculate Document Statistics

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd};

#[derive(Debug, Default)]
struct DocStats {
    word_count: usize,
    heading_count: usize,
    paragraph_count: usize,
    link_count: usize,
    code_block_count: usize,
    max_nesting: usize,
}

fn analyze_document(markdown: &str) -> DocStats {
    let parser = Parser::new(markdown);
    let mut stats = DocStats::default();
    let mut current_depth = 0;

    for event in parser {
        match event {
            Event::Start(tag) => {
                current_depth += 1;
                stats.max_nesting = stats.max_nesting.max(current_depth);
                match tag {
                    Tag::Heading { .. } => stats.heading_count += 1,
                    Tag::Paragraph => stats.paragraph_count += 1,
                    Tag::Link { .. } => stats.link_count += 1,
                    Tag::CodeBlock(_) => stats.code_block_count += 1,
                    _ => {}
                }
            }
            Event::End(_) => {
                current_depth -= 1;
            }
            Event::Text(text) => {
                stats.word_count += text.split_whitespace().count();
            }
            _ => {}
        }
    }
    stats
}
```

## Render with Source Positions

```rust
use pulldown_cmark::{Parser, Event};

fn render_with_positions(markdown: &str) {
    let parser = Parser::new(markdown);

    for (event, range) in parser.into_offset_iter() {
        let source = &markdown[range.clone()];
        match event {
            Event::Start(tag) => {
                println!("[{}..{}] START {:?}", range.start, range.end, tag);
            }
            Event::End(tag) => {
                println!("[{}..{}] END {:?}", range.start, range.end, tag);
            }
            Event::Text(_) => {
                println!("[{}..{}] TEXT: {:?}", range.start, range.end, source);
            }
            other => {
                println!("[{}..{}] {:?}", range.start, range.end, other);
            }
        }
    }
}
```

## Wrap Headings in Sections

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel, html, CowStr};

fn wrap_headings_in_sections(markdown: &str) -> String {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown, options);
    let mut events: Vec<Event> = Vec::new();
    let mut section_stack: Vec<HeadingLevel> = Vec::new();

    for event in parser {
        match &event {
            Event::Start(Tag::Heading { level, id, .. }) => {
                // Close sections at same or lower level
                while let Some(open_level) = section_stack.last() {
                    if *level <= *open_level {
                        events.push(Event::Html("</section>\n".into()));
                        section_stack.pop();
                    } else {
                        break;
                    }
                }
                // Open new section
                let id_attr = id.as_ref()
                    .map(|i| format!(r#" id="section-{}""#, i))
                    .unwrap_or_default();
                events.push(Event::Html(format!("<section{}>\n", id_attr).into()));
                section_stack.push(*level);
                events.push(event);
            }
            _ => events.push(event),
        }
    }

    // Close remaining sections
    for _ in section_stack {
        events.push(Event::Html("</section>\n".into()));
    }

    let mut output = String::new();
    html::push_html(&mut output, events.into_iter());
    output
}
```

## Strip Markdown to Plain Text

```rust
use pulldown_cmark::{Parser, Event};

fn strip_markdown(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut plain_text = String::new();

    for event in parser {
        match event {
            Event::Text(text) | Event::Code(text) => {
                plain_text.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak => {
                plain_text.push('\n');
            }
            Event::Start(_) | Event::End(_) => {
                // Add space between block elements
                if !plain_text.ends_with(' ') && !plain_text.ends_with('\n') {
                    plain_text.push(' ');
                }
            }
            _ => {}
        }
    }
    plain_text.trim().to_string()
}
```
