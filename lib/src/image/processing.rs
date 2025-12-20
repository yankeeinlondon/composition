use crate::error::{CompositionError, Result};
use crate::image::BREAKPOINTS;
use image::{DynamicImage, ImageFormat as ImgFormat, GenericImageView};
use rayon::prelude::*;
use std::io::Cursor;
use tracing::{debug, instrument};

/// Format for image output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    Avif,
    WebP,
    Jpeg,
    Png,
}

impl ImageFormat {
    /// Get the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Avif => "image/avif",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Png => "image/png",
        }
    }

    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Avif => "avif",
            ImageFormat::WebP => "webp",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Png => "png",
        }
    }
}

/// A single image variant (specific width and format)
#[derive(Debug, Clone)]
pub struct ImageVariant {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub data: Vec<u8>,
    pub size_bytes: usize,
}

/// Options for image processing
#[derive(Debug, Clone)]
pub struct ImageOptions {
    /// Whether to strip metadata (default: true)
    pub strip_metadata: bool,
    /// Maximum width (no variants larger than this)
    pub max_width: Option<u32>,
    /// Quality for lossy formats (1-100, default: 85)
    pub quality: u8,
}

impl Default for ImageOptions {
    fn default() -> Self {
        Self {
            strip_metadata: true,
            max_width: None,
            quality: 85,
        }
    }
}

/// Detect if an image has transparency
pub fn detect_transparency(img: &DynamicImage) -> bool {
    match img {
        DynamicImage::ImageRgba8(rgba) => {
            // Check if any pixel has alpha < 255
            rgba.pixels().any(|p| p.0[3] < 255)
        }
        DynamicImage::ImageRgba16(rgba) => {
            rgba.pixels().any(|p| p.0[3] < 65535)
        }
        DynamicImage::ImageLumaA8(luma_a) => {
            luma_a.pixels().any(|p| p.0[1] < 255)
        }
        DynamicImage::ImageLumaA16(luma_a) => {
            luma_a.pixels().any(|p| p.0[1] < 65535)
        }
        // Other formats don't support transparency
        _ => false,
    }
}

/// Resize an image to a target width, maintaining aspect ratio
fn resize_image(img: &DynamicImage, target_width: u32) -> DynamicImage {
    let (orig_width, orig_height) = img.dimensions();

    // Don't upscale
    if target_width >= orig_width {
        return img.clone();
    }

    let target_height = (orig_height as f64 * target_width as f64 / orig_width as f64) as u32;

    img.resize_exact(
        target_width,
        target_height,
        image::imageops::FilterType::Lanczos3,
    )
}

/// Encode an image to a specific format
fn encode_image(img: &DynamicImage, format: ImageFormat, quality: u8) -> Result<Vec<u8>> {
    let mut buffer = Cursor::new(Vec::new());

    match format {
        ImageFormat::Png => {
            img.write_to(&mut buffer, ImgFormat::Png).map_err(|e| {
                CompositionError::Render(crate::error::RenderError::ImageProcessing(
                    format!("Failed to encode PNG: {}", e)
                ))
            })?;
        }
        ImageFormat::Jpeg => {
            let rgb = img.to_rgb8();
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality);
            encoder.encode(
                rgb.as_raw(),
                img.width(),
                img.height(),
                image::ExtendedColorType::Rgb8,
            ).map_err(|e| {
                CompositionError::Render(crate::error::RenderError::ImageProcessing(
                    format!("Failed to encode JPEG: {}", e)
                ))
            })?;
        }
        ImageFormat::WebP => {
            // WebP encoding requires webp feature
            // For now, fall back to PNG
            // TODO: Add proper WebP encoding
            img.write_to(&mut buffer, ImgFormat::Png).map_err(|e| {
                CompositionError::Render(crate::error::RenderError::ImageProcessing(
                    format!("Failed to encode WebP (using PNG fallback): {}", e)
                ))
            })?;
        }
        ImageFormat::Avif => {
            // AVIF encoding requires avif feature
            // For now, fall back to JPEG
            // TODO: Add proper AVIF encoding
            let rgb = img.to_rgb8();
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality);
            encoder.encode(
                rgb.as_raw(),
                img.width(),
                img.height(),
                image::ExtendedColorType::Rgb8,
            ).map_err(|e| {
                CompositionError::Render(crate::error::RenderError::ImageProcessing(
                    format!("Failed to encode AVIF (using JPEG fallback): {}", e)
                ))
            })?;
        }
    }

    Ok(buffer.into_inner())
}

/// Generate all format variants for a single width
fn generate_format_variants(
    img: &DynamicImage,
    has_transparency: bool,
    quality: u8,
) -> Result<Vec<ImageVariant>> {
    let formats = if has_transparency {
        // For images with transparency, use PNG and WebP (AVIF also supports transparency)
        vec![ImageFormat::Avif, ImageFormat::WebP, ImageFormat::Png]
    } else {
        // For opaque images, use all formats
        vec![ImageFormat::Avif, ImageFormat::WebP, ImageFormat::Jpeg]
    };

    formats
        .into_iter()
        .map(|format| {
            let data = encode_image(img, format, quality)?;
            let size_bytes = data.len();
            Ok(ImageVariant {
                width: img.width(),
                height: img.height(),
                format,
                data,
                size_bytes,
            })
        })
        .collect()
}

/// Generate a blur placeholder (tiny image encoded as base64 data URI)
pub fn generate_blur_placeholder(img: &DynamicImage, width: u32) -> Result<String> {
    let tiny = resize_image(img, width);
    let data = encode_image(&tiny, ImageFormat::Jpeg, 50)?;
    let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
    Ok(format!("data:image/jpeg;base64,{}", base64))
}

/// Process an image and generate all variants
#[instrument(skip(img), fields(width = img.width(), height = img.height()))]
pub fn process_image(
    img: DynamicImage,
    options: ImageOptions,
) -> Result<(Vec<ImageVariant>, bool, String)> {
    let (orig_width, _) = img.dimensions();

    // Detect transparency
    let has_transparency = detect_transparency(&img);
    debug!("Transparency detected: {}", has_transparency);

    // Determine which breakpoints to use
    let max_width = options.max_width.unwrap_or(orig_width);
    let widths: Vec<u32> = BREAKPOINTS
        .iter()
        .map(|(_, w)| *w)
        .filter(|w| *w <= max_width && *w <= orig_width)
        .collect();

    debug!("Processing {} breakpoint widths", widths.len());

    // Generate variants in parallel
    let variants: Vec<ImageVariant> = widths
        .par_iter()
        .map(|width| {
            let resized = resize_image(&img, *width);
            generate_format_variants(&resized, has_transparency, options.quality)
        })
        .collect::<Result<Vec<Vec<ImageVariant>>>>()?
        .into_iter()
        .flatten()
        .collect();

    debug!("Generated {} total variants", variants.len());

    // Generate blur placeholder
    let blur_placeholder = generate_blur_placeholder(&img, 20)?;

    Ok((variants, has_transparency, blur_placeholder))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbaImage, Rgba};

    fn create_test_image(width: u32, height: u32, has_alpha: bool) -> DynamicImage {
        let mut img = RgbaImage::new(width, height);
        for (_, _, pixel) in img.enumerate_pixels_mut() {
            *pixel = if has_alpha {
                Rgba([255, 0, 0, 128]) // Red with 50% transparency
            } else {
                Rgba([255, 0, 0, 255]) // Opaque red
            };
        }
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn test_detect_transparency_opaque() {
        let img = create_test_image(100, 100, false);
        assert!(!detect_transparency(&img));
    }

    #[test]
    fn test_detect_transparency_transparent() {
        let img = create_test_image(100, 100, true);
        assert!(detect_transparency(&img));
    }

    #[test]
    fn test_resize_no_upscale() {
        let img = create_test_image(100, 100, false);
        let resized = resize_image(&img, 200);
        assert_eq!(resized.width(), 100); // Should not upscale
    }

    #[test]
    fn test_resize_maintains_aspect_ratio() {
        let img = create_test_image(200, 100, false);
        let resized = resize_image(&img, 100);
        assert_eq!(resized.width(), 100);
        assert_eq!(resized.height(), 50);
    }

    #[test]
    fn test_image_format_mime_types() {
        assert_eq!(ImageFormat::Avif.mime_type(), "image/avif");
        assert_eq!(ImageFormat::WebP.mime_type(), "image/webp");
        assert_eq!(ImageFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(ImageFormat::Png.mime_type(), "image/png");
    }

    #[test]
    fn test_encode_jpeg() {
        let img = create_test_image(10, 10, false);
        let result = encode_image(&img, ImageFormat::Jpeg, 85);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_encode_png() {
        let img = create_test_image(10, 10, true);
        let result = encode_image(&img, ImageFormat::Png, 85);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_blur_placeholder() {
        let img = create_test_image(100, 100, false);
        let result = generate_blur_placeholder(&img, 20);
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("data:image/jpeg;base64,"));
    }

    #[test]
    fn test_process_image_generates_variants() {
        let img = create_test_image(2000, 1000, false);
        let options = ImageOptions::default();
        let result = process_image(img, options);
        assert!(result.is_ok());

        let (variants, has_transparency, blur) = result.unwrap();
        assert!(!has_transparency);
        assert!(blur.starts_with("data:image/jpeg;base64,"));
        assert!(!variants.is_empty());

        // Check that we don't upscale
        for variant in &variants {
            assert!(variant.width <= 2000);
        }
    }
}
