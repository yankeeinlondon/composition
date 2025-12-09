# MDX Support with markdown-rs

MDX extends Markdown with JSX components, JavaScript expressions, and ESM imports. markdown-rs provides full MDX support through configurable constructs.

## Enabling MDX

```rust
use markdown::{to_html_with_options, Options, ParseOptions, Constructs};

let options = Options {
    parse: ParseOptions {
        constructs: Constructs {
            mdx_esm: true,           // import/export statements
            mdx_expression_flow: true,  // {expressions} in flow
            mdx_expression_text: true,  // {expressions} in text
            mdx_jsx_flow: true,      // <Component /> in flow
            mdx_jsx_text: true,      // <Component /> in text
            ..Constructs::gfm()
        },
        ..ParseOptions::default()
    },
    ..Options::default()
};

let html = to_html_with_options(mdx_input, &options)?;
```

## MDX Features

### JSX Components (Flow)

```mdx
# My Document

<Alert type="warning">
  This is a warning message.
</Alert>

<Card>
  ## Card Title
  Some content inside the card.
</Card>
```

**Construct**: `mdx_jsx_flow: true`

### JSX Components (Text/Inline)

```mdx
Read the <Link href="/docs">documentation</Link> for more info.
```

**Construct**: `mdx_jsx_text: true`

### Expressions (Flow)

```mdx
{/* This is a comment */}

{items.map(item => (
  <li key={item.id}>{item.name}</li>
))}
```

**Construct**: `mdx_expression_flow: true`

### Expressions (Text/Inline)

```mdx
The answer is {2 + 2}.

Hello, {user.name}!
```

**Construct**: `mdx_expression_text: true`

### ESM Imports/Exports

```mdx
import { Button } from './components'
import data from './data.json'

export const metadata = {
  title: 'My Page'
}

# Page Content

<Button onClick={() => console.log('clicked')}>
  Click me
</Button>
```

**Construct**: `mdx_esm: true`

## Full MDX Configuration

```rust
use markdown::{Options, ParseOptions, CompileOptions, Constructs};

fn mdx_options() -> Options {
    Options {
        parse: ParseOptions {
            constructs: Constructs {
                // MDX features
                mdx_esm: true,
                mdx_expression_flow: true,
                mdx_expression_text: true,
                mdx_jsx_flow: true,
                mdx_jsx_text: true,

                // GFM features (often used with MDX)
                gfm_table: true,
                gfm_task_list_item: true,
                gfm_strikethrough: true,
                gfm_autolink_literal: true,

                // Math support
                math_flow: true,
                math_text: true,

                ..Constructs::default()
            },
            // MDX-specific parse options
            mdx_esm_parse: None,          // Custom ESM parser
            mdx_expression_parse: None,   // Custom expression parser
            ..ParseOptions::default()
        },
        compile: CompileOptions {
            // MDX often needs dangerous HTML for components
            allow_dangerous_html: true,
            ..CompileOptions::default()
        },
    }
}
```

## Working with AST

MDX nodes are represented in the mdast AST:

```rust
use markdown::{to_mdast, ParseOptions, Constructs};
use markdown::mdast::Node;

fn parse_mdx(input: &str) {
    let options = ParseOptions {
        constructs: Constructs {
            mdx_jsx_flow: true,
            mdx_jsx_text: true,
            mdx_expression_flow: true,
            mdx_expression_text: true,
            mdx_esm: true,
            ..Constructs::default()
        },
        ..ParseOptions::default()
    };

    let ast = to_mdast(input, &options).unwrap();

    fn walk(node: &Node, depth: usize) {
        let indent = "  ".repeat(depth);
        match node {
            Node::MdxJsxFlowElement(jsx) => {
                println!("{}JSX Flow: <{}>", indent, jsx.name.as_deref().unwrap_or("fragment"));
            }
            Node::MdxJsxTextElement(jsx) => {
                println!("{}JSX Text: <{}>", indent, jsx.name.as_deref().unwrap_or("fragment"));
            }
            Node::MdxFlowExpression(expr) => {
                println!("{}Expression Flow: {{{}}}", indent, expr.value);
            }
            Node::MdxTextExpression(expr) => {
                println!("{}Expression Text: {{{}}}", indent, expr.value);
            }
            Node::MdxjsEsm(esm) => {
                println!("{}ESM: {}", indent, esm.value);
            }
            _ => {}
        }

        if let Some(children) = node.children() {
            for child in children {
                walk(child, depth + 1);
            }
        }
    }

    walk(&ast, 0);
}
```

## MDX AST Node Types

| Node Type | Description | Example |
|-----------|-------------|---------|
| `MdxJsxFlowElement` | Block-level JSX | `<Card>...</Card>` |
| `MdxJsxTextElement` | Inline JSX | `<Link>text</Link>` |
| `MdxFlowExpression` | Block-level expression | `{items.map(...)}` |
| `MdxTextExpression` | Inline expression | `{user.name}` |
| `MdxjsEsm` | Import/export | `import X from 'y'` |

## Integration with mdxjs-rs

For full MDX compilation to JavaScript, use the sibling `mdxjs-rs` crate:

```toml
[dependencies]
markdown = "1.0.0-alpha.21"
mdxjs = "0.2"
```

```rust
use mdxjs::{compile, Options};

fn compile_mdx(input: &str) -> Result<String, String> {
    let options = Options {
        // Configure development mode, JSX runtime, etc.
        development: false,
        ..Options::default()
    };

    compile(input, &options)
        .map_err(|e| e.to_string())
}
```

## Best Practices

1. **Enable only needed MDX features** - each construct adds parsing overhead
2. **Set `allow_dangerous_html: true`** when using JSX components
3. **Use mdxjs-rs for full compilation** - markdown-rs parses, mdxjs-rs compiles to JS
4. **Validate JSX syntax** - markdown-rs parses but doesn't validate JS/JSX
5. **Consider security** - MDX allows arbitrary code, only use with trusted content

## Common Patterns

### Extracting Component Usage

```rust
use markdown::mdast::{Node, MdxJsxFlowElement, MdxJsxTextElement};
use std::collections::HashSet;

fn extract_component_names(ast: &Node) -> HashSet<String> {
    let mut components = HashSet::new();

    fn walk(node: &Node, components: &mut HashSet<String>) {
        match node {
            Node::MdxJsxFlowElement(MdxJsxFlowElement { name: Some(n), .. }) |
            Node::MdxJsxTextElement(MdxJsxTextElement { name: Some(n), .. }) => {
                // Only track PascalCase names (components, not HTML elements)
                if n.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    components.insert(n.clone());
                }
            }
            _ => {}
        }

        if let Some(children) = node.children() {
            for child in children {
                walk(child, components);
            }
        }
    }

    walk(ast, &mut components);
    components
}
```

### Extracting Imports

```rust
use markdown::mdast::{Node, MdxjsEsm};

fn extract_imports(ast: &Node) -> Vec<String> {
    let mut imports = Vec::new();

    fn walk(node: &Node, imports: &mut Vec<String>) {
        if let Node::MdxjsEsm(MdxjsEsm { value, .. }) = node {
            if value.trim_start().starts_with("import") {
                imports.push(value.clone());
            }
        }

        if let Some(children) = node.children() {
            for child in children {
                walk(child, imports);
            }
        }
    }

    walk(ast, &mut imports);
    imports
}
```
