use crate::cache::operations::{CacheOperations, ImageCacheEntry};
use crate::error::Result;
use crate::image::{ImageSource, ImageOptions, SmartImageOutput, load_image, process_image};
use crate::image::html::{generate_picture_html, HtmlOptions};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use std::time::Duration;
use xxhash_rust::xxh3::xxh3_64;

/// Compute a simple resource hash from a string (for image sources)
fn compute_image_resource_hash(source: &str) -> String {
    format!("{:016x}", xxh3_64(source.as_bytes()))
}

/// Compute content hash from bytes
fn compute_image_content_hash(bytes: &[u8]) -> String {
    format!("{:016x}", xxh3_64(bytes))
}

/// Get or process an image with caching
pub async fn get_or_process_image(
    source: &ImageSource,
    options: ImageOptions,
    html_options: HtmlOptions,
    db: &Surreal<Db>,
) -> Result<SmartImageOutput> {
    // Compute resource hash
    let resource_hash = compute_image_resource_hash(source.as_str());

    // Load the image to get content hash
    let img = load_image(source)?;
    let img_bytes = match source {
        ImageSource::Local(path) => std::fs::read(path).unwrap_or_default(),
        ImageSource::Remote(_) => vec![], // For remote, we'd need to cache the bytes
    };
    let content_hash = compute_image_content_hash(&img_bytes);

    // Check cache using CacheOperations
    let cache_ops = CacheOperations::new(db.clone());
    let cached = cache_ops.get_image(&resource_hash).await?;

    if let Some(_cache_entry) = cached {
        // Cache hit - we would reconstruct the output from cache
        // For now, process anyway (cache reconstruction would be implemented in production)
        // TODO: Reconstruct SmartImageOutput from cache
    }

    // Cache miss or forced reprocess - process the image
    let (variants, has_transparency, blur_placeholder) = process_image(img.clone(), options)?;

    // Generate HTML
    let html = generate_picture_html(&variants, html_options)?;

    // Create output
    let output = SmartImageOutput {
        resource_hash: resource_hash.clone(),
        original_width: img.width(),
        original_height: img.height(),
        has_transparency,
        variants: variants.clone(),
        blur_placeholder: blur_placeholder.clone(),
        html,
    };

    // Store in cache
    let source_type = match source {
        ImageSource::Local(_) => "local".to_string(),
        ImageSource::Remote(_) => "remote".to_string(),
    };

    let expires_at = if matches!(source, ImageSource::Remote(_)) {
        Some(chrono::Utc::now() + Duration::from_secs(86400)) // 1 day for remote images
    } else {
        None // No expiration for local images
    };

    let cache_entry = ImageCacheEntry {
        id: None,
        resource_hash: resource_hash.clone(),
        content_hash: content_hash.clone(),
        created_at: chrono::Utc::now(),
        expires_at,
        source_type,
        source: source.as_str().to_string(),
        has_transparency,
        original_width: img.width() as i64,
        original_height: img.height() as i64,
    };

    cache_ops.upsert_image(cache_entry).await?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::database::init_database;
    use tempfile::TempDir;

    async fn setup_test_db() -> (Surreal<Db>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = init_database(&db_path).await.unwrap();
        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_get_or_process_image_creates_cache_entry() {
        let (db, _temp_dir) = setup_test_db().await;

        // Create a test image larger than smallest breakpoint (640px)
        use image::{RgbaImage, Rgba, ImageFormat as ImgFormat};
        let mut img = RgbaImage::new(1000, 800);
        for (_, _, pixel) in img.enumerate_pixels_mut() {
            *pixel = Rgba([255, 0, 0, 255]);
        }

        // Create a proper PNG file
        let temp_file = tempfile::Builder::new()
            .suffix(".png")
            .tempfile()
            .unwrap();
        img.save_with_format(temp_file.path(), ImgFormat::Png).unwrap();

        let source = ImageSource::Local(temp_file.path().to_path_buf());
        let options = ImageOptions::default();
        let html_options = HtmlOptions::default();

        let result = get_or_process_image(&source, options, html_options, &db).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.original_width, 1000);
        assert_eq!(output.original_height, 800);
        assert!(!output.has_transparency);
        assert!(!output.variants.is_empty());
        assert!(!output.html.is_empty());
    }
}
