use async_trait::async_trait;

/// Trait for completion (text generation) models
#[async_trait]
pub trait CompletionModel: Send + Sync {
    /// Generate a completion for the given prompt
    async fn complete(&self, prompt: &str, max_tokens: Option<u32>) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;

    /// Get the model name/identifier
    fn model_name(&self) -> &str;
}

/// Trait for embedding (vector generation) models
#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    /// Generate embeddings for the given texts
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>>;

    /// Get the model name/identifier
    fn model_name(&self) -> &str;

    /// Get the number of dimensions in the embedding vectors
    fn dimensions(&self) -> usize;
}

/// Response from a completion request
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: String,
    pub tokens_used: Option<u32>,
}

/// Response from an embedding request
#[derive(Debug, Clone)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
}
