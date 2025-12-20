use image::{Rgba, RgbaImage, ImageFormat as ImgFormat};
use lib::cache::database::init_database;
use lib::image::{
    get_or_process_image, ImageOptions, ImageSource,
};
use lib::image::html::{HtmlOptions, LayoutMode, Loading, Decoding};
use tempfile::TempDir;

async fn setup_test_db() -> (surrealdb::Surreal<surrealdb::engine::local::Db>, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = init_database(&db_path).await.unwrap();
    (db, temp_dir)
}

fn create_test_image(width: u32, height: u32, has_transparency: bool) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = if has_transparency {
            Rgba([255, 0, 0, 128]) // Red with 50% transparency
        } else {
            Rgba([0, 0, 255, 255]) // Opaque blue
        };
    }
    img
}

#[tokio::test]
async fn test_process_local_image_creates_variants() {
    let (db, temp_dir) = setup_test_db().await;

    // Create a test image
    let img = create_test_image(1200, 800, false);
    let temp_path = temp_dir.path().join("test.png");
    img.save_with_format(&temp_path, ImgFormat::Png).unwrap();

    let source = ImageSource::Local(temp_path.clone());
    let options = ImageOptions::default();
    let html_options = HtmlOptions::default();

    let result = get_or_process_image(&source, options, html_options, &db).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.original_width, 1200);
    assert_eq!(output.original_height, 800);
    assert!(!output.has_transparency);
    assert!(!output.variants.is_empty());
    assert!(output.html.contains("<picture>"));
    assert!(output.html.contains("</picture>"));
    assert!(!output.blur_placeholder.is_empty());
}

#[tokio::test]
async fn test_transparent_image_uses_correct_formats() {
    let (db, temp_dir) = setup_test_db().await;

    // Create a transparent test image
    let img = create_test_image(640, 480, true);
    let temp_path = temp_dir.path().join("test_transparent.png");
    img.save_with_format(&temp_path, ImgFormat::Png).unwrap();

    let source = ImageSource::Local(temp_path);
    let options = ImageOptions::default();
    let html_options = HtmlOptions::default();

    let result = get_or_process_image(&source, options, html_options, &db).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.has_transparency);

    // Transparent images should use PNG/WebP/AVIF, not JPEG
    let has_jpeg = output.variants.iter().any(|v| {
        matches!(v.format, lib::image::ImageFormat::Jpeg)
    });
    assert!(!has_jpeg, "Transparent images should not have JPEG variants");
}

#[tokio::test]
async fn test_small_image_no_upscaling() {
    let (db, temp_dir) = setup_test_db().await;

    // Create a small image (smaller than smallest breakpoint)
    let img = create_test_image(400, 300, false);
    let temp_path = temp_dir.path().join("test_small.png");
    img.save_with_format(&temp_path, ImgFormat::Png).unwrap();

    let source = ImageSource::Local(temp_path);
    let options = ImageOptions::default();
    let html_options = HtmlOptions::default();

    let result = get_or_process_image(&source, options, html_options, &db).await;
    assert!(result.is_ok());

    let output = result.unwrap();

    // All variants should be <= original width
    for variant in &output.variants {
        assert!(variant.width <= 400, "Image should not be upscaled");
    }
}

#[tokio::test]
async fn test_html_generation_with_alt_text() {
    let (db, temp_dir) = setup_test_db().await;

    let img = create_test_image(800, 600, false);
    let temp_path = temp_dir.path().join("test_alt.png");
    img.save_with_format(&temp_path, ImgFormat::Png).unwrap();

    let source = ImageSource::Local(temp_path);
    let options = ImageOptions::default();
    let html_options = HtmlOptions {
        alt_text: Some("Test image description".to_string()),
        layout: LayoutMode::FullWidth,
        loading: Loading::Eager,
        decoding: Decoding::Sync,
        blur_placeholder: None,
    };

    let result = get_or_process_image(&source, options, html_options, &db).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.html.contains(r#"alt="Test image description""#));
    assert!(output.html.contains(r#"loading="eager""#));
    assert!(output.html.contains(r#"decoding="sync""#));
}

#[tokio::test]
async fn test_image_caching() {
    let (db, temp_dir) = setup_test_db().await;

    let img = create_test_image(640, 480, false);
    let temp_path = temp_dir.path().join("test_cache.png");
    img.save_with_format(&temp_path, ImgFormat::Png).unwrap();

    let source = ImageSource::Local(temp_path.clone());
    let options = ImageOptions::default();
    let html_options = HtmlOptions::default();

    // Process first time - should create cache entry
    let result1 = get_or_process_image(&source, options.clone(), html_options.clone(), &db).await;
    assert!(result1.is_ok());

    // Process second time - should hit cache (though currently still processes)
    let result2 = get_or_process_image(&source, options, html_options, &db).await;
    assert!(result2.is_ok());

    // Both results should be identical
    let output1 = result1.unwrap();
    let output2 = result2.unwrap();
    assert_eq!(output1.resource_hash, output2.resource_hash);
    assert_eq!(output1.original_width, output2.original_width);
    assert_eq!(output1.original_height, output2.original_height);
}

#[tokio::test]
async fn test_quality_setting() {
    let (_db, temp_dir) = setup_test_db().await;

    let img_rgba = create_test_image(200, 200, false);
    let temp_path = temp_dir.path().join("test_quality.png");
    img_rgba.save_with_format(&temp_path, ImgFormat::Png).unwrap();

    // Test with different quality settings
    let options_low = ImageOptions {
        quality: 50,
        ..Default::default()
    };

    let options_high = ImageOptions {
        quality: 95,
        ..Default::default()
    };

    // Just verify they both work without errors
    let img_dynamic = image::open(&temp_path).unwrap();
    let result_low = lib::image::process_image(img_dynamic.clone(), options_low);
    assert!(result_low.is_ok());

    let result_high = lib::image::process_image(img_dynamic, options_high);
    assert!(result_high.is_ok());
}

#[test]
fn test_breakpoints_correct_order() {
    use lib::image::BREAKPOINTS;

    assert_eq!(BREAKPOINTS.len(), 5);

    // Verify ascending order
    for i in 0..BREAKPOINTS.len() - 1 {
        assert!(BREAKPOINTS[i].1 < BREAKPOINTS[i + 1].1);
    }

    // Verify specific breakpoints
    assert_eq!(BREAKPOINTS[0].1, 640);  // sm
    assert_eq!(BREAKPOINTS[1].1, 768);  // md
    assert_eq!(BREAKPOINTS[2].1, 1024); // lg
    assert_eq!(BREAKPOINTS[3].1, 1280); // xl
    assert_eq!(BREAKPOINTS[4].1, 1536); // xxl
}
