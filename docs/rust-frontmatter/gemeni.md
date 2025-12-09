# Rust Frontmatter Parsing Crates

The best Rust crates for parsing frontmatter metadata in a Markdown file offer a balance of format support, ease of use, and integration with the popular `serde` framework.

The top recommended crates are:

## 1. `markdown-frontmatter`

This is a dedicated, **type-safe** frontmatter parser designed specifically for this task.

- **Key Feature:** Provides a simple function to split the document into the structured frontmatter and the remaining body.
- **Format Support:** Supports **YAML** (delimited by `---`), **TOML** (delimited by `+++`), and **JSON** (delimited by `{` and `}`), typically enabled via Cargo features.
- **Deserialization:** Works seamlessly with `serde`, allowing you to deserialize the metadata directly into a custom Rust struct.
- **Usage:**

    ```rust
    // Example Rust struct for your frontmatter
    #[derive(serde::Deserialize)]
    struct Frontmatter {
        title: String,
        tags: Option<Vec<String>>,
    }

    // Call the parse function
    let doc = "--- title: My Post --- Body content";
    let (frontmatter, body) = markdown_frontmatter::parse::<Frontmatter>(doc)?;
    ```

---

## 2. `gray_matter`

This crate is a fast Rust implementation of the popular JavaScript `gray-matter` library, known for its robustness and flexibility.

- **Key Feature:** Highly flexible with support for **custom delimiters** and the ability to parse excerpts (a portion of the content body).
- **Format Support:** Built-in support for **YAML**, **TOML**, and **JSON**, also leveraging `serde` for deserialization into a custom struct.
- **Deserialization:** Deserializes the data into a generic type like `gray_matter::engine::YAML`'s internal `Pod` or directly into your custom `#[derive(Deserialize)]` struct.
- **Flexibility:** Allows you to define custom opening and closing delimiters if you need to support non-standard formats.

---

## 3. `markdown` (via `markdown-rs`)

If you are already using a CommonMark-compliant parser and want to enable frontmatter as an extension, the `markdown` crate is a strong choice.

- **Key Feature:** It's an all-in-one, CommonMark-compliant Markdown parser that supports frontmatter as an **extension**.
- **Context:** Useful if you need a full Markdown parser/renderer *and* metadata extraction in a single, robust tool.
- **Extensions:** To use frontmatter, you typically enable the `frontmatter` extension feature when configuring the parser.

---

## Comparison Table

| Crate | Primary Focus | Supported Formats | `serde` Integration | Custom Delimiters |
| :--- | :--- | :--- | :--- | :--- |
| **`markdown-frontmatter`** | Dedicated frontmatter extraction | YAML, TOML, JSON | Excellent | No |
| **`gray_matter`** | Flexible frontmatter/excerpt extraction | YAML, TOML, JSON | Excellent | Yes |
| **`markdown` (via `markdown-rs`)**| Full CommonMark parser/renderer | YAML, TOML (via feature) | Integrated | No |
