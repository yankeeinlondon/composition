use crate::ai::traits::CompletionModel;
use crate::cache::operations::{CacheOperations, LlmCacheEntry};
use crate::error::{AIError, Result};
use chrono::{Duration, Utc};
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use tracing::{debug, instrument};
use xxhash_rust::xxh3::xxh3_64;

/// Default cache duration for LLM responses (30 days)
const DEFAULT_CACHE_DURATION_DAYS: i64 = 30;

/// Summarize a document using an LLM.
///
/// This function generates a concise summary of the provided text using the specified
/// completion model. Results are cached in SurrealDB to avoid redundant API calls.
///
/// # Arguments
///
/// * `db` - Database connection for caching
/// * `model` - The completion model to use for summarization
/// * `text` - The text content to summarize
/// * `max_tokens` - Optional maximum tokens for the summary
///
/// # Returns
///
/// The summarized text.
///
/// # Examples
///
/// ```no_run
/// use lib::ai::summarize::summarize;
/// use lib::ai::mock::MockCompletionModel;
/// use surrealdb::Surreal;
/// use surrealdb::engine::local::Mem;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = Surreal::new::<Mem>(()).await?;
/// let model = MockCompletionModel::new(vec!["Summary of the text.".to_string()]);
/// let summary = summarize(
///     Arc::new(db),
///     Arc::new(model),
///     "Long text to summarize...",
///     Some(150)
/// ).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(db, model, text))]
pub async fn summarize(
    db: Arc<Surreal<Db>>,
    model: Arc<dyn CompletionModel>,
    text: &str,
    max_tokens: Option<u32>,
) -> Result<String> {
    // Compute hash of input text
    let input_hash = format!("{:x}", xxh3_64(text.as_bytes()));
    let model_name = model.model_name();

    debug!(
        "Summarizing text (hash: {}, model: {})",
        input_hash, model_name
    );

    // Check cache first
    let cache = CacheOperations::new((*db).clone());
    if let Some(cached) = cache
        .get_llm("summarize", &input_hash, model_name)
        .await?
    {
        debug!("Cache hit for summarization");
        return Ok(cached.response);
    }

    debug!("Cache miss, calling LLM");

    // Build the summarization prompt
    let prompt = build_summarization_prompt(text, max_tokens);

    // Call the LLM
    let summary = model
        .complete(&prompt, max_tokens)
        .await
        .map_err(|e| AIError::SummarizationFailed(e.to_string()))?;

    // Cache the result
    let cache_entry = LlmCacheEntry {
        id: None,
        operation: "summarize".to_string(),
        input_hash,
        model: model_name.to_string(),
        response: summary.clone(),
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::days(DEFAULT_CACHE_DURATION_DAYS),
        tokens_used: None, // Token tracking can be added later
    };

    cache.upsert_llm(cache_entry).await?;

    Ok(summary)
}

/// Build the summarization prompt.
fn build_summarization_prompt(text: &str, max_tokens: Option<u32>) -> String {
    let length_guidance = if let Some(tokens) = max_tokens {
        format!(" Keep the summary under {} tokens.", tokens)
    } else {
        String::new()
    };

    format!(
        "Please provide a concise summary of the following text.{}

Text to summarize:
{}",
        length_guidance, text
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::mock::MockCompletionModel;
    use surrealdb::engine::local::Mem;

    async fn setup_test_db() -> Arc<Surreal<Db>> {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        // Initialize schema
        db.query(
            r#"
            DEFINE TABLE llm_cache SCHEMAFULL;
            DEFINE FIELD operation ON llm_cache TYPE string;
            DEFINE FIELD input_hash ON llm_cache TYPE string;
            DEFINE FIELD model ON llm_cache TYPE string;
            DEFINE FIELD response ON llm_cache TYPE string;
            DEFINE FIELD created_at ON llm_cache TYPE datetime;
            DEFINE FIELD expires_at ON llm_cache TYPE datetime;
            DEFINE FIELD tokens_used ON llm_cache TYPE option<int>;
            "#,
        )
        .await
        .unwrap();

        Arc::new(db)
    }

    #[tokio::test]
    async fn test_summarize_basic() {
        let db = setup_test_db().await;
        let model = Arc::new(MockCompletionModel::new(vec![
            "This is a summary.".to_string(),
        ]));

        let text = "This is a long document that needs to be summarized. It contains multiple sentences with various information.";

        let summary = summarize(db, model.clone(), text, None).await.unwrap();
        assert_eq!(summary, "This is a summary.");
        assert_eq!(model.call_count(), 1);
    }

    #[tokio::test]
    async fn test_summarize_with_max_tokens() {
        let db = setup_test_db().await;
        let model = Arc::new(MockCompletionModel::new(vec![
            "Short summary.".to_string(),
        ]));

        let text = "Long text here...";

        let summary = summarize(db, model, text, Some(50)).await.unwrap();
        assert_eq!(summary, "Short summary.");
    }

    #[tokio::test]
    async fn test_summarize_caching() {
        let db = setup_test_db().await;
        let model = Arc::new(MockCompletionModel::new(vec![
            "Cached summary.".to_string(),
        ]));

        let text = "Document to cache.";

        // First call - should hit the model
        let summary1 = summarize(db.clone(), model.clone(), text, None)
            .await
            .unwrap();
        assert_eq!(summary1, "Cached summary.");
        assert_eq!(model.call_count(), 1);

        // Second call - should hit the cache
        let summary2 = summarize(db, model.clone(), text, None).await.unwrap();
        assert_eq!(summary2, "Cached summary.");
        assert_eq!(model.call_count(), 1); // Should not increment
    }

    #[tokio::test]
    async fn test_summarize_different_inputs() {
        let db = setup_test_db().await;
        let model = Arc::new(MockCompletionModel::new(vec![
            "Summary 1.".to_string(),
            "Summary 2.".to_string(),
        ]));

        let text1 = "First document.";
        let text2 = "Second document.";

        let summary1 = summarize(db.clone(), model.clone(), text1, None)
            .await
            .unwrap();
        let summary2 = summarize(db, model.clone(), text2, None).await.unwrap();

        assert_eq!(summary1, "Summary 1.");
        assert_eq!(summary2, "Summary 2.");
        assert_eq!(model.call_count(), 2);
    }

    #[test]
    fn test_build_summarization_prompt_no_max_tokens() {
        let prompt = build_summarization_prompt("Test text", None);
        assert!(prompt.contains("Test text"));
        assert!(!prompt.contains("tokens"));
    }

    #[test]
    fn test_build_summarization_prompt_with_max_tokens() {
        let prompt = build_summarization_prompt("Test text", Some(100));
        assert!(prompt.contains("Test text"));
        assert!(prompt.contains("100 tokens"));
    }
}
