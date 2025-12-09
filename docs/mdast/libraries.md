---
_fixed: true
---

# MDAST Library Support Across Languages

The level of native support for the MDAST (Markdown Abstract Syntax Tree) specification varies significantly across programming languages.

## Summary

| Language | MDAST Support Status | Key Libraries/Approaches |
| :--- | :--- | :--- |
| **JavaScript/TypeScript** | **Native & Extensive** | Core ecosystem with `mdast` spec, `mdast-util-from-markdown`, `remark`, and many utilities. |
| **Rust** | **Good Native Support** | Libraries like `markdown-rs` and `mdast2minimad` can produce or work with MDAST. |
| **Go (Golang)** | **Not Found** | The search did not uncover any Go libraries that explicitly implement or support the MDAST format. |
| **Python** | **Not Found** | The search did not uncover any Python libraries that explicitly implement or support the MDAST format. |

## Detailed Findings by Language

- **JavaScript/TypeScript**: This is the native ecosystem for MDAST. The specification is maintained here, and there is a rich set of tools like `remark` for processing and utilities like `mdast-util-from-markdown` for parsing. You can also use `@types/mdast` for TypeScript definitions.
- **Rust**: There is active support. The `markdown-rs` library is a CommonMark parser written in Rust that can output an AST compatible with MDAST. Furthermore, crates like `mdast2minimad` exist specifically to convert or work with MDAST structures.
- **Go & Python**: The search results did not mention any libraries for these languages that implement the MDAST specification. This suggests that native, dedicated support is likely limited or non-existent.

## Recommendations for Working with MDAST

Given the uneven support, your approach will depend on your project's needs:

1. **For cross-language interoperability**: If your goal is to share a MDAST between services written in different languages, using JSON is the most practical path. Since MDAST is a data specification, you can serialize the tree to JSON in JavaScript or Rust and then deserialize it in Go or Python for further processing, even if those languages lack dedicated parsing libraries.
2. **If you must parse Markdown in Go or Python**: You will likely need to use each language's native Markdown libraries (like `goldmark` for Go or `markdown-it-py`/`mistune` for Python) and then write an adapter to transform their internal AST into a MDAST-compatible JSON structure.
3. **Verify for Go/Python**: To be absolutely certain, you could try searching specifically within the Go and Python community resources. You might use search terms like "Golang mdast" or "Python mdast util" on platforms like GitHub, `pkg.go.dev`, or PyPI.
