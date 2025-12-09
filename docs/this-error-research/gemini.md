---
_fixed: true
---

# The `thiserror` Crate in Rust

The **thiserror** crate is a popular and powerful utility in **Rust** for creating **idiomatic and clean error types** using procedural macros. It significantly reduces the boilerplate required for implementing standard traits like `std::fmt::Display` and `std::error::Error`.

## Key Features and Usage

`thiserror` is used by annotating an enum or struct that represents your custom error type.

### Deriving Error Traits

The core functionality comes from the `#[derive(Error)]` attribute, which automatically implements:

- **`std::error::Error`**: The fundamental trait for error handling in Rust.
- **`std::fmt::Debug`**: Derived as usual, often required for error types.
- **`std::fmt::Display`**: Implemented based on the provided `#[error(...)]` attributes.

### Implementing `Display` with `#[error]`

You define the user-facing message for each variant using the `#[error(...)]` attribute, which supports standard **Rust formatting syntax**:

- **Simple Variant:**

    ```rust
    #[derive(Error, Debug)]
    pub enum MyError {
        #[error("Invalid configuration value provided.")]
        InvalidConfig,
        // ...
    }
    ```

- **Variant with Data:** You can reference fields within the variant's data using their indices (for tuple variants) or names (for struct variants).

    ```rust
    #[derive(Error, Debug)]
    pub enum MyError {
        #[error("Failed to read file '{0}'.")] // Use {0} for the first tuple field
        IoError(String),

        #[error("Resource limit of {limit} exceeded for user '{user}'.")]
        ResourceLimitExceeded { limit: usize, user: String },
        // ...
    }
    ```

### Handling Source Errors (Chaining)

To create an error chain (where one error is caused by another), you use the `#[source]` attribute on a field within the error variant. The type of this field must implement the `std::error::Error` trait.

This automatically implements the `source()` method from `std::error::Error`:

```rust
use std::io;

#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error("Database query failed.")]
    DatabaseError {
        #[source] // This field is the source error
        source: sqlx::Error,
    },

    #[error("File operation error.")]
    IoOperation(
        #[from] // The `#[from]` attribute is often used with `#[source]`
        #[source] // Note: `#[from]` implies `#[source]` on enums, but is explicit here for clarity.
        io::Error
    ),
}
```

### Automatic `From` Implementation

The `#[from]` attribute is a highly convenient feature, especially for enums. When placed on a variant, it automatically implements `From<T>` for the error type, where `T` is the type of the single field in the variant. This allows using the **`?` operator** to convert the source error into your custom error type.

```rust
use std::io;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    // Automatically implements From<io::Error>
    #[error("Could not read configuration file: {0}")]
    Io(#[from] io::Error),

    // Automatically implements From<serde_json::Error>
    #[error("Could not parse configuration: {0}")]
    Parse(#[from] serde_json::Error),
}

// When a function returns Result<(), ConfigError>, you can use `?` on
// io::read_to_string() or serde_json::from_str().
```

---

## Comparison with Other Choices

The primary goal of error handling crates is to reduce boilerplate, especially the repetitive implementation of `Display`, `Debug`, and `Error`.

### Comparison with `anyhow`

| Feature | `thiserror` | `anyhow` |
| :--- | :--- | :--- |
| **Purpose** | **Defines Library Errors** (Specific, controlled error types) | **Handles Application Errors** (Quick, generic error wrapping) |
| **Type** | **Specific `enum` or `struct`** (Your custom error type) | **Generic `anyhow::Error`** (A trait object `dyn Error + Send + Sync`) |
| **Ergonomics** | Excellent for *defining* the error structure. | Excellent for *propagating* errors with context. |
| **Best Use** | Libraries and code that needs precise error types | Applications where you need quick error handling |
| **Context** | Add context via error variants | Add context via `.context()` method |
