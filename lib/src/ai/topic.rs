use crate::ai::traits::CompletionModel;
use crate::cache::operations::{CacheOperations, LlmCacheEntry};
use crate::error::{AIError, Result};
use chrono::{Duration, Utc};
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use tracing::{debug, instrument};
use xxhash_rust::xxh3::xxh3_64;

const DEFAULT_CACHE_DURATION_DAYS: i64 = 30;

#[instrument(skip(db, model, documents))]
pub async fn extract_topic(
    db: Arc<Surreal<Db>>,
    model: Arc<dyn CompletionModel>,
    topic: &str,
    documents: &[&str],
    review: bool,
    max_tokens: Option<u32>,
) -> Result<String> {
    if documents.is_empty() {
        return Err(AIError::TopicExtractionFailed(
            "No documents provided for topic extraction".to_string(),
        )
        .into());
    }

    if topic.trim().is_empty() {
        return Err(AIError::TopicExtractionFailed(
            "Topic cannot be empty".to_string(),
        )
        .into());
    }

    let combined_input = format!(
        "topic:{}\nreview:{}\n{}",
        topic,
        review,
        documents.join("\n---DOCUMENT_SEPARATOR---\n")
    );
    let input_hash = format!("{:x}", xxh3_64(combined_input.as_bytes()));
    let model_name = model.model_name();

    debug!(
        "Extracting topic '{}' from {} documents (hash: {}, model: {})",
        topic,
        documents.len(),
        input_hash,
        model_name
    );

    let cache = CacheOperations::new((*db).clone());
    if let Some(cached) = cache
        .get_llm("topic_extraction", &input_hash, model_name)
        .await?
    {
        debug!("Cache hit for topic extraction");
        return Ok(cached.response);
    }

    debug!("Cache miss, calling LLM");

    let prompt = build_topic_extraction_prompt(topic, documents, review, max_tokens);
    let extracted = model
        .complete(&prompt, max_tokens)
        .await
        .map_err(|e| AIError::TopicExtractionFailed(e.to_string()))?;

    let cache_entry = LlmCacheEntry {
        id: None,
        operation: "topic_extraction".to_string(),
        input_hash,
        model: model_name.to_string(),
        response: extracted.clone(),
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::days(DEFAULT_CACHE_DURATION_DAYS),
        tokens_used: None,
    };

    cache.upsert_llm(cache_entry).await?;

    Ok(extracted)
}

fn build_topic_extraction_prompt(
    topic: &str,
    documents: &[&str],
    review: bool,
    max_tokens: Option<u32>,
) -> String {
    let length_guidance = if let Some(tokens) = max_tokens {
        format!(" Keep the output under {} tokens.", tokens)
    } else {
        String::new()
    };

    let review_instruction = if review {
        " After extracting the relevant content, provide a brief analysis or review of the findings."
    } else {
        ""
    };

    let mut prompt = format!(
        "Please extract all content related to the topic '{}' from the following {} documents. Include only the information that is directly relevant to this topic.{}{}

",
        topic,
        documents.len(),
        review_instruction,
        length_guidance
    );

    for (idx, doc) in documents.iter().enumerate() {
        prompt.push_str(&format!("--- Document {} ---\n{}\n\n", idx + 1, doc));
    }

    prompt.push_str(&format!(
        "Please provide the content related to '{}':",
        topic
    ));

    prompt
}
