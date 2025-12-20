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
pub async fn consolidate(
    db: Arc<Surreal<Db>>,
    model: Arc<dyn CompletionModel>,
    documents: &[&str],
    max_tokens: Option<u32>,
) -> Result<String> {
    if documents.is_empty() {
        return Err(AIError::ConsolidationFailed(
            "No documents provided for consolidation".to_string(),
        )
        .into());
    }

    let combined_input = documents.join("\n---DOCUMENT_SEPARATOR---\n");
    let input_hash = format!("{:x}", xxh3_64(combined_input.as_bytes()));
    let model_name = model.model_name();

    debug!(
        "Consolidating {} documents (hash: {}, model: {})",
        documents.len(),
        input_hash,
        model_name
    );

    let cache = CacheOperations::new((*db).clone());
    if let Some(cached) = cache
        .get_llm("consolidate", &input_hash, model_name)
        .await?
    {
        debug!("Cache hit for consolidation");
        return Ok(cached.response);
    }

    debug!("Cache miss, calling LLM");

    let prompt = build_consolidation_prompt(documents, max_tokens);
    let consolidated = model
        .complete(&prompt, max_tokens)
        .await
        .map_err(|e| AIError::ConsolidationFailed(e.to_string()))?;

    let cache_entry = LlmCacheEntry {
        id: None,
        operation: "consolidate".to_string(),
        input_hash,
        model: model_name.to_string(),
        response: consolidated.clone(),
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::days(DEFAULT_CACHE_DURATION_DAYS),
        tokens_used: None,
    };

    cache.upsert_llm(cache_entry).await?;

    Ok(consolidated)
}

fn build_consolidation_prompt(documents: &[&str], max_tokens: Option<u32>) -> String {
    let length_guidance = if let Some(tokens) = max_tokens {
        format!(" Keep the consolidated output under {} tokens.", tokens)
    } else {
        String::new()
    };

    let mut prompt = format!(
        "Please consolidate the following {} documents into a single, coherent document. Remove redundancies, merge related information, and maintain a logical flow.{}

",
        documents.len(),
        length_guidance
    );

    for (idx, doc) in documents.iter().enumerate() {
        prompt.push_str(&format!("--- Document {} ---\n{}\n\n", idx + 1, doc));
    }

    prompt.push_str("Please provide the consolidated document:");
    prompt
}
