use thiserror::Error;
use std::path::PathBuf;

/// Top-level error type for the composition library
#[derive(Error, Debug)]
pub enum CompositionError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),

    #[error("Render error: {0}")]
    Render(#[from] RenderError),

    #[error("AI error: {0}")]
    AI(#[from] AIError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Errors related to parsing markdown and DSL syntax
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid markdown at line {line}: {message}")]
    InvalidMarkdown { line: usize, message: String },

    #[error("Invalid DarkMatter directive at line {line}: {directive}")]
    InvalidDirective { line: usize, directive: String },

    #[error("Invalid frontmatter: {0}")]
    InvalidFrontmatter(String),

    #[error("Invalid resource reference: {0}")]
    InvalidResource(String),

    #[error("Circular dependency detected: {cycle}")]
    CircularDependency { cycle: String },

    #[error("Required resource not found: {resource}")]
    RequiredResourceNotFound { resource: String },

    #[error("Resource not found at {path}: {error}")]
    ResourceNotFound { path: String, error: String },

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("YAML parse error: {0}")]
    YamlParse(String),
}

/// Errors related to database and caching operations
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Failed to connect to database: {0}")]
    ConnectionFailed(String),

    #[error("Failed to initialize database at {path}: {error}")]
    InitializationFailed { path: PathBuf, error: String },

    #[error("Failed to execute query: {0}")]
    QueryFailed(String),

    #[error("Cache entry not found for key: {0}")]
    NotFound(String),

    #[error("Failed to serialize data: {0}")]
    SerializationError(String),

    #[error("Failed to deserialize data: {0}")]
    DeserializationError(String),

    #[error("Invalidation failed: {0}")]
    InvalidationFailed(String),
}

/// Errors related to rendering pipeline
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Failed to resolve transclusion: {resource}")]
    TransclusionFailed { resource: String },

    #[error("Failed to process image: {path}")]
    ImageProcessingFailed { path: String },

    #[error("Failed to generate HTML: {0}")]
    HtmlGenerationFailed(String),

    #[error("Failed to read file: {path}")]
    FileReadFailed { path: PathBuf },

    #[error("Failed to fetch remote resource: {url}")]
    RemoteFetchFailed { url: String },

    #[error("Template interpolation failed: {variable}")]
    InterpolationFailed { variable: String },

    #[error("Work plan generation failed: {0}")]
    WorkPlanFailed(String),

    #[error("Missing required dependency: {0}")]
    MissingDependency(String),

    #[error("Resource not found at {0}: {1}")]
    ResourceNotFound(String, String),

    #[error("Remote fetch error for {0}: {1}")]
    RemoteFetchError(String, String),

    #[error("Invalid line range: {0}")]
    InvalidLineRange(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("CSV parse error: {0}")]
    CsvError(String),

    #[error("Table rendering error: {0}")]
    TableError(String),

    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    #[error("Chart rendering error: {0}")]
    ChartError(String),

    #[error("Popover rendering error: {0}")]
    PopoverError(String),

    #[error("Column layout error: {0}")]
    ColumnError(String),

    #[error("Disclosure rendering error: {0}")]
    DisclosureError(String),
}

/// Errors related to AI/LLM operations
#[derive(Error, Debug)]
pub enum AIError {
    #[error("LLM provider error ({provider}): {message}")]
    ProviderError { provider: String, message: String },

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Invalid model configuration: {0}")]
    InvalidModelConfig(String),

    #[error("Summarization failed: {0}")]
    SummarizationFailed(String),

    #[error("Consolidation failed: {0}")]
    ConsolidationFailed(String),

    #[error("Topic extraction failed: {0}")]
    TopicExtractionFailed(String),

    #[error("Embedding generation failed: {0}")]
    EmbeddingFailed(String),

    #[error("API key not found for provider: {0}")]
    MissingApiKey(String),

    #[error("Rate limit exceeded for provider: {0}")]
    RateLimitExceeded(String),

    #[error("Request timeout: {0}")]
    Timeout(String),
}

/// Result type alias for composition operations
pub type Result<T> = std::result::Result<T, CompositionError>;

// Conversion from SurrealDB errors
impl From<surrealdb::Error> for CacheError {
    fn from(err: surrealdb::Error) -> Self {
        CacheError::Database(err.to_string())
    }
}

impl From<surrealdb::Error> for CompositionError {
    fn from(err: surrealdb::Error) -> Self {
        CompositionError::Cache(CacheError::from(err))
    }
}
