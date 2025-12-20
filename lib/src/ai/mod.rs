// AI module - Phase 6 implementation
// This module handles LLM operations (summarize, consolidate, topic extraction)

pub mod consolidate;
pub mod embedding;
pub mod mock;
pub mod providers;
pub mod summarize;
pub mod topic;
pub mod traits;

// Re-exports for convenience
pub use consolidate::consolidate;
pub use embedding::{find_similar, generate_embedding, EmbeddingEntry};
pub use mock::{MockCompletionModel, MockEmbeddingModel};
pub use summarize::summarize;
pub use topic::extract_topic;
pub use traits::{CompletionModel, EmbeddingModel};
