# GFM Extensions

GFM extends CommonMark with features designed for GitHub's use cases while maintaining readability.

## Tables

Create structured data with pipe-based syntax:

```markdown
| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |
```

**Enable with:** `Options::ENABLE_TABLES`

**Notes:**
- Basic table structure is supported
- Some GitHub-specific formatting nuances may require post-processing

## Task Lists

Checkbox functionality in lists:

```markdown
- [x] Completed task
- [ ] Incomplete task
```

**Enable with:** `Options::ENABLE_TASKLISTS`

**Use cases:**
- Project management
- Issue tracking
- Progress indicators

## Strikethrough

Mark text as deleted or no longer relevant:

```markdown
~~This text is struck through~~
```

**Enable with:** `Options::ENABLE_STRIKETHROUGH`

**Notes:**
- Uses one or two tildes around text
- Rendered as `<del>` or `<s>` tags in HTML

## Autolinks (Extended)

URLs automatically converted to links without markup:

```markdown
https://github.com
user@example.com
```

**Enable with:** `Options::ENABLE_AUTOLINK`

**Behavior:**
- Broader URL detection than CommonMark
- Email addresses auto-linked
- No angle brackets required

## Footnotes

Add numbered references:

```markdown
Here is a footnote reference[^1].

[^1]: This is the footnote content.
```

**Enable with:** `Options::ENABLE_FOOTNOTES`

## Comparison: CommonMark vs GFM

| Feature | CommonMark | GFM |
|---------|------------|-----|
| Tables | No | Yes |
| Task Lists | No | Yes |
| Strikethrough | No | Yes |
| Autolinks | Limited | Extended |
| HTML Restrictions | No | Yes (security) |
| Footnotes | No | Yes |

## Disallowed Raw HTML

For security, GFM restricts certain HTML tags that could be malicious. This is automatic in GFM contexts but not enforced by pulldown-cmark itself.

**Best practice:** Always sanitize HTML output for user-generated content.

## Related

- [Event Processing](./event-processing.md) - Handle extension events in code
- [Parsing Strategy](./parsing-strategy.md) - How GFM parsing works
