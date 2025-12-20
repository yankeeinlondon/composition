//! Phase 0.2: Async/Sync Integration Prototype
//!
//! This spike validates that:
//! - tokio + rayon can be integrated safely
//! - spawn_blocking works with rayon thread pool
//! - AI calls work inside async context
//! - No deadlocks occur under concurrent load

use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::task;

/// Task 0.2.1: Prototype tokio + rayon integration
#[tokio::test]
async fn test_tokio_rayon_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate processing items with both async and parallel operations
    let items: Vec<u32> = (0..100).collect();
    let counter = Arc::new(AtomicUsize::new(0));

    // Process items using tokio tasks that spawn blocking rayon work
    let mut handles = vec![];

    for chunk in items.chunks(10) {
        let chunk = chunk.to_vec();
        let counter_clone = counter.clone();

        let handle = tokio::spawn(async move {
            // Use spawn_blocking to run CPU-intensive rayon work
            task::spawn_blocking(move || {
                // Use rayon to parallelize within this chunk
                let sum: u32 = chunk
                    .par_iter()
                    .map(|&x| {
                        // Simulate CPU work
                        let result = expensive_computation(x);
                        counter_clone.fetch_add(1, Ordering::SeqCst);
                        result
                    })
                    .sum();
                sum
            })
            .await
            .expect("Blocking task should complete")
        });

        handles.push(handle);
    }

    // Wait for all tasks
    let mut total_sum = 0u32;
    for handle in handles {
        total_sum += handle.await?;
    }

    // Verify all items were processed
    assert_eq!(counter.load(Ordering::SeqCst), 100);
    println!("✓ Successfully integrated tokio + rayon for 100 items");
    println!("  Total sum: {}", total_sum);

    Ok(())
}

/// Task 0.2.2: Verify spawn_blocking works with rayon thread pool
#[tokio::test]
async fn test_spawn_blocking_with_rayon() -> Result<(), Box<dyn std::error::Error>> {
    let data: Vec<Vec<u32>> = (0..50).map(|i| vec![i; 100]).collect();

    // Process multiple batches concurrently
    let results: Vec<_> = data
        .into_iter()
        .map(|batch| {
            tokio::spawn(async move {
                task::spawn_blocking(move || {
                    // Use rayon inside spawn_blocking
                    batch
                        .par_iter()
                        .map(|&x| expensive_computation(x))
                        .sum::<u32>()
                })
                .await
                .expect("Should complete")
            })
        })
        .collect();

    // Await all results
    let mut total = 0u32;
    for result in results {
        total += result.await?;
    }

    println!("✓ spawn_blocking + rayon processed 50 batches without deadlock");
    println!("  Total: {}", total);

    Ok(())
}

/// Task 0.2.3: Test AI call inside async context (simulated)
#[tokio::test]
async fn test_simulated_ai_call_in_async() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate multiple documents that need AI processing
    let documents: Vec<String> = (0..20)
        .map(|i| format!("Document content for item {}", i))
        .collect();

    let results: Vec<_> = documents
        .into_iter()
        .map(|doc| {
            tokio::spawn(async move {
                // Simulate async AI API call
                let summary = simulate_ai_summarize(&doc).await;

                // Also do some CPU-intensive work
                task::spawn_blocking(move || {
                    expensive_text_processing(&summary)
                })
                .await
                .expect("Should complete")
            })
        })
        .collect();

    // Collect all results
    let summaries: Vec<_> = futures::future::try_join_all(results).await?;

    assert_eq!(summaries.len(), 20);
    println!("✓ Successfully processed 20 AI calls in async context");

    Ok(())
}

/// Task 0.2.4: Confirm no deadlocks under concurrent load
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_heavy_concurrent_load() -> Result<(), Box<dyn std::error::Error>> {
    let num_tasks = 100;
    let items_per_task = 50;

    let start = std::time::Instant::now();
    let processed_count = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    for task_id in 0..num_tasks {
        let items: Vec<u32> = (0..items_per_task)
            .map(|i| task_id * items_per_task + i)
            .collect();
        let processed_clone = processed_count.clone();

        let handle = tokio::spawn(async move {
            // Mix async and blocking work
            let async_result = simulate_async_work(task_id).await;

            // CPU-bound work with rayon
            let blocking_result = task::spawn_blocking(move || {
                items
                    .par_iter()
                    .map(|&x| {
                        processed_clone.fetch_add(1, Ordering::SeqCst);
                        expensive_computation(x)
                    })
                    .sum::<u32>()
            })
            .await
            .expect("Should complete");

            async_result + blocking_result
        });

        handles.push(handle);
    }

    // Wait for all tasks
    let results: Vec<_> = futures::future::try_join_all(handles).await?;

    let duration = start.elapsed();
    let total_items = processed_count.load(Ordering::SeqCst);

    assert_eq!(total_items, num_tasks as usize * items_per_task as usize);
    println!("✓ Completed {} tasks with {} total items in {:?}", num_tasks, total_items, duration);
    println!("  No deadlocks detected");
    println!("  Throughput: {:.2} items/sec", total_items as f64 / duration.as_secs_f64());

    Ok(())
}

/// Test integration pattern: async function that uses rayon for parallel work
#[tokio::test]
async fn test_realistic_document_processing() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate processing a document with multiple steps
    let document = "Large document content".to_string();

    // Step 1: Parse (CPU-bound, use rayon)
    let parsed = task::spawn_blocking(move || {
        parse_document_with_rayon(&document)
    })
    .await?;

    println!("✓ Parsed document with {} sections", parsed.len());

    // Step 2: Process each section with AI (async)
    let ai_results: Vec<_> = parsed
        .into_iter()
        .map(|section| async move {
            simulate_ai_summarize(&section).await
        })
        .collect();

    let summaries = futures::future::join_all(ai_results).await;

    println!("✓ Generated {} AI summaries", summaries.len());

    // Step 3: Combine results (CPU-bound)
    let final_result = task::spawn_blocking(move || {
        combine_with_rayon(summaries)
    })
    .await?;

    println!("✓ Combined results: {} chars", final_result.len());

    Ok(())
}

// ===== Helper Functions =====

fn expensive_computation(x: u32) -> u32 {
    // Simulate CPU-intensive work
    (0..1000).fold(x, |acc, i| acc.wrapping_add(i))
}

async fn simulate_ai_summarize(text: &str) -> String {
    // Simulate network delay for AI API
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    format!("Summary of: {}", &text[..text.len().min(20)])
}

async fn simulate_async_work(id: u32) -> u32 {
    // Simulate async I/O
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    id * 2
}

fn expensive_text_processing(text: &str) -> String {
    // Simulate CPU work
    text.chars()
        .collect::<Vec<_>>()
        .par_iter()
        .map(|c| c.to_uppercase().to_string())
        .collect::<String>()
}

fn parse_document_with_rayon(doc: &str) -> Vec<String> {
    // Simulate parsing with parallel processing
    let sections: Vec<&str> = doc.split_whitespace().collect();

    sections
        .par_iter()
        .map(|section| {
            // Simulate expensive parsing
            expensive_computation(section.len() as u32);
            section.to_string()
        })
        .collect()
}

fn combine_with_rayon(summaries: Vec<String>) -> String {
    summaries
        .par_iter()
        .map(|s| format!("- {}\n", s))
        .collect::<String>()
}
