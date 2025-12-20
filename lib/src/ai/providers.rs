// Provider integration for AI models
//
// This module provides documentation and examples for integrating different AI providers.
// Since rig-core has API instability, we define our own traits in `traits.rs` and users
// can implement them for their preferred providers.
//
// # Example: OpenAI Integration
//
// To integrate OpenAI, you would implement the CompletionModel trait:
//
// ```rust,ignore
// use async_trait::async_trait;
// use crate::ai::traits::CompletionModel;
//
// struct OpenAIModel {
//     api_key: String,
//     model_name: String,
// }
//
// #[async_trait]
// impl CompletionModel for OpenAIModel {
//     async fn complete(&self, prompt: &str, max_tokens: Option<u32>) 
//         -> Result<String, Box<dyn std::error::Error + Send + Sync>> 
//     {
//         // Call OpenAI API here
//         todo!()
//     }
//
//     fn model_name(&self) -> &str {
//         &self.model_name
//     }
// }
// ```
//
// # Recommended Providers
//
// - OpenAI: Use `openai` crate or `async-openai`
// - Anthropic: Use `anthropic-sdk` or implement custom HTTP client
// - Ollama: Use HTTP client to call local API at http://localhost:11434
// - OpenRouter: OpenAI-compatible API at https://openrouter.ai/api/v1
//
// For now, use the MockCompletionModel and MockEmbeddingModel for testing.
