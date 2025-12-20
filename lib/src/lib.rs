//! # Composition Library
//!
//! A Rust library for document composition using the DarkMatter DSL.
//!
//! ## Overview
//!
//! The Composition library provides a powerful document composition system that extends
//! CommonMark/GFM markdown with advanced features including:
//!
//! - **Transclusion**: Include content from local and remote files
//! - **AI-powered operations**: Summarization, consolidation, and topic extraction
//! - **Smart image processing**: Responsive images with automatic format optimization
//! - **Caching**: Persistent caching via SurrealDB for performance
//! - **Dependency graphs**: Build and analyze document dependencies
//!
//! ## Quick Start
//!
//! ```no_run
//! use lib::{init, Resource, ResourceSource, ResourceRequirement};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the library
//!     let api = init(None, None).await?;
//!
//!     // Build dependency graph for a document
//!     let resource = Resource {
//!         source: ResourceSource::Local(PathBuf::from("document.md")),
//!         requirement: ResourceRequirement::Required,
//!         cache_duration: None,
//!     };
//!
//!     let graph = api.graph(resource).await?;
//!     println!("Document has {} dependencies", graph.nodes.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Core API Functions
//!
//! - [`init()`] - Initialize the library with database and frontmatter
//! - [`CompositionApi::graph()`] - Build dependency graph for a resource
//! - [`CompositionApi::generate_workplan()`] - Create optimized rendering plan
//! - [`CompositionApi::render()`] - Render documents with concurrency
//! - [`CompositionApi::to_html()`] - Convert to self-contained HTML
//!
//! ## Architecture
//!
//! The library is organized into several modules:
//!
//! - [`api`] - Main API surface and handle
//! - [`types`] - Core types (Resource, Document, Graph, etc.)
//! - [`error`] - Error types and Result aliases
//! - [`cache`] - SurrealDB-based caching
//! - [`parse`] - Markdown and DarkMatter DSL parsing
//! - [`graph`] - Dependency graph building and workplan generation
//! - [`image`] - Smart image processing and optimization
//! - [`render`] - Document rendering and transclusion
//! - [`ai`] - AI-powered operations (summarization, consolidation)

// Module declarations
pub mod api;
pub mod cache;
pub mod error;
pub mod graph;
pub mod init;
pub mod types;

// Implemented feature modules
pub mod parse;
pub mod image;

// Placeholder modules for future phases
pub mod render;
pub mod ai;

// Re-exports for convenience
pub use api::{CompositionApi, CompositionConfig, HtmlOutput, ImageSource, SmartImageOutput};
pub use error::{
    AIError, CacheError, CompositionError, ParseError, RenderError, Result,
};
pub use init::init;
pub use types::{
    Breakpoint, ChartData, DarkMatterNode, DataPoint, DependencyGraph, Document,
    Frontmatter, GraphNode, LineRange, ListExpansion, MarkdownContent, Resource,
    ResourceHash, ResourceRequirement, ResourceSource, TableSource, WorkLayer, WorkPlan,
};
