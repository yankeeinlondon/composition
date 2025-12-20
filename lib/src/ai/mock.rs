use crate::ai::traits::{CompletionModel, EmbeddingModel};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

/// Mock completion model for deterministic testing.
///
/// This model returns predefined responses and tracks call counts.
#[derive(Clone)]
pub struct MockCompletionModel {
    responses: Arc<Mutex<Vec<String>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockCompletionModel {
    /// Create a new mock model with predefined responses.
    ///
    /// Responses are returned in order. If more calls are made than responses provided,
    /// the last response is repeated.
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Get the number of times the model has been called.
    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    /// Reset the call count to zero.
    pub fn reset_call_count(&self) {
        *self.call_count.lock().unwrap() = 0;
    }
}

#[async_trait]
impl CompletionModel for MockCompletionModel {
    async fn complete(
        &self,
        _prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut count = self.call_count.lock().unwrap();
        let responses = self.responses.lock().unwrap();

        let response_text = if responses.is_empty() {
            "Mock response".to_string()
        } else if *count < responses.len() {
            responses[*count].clone()
        } else {
            // Repeat last response
            responses.last().unwrap().clone()
        };

        *count += 1;

        Ok(response_text)
    }

    fn model_name(&self) -> &str {
        "mock-completion-model"
    }
}

/// Mock embedding model for deterministic testing.
///
/// This model returns fixed vectors and tracks call counts.
#[derive(Clone)]
pub struct MockEmbeddingModel {
    dimension: usize,
    call_count: Arc<Mutex<usize>>,
}

impl MockEmbeddingModel {
    /// Create a new mock embedding model with the specified dimension.
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Get the number of times the model has been called.
    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    /// Reset the call count to zero.
    pub fn reset_call_count(&self) {
        *self.call_count.lock().unwrap() = 0;
    }

    /// Generate a deterministic vector based on input text.
    fn generate_vector(&self, text: &str) -> Vec<f32> {
        // Generate deterministic vector based on text hash
        let hash = text.bytes().map(|b| b as u32).sum::<u32>();
        let base_value = (hash % 1000) as f32 / 1000.0;

        (0..self.dimension)
            .map(|i| {
                // Create variation across dimensions
                let offset = (i as f32 * 0.1).sin();
                base_value + offset * 0.1
            })
            .collect()
    }
}

#[async_trait]
impl EmbeddingModel for MockEmbeddingModel {
    async fn embed(
        &self,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;

        let embeddings = texts.iter().map(|text| self.generate_vector(text)).collect();

        Ok(embeddings)
    }

    fn model_name(&self) -> &str {
        "mock-embedding-model"
    }

    fn dimensions(&self) -> usize {
        self.dimension
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_completion_model_single_response() {
        let model = MockCompletionModel::new(vec!["Test response".to_string()]);

        let response = model.complete("Test prompt", None).await.unwrap();
        assert_eq!(response, "Test response");
        assert_eq!(model.call_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_completion_model_multiple_responses() {
        let model = MockCompletionModel::new(vec![
            "First response".to_string(),
            "Second response".to_string(),
        ]);

        let response1 = model.complete("Test prompt", None).await.unwrap();
        assert_eq!(response1, "First response");

        let response2 = model.complete("Test prompt", None).await.unwrap();
        assert_eq!(response2, "Second response");

        // Should repeat last response
        let response3 = model.complete("Test prompt", None).await.unwrap();
        assert_eq!(response3, "Second response");

        assert_eq!(model.call_count(), 3);
    }

    #[tokio::test]
    async fn test_mock_completion_model_reset_call_count() {
        let model = MockCompletionModel::new(vec!["Response".to_string()]);

        model.complete("Test", None).await.unwrap();
        assert_eq!(model.call_count(), 1);

        model.reset_call_count();
        assert_eq!(model.call_count(), 0);
    }

    #[tokio::test]
    async fn test_mock_embedding_model_dimensions() {
        let model = MockEmbeddingModel::new(1536);
        assert_eq!(model.dimensions(), 1536);

        let texts = vec!["Test text".to_string()];

        let embeddings = model.embed(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].len(), 1536);
    }

    #[tokio::test]
    async fn test_mock_embedding_model_deterministic() {
        let model = MockEmbeddingModel::new(128);

        let texts = vec!["Same text".to_string()];

        let embeddings1 = model.embed(&texts).await.unwrap();
        let embeddings2 = model.embed(&texts).await.unwrap();

        // Same input should produce same embedding
        assert_eq!(embeddings1[0], embeddings2[0]);
        assert_eq!(model.call_count(), 2);
    }

    #[tokio::test]
    async fn test_mock_embedding_model_different_inputs() {
        let model = MockEmbeddingModel::new(128);

        let texts = vec!["Text A".to_string(), "Text B".to_string()];

        let embeddings = model.embed(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 2);

        // Different inputs should produce different embeddings
        assert_ne!(embeddings[0], embeddings[1]);
    }
}
