use crate::error::Result;
use crate::image::{ImageVariant, ImageFormat};
use std::collections::HashMap;

/// Layout mode for responsive images
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Full width of container
    FullWidth,
    /// Fixed width in pixels
    Fixed(u32),
    /// Percentage of container width
    Percentage(u8),
    /// Auto-detect based on image dimensions
    Auto,
}

impl Default for LayoutMode {
    fn default() -> Self {
        LayoutMode::Auto
    }
}

/// Options for HTML generation
#[derive(Debug, Clone)]
pub struct HtmlOptions {
    pub layout: LayoutMode,
    pub alt_text: Option<String>,
    pub loading: Loading,
    pub decoding: Decoding,
    pub blur_placeholder: Option<String>,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            layout: LayoutMode::Auto,
            alt_text: None,
            loading: Loading::Lazy,
            decoding: Decoding::Async,
            blur_placeholder: None,
        }
    }
}

/// Image loading strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loading {
    Eager,
    Lazy,
}

impl Loading {
    fn as_str(&self) -> &'static str {
        match self {
            Loading::Eager => "eager",
            Loading::Lazy => "lazy",
        }
    }
}

/// Image decoding strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decoding {
    Sync,
    Async,
    Auto,
}

impl Decoding {
    fn as_str(&self) -> &'static str {
        match self {
            Decoding::Sync => "sync",
            Decoding::Async => "async",
            Decoding::Auto => "auto",
        }
    }
}

/// Generate the `sizes` attribute value based on layout mode
fn generate_sizes_attribute(layout: LayoutMode, max_width: u32) -> String {
    match layout {
        LayoutMode::FullWidth => "100vw".to_string(),
        LayoutMode::Fixed(width) => format!("{}px", width),
        LayoutMode::Percentage(pct) => format!("{}vw", pct),
        LayoutMode::Auto => {
            // Use responsive sizes based on common breakpoints
            if max_width >= 1536 {
                "(min-width: 1536px) 1536px, (min-width: 1280px) 1280px, (min-width: 1024px) 1024px, (min-width: 768px) 768px, (min-width: 640px) 640px, 100vw".to_string()
            } else if max_width >= 1280 {
                "(min-width: 1280px) 1280px, (min-width: 1024px) 1024px, (min-width: 768px) 768px, (min-width: 640px) 640px, 100vw".to_string()
            } else if max_width >= 1024 {
                "(min-width: 1024px) 1024px, (min-width: 768px) 768px, (min-width: 640px) 640px, 100vw".to_string()
            } else if max_width >= 768 {
                "(min-width: 768px) 768px, (min-width: 640px) 640px, 100vw".to_string()
            } else {
                "100vw".to_string()
            }
        }
    }
}

/// Group variants by format
fn group_by_format(variants: &[ImageVariant]) -> HashMap<ImageFormat, Vec<&ImageVariant>> {
    let mut grouped: HashMap<ImageFormat, Vec<&ImageVariant>> = HashMap::new();

    for variant in variants {
        grouped.entry(variant.format).or_default().push(variant);
    }

    // Sort each group by width (ascending)
    for variants in grouped.values_mut() {
        variants.sort_by_key(|v| v.width);
    }

    grouped
}

/// Generate srcset attribute for a list of variants
fn generate_srcset(variants: &[&ImageVariant]) -> String {
    variants
        .iter()
        .map(|v| {
            // For now, use inline data URIs (in production, these would be file paths)
            let data_uri = format!(
                "data:{};base64,{}",
                v.format.mime_type(),
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &v.data)
            );
            format!("{} {}w", data_uri, v.width)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Generate a <picture> element with srcset
pub fn generate_picture_html(variants: &[ImageVariant], options: HtmlOptions) -> Result<String> {
    if variants.is_empty() {
        return Ok(String::new());
    }

    let grouped = group_by_format(variants);

    // Find max width for sizes attribute
    let max_width = variants.iter().map(|v| v.width).max().unwrap_or(0);

    // Generate sizes attribute
    let sizes = generate_sizes_attribute(options.layout, max_width);

    // Build <picture> element
    let mut html = String::from("<picture>");

    // Add source elements in order of preference (AVIF, WebP, JPEG/PNG)
    let format_order = [
        ImageFormat::Avif,
        ImageFormat::WebP,
        ImageFormat::Jpeg,
        ImageFormat::Png,
    ];

    for format in &format_order {
        if let Some(format_variants) = grouped.get(format) {
            if !format_variants.is_empty() {
                let srcset = generate_srcset(format_variants);
                html.push_str(&format!(
                    r#"<source type="{}" srcset="{}" sizes="{}">"#,
                    format.mime_type(),
                    srcset,
                    sizes
                ));
            }
        }
    }

    // Add fallback <img> tag
    let fallback = variants.first().unwrap();
    let fallback_src = format!(
        "data:{};base64,{}",
        fallback.format.mime_type(),
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &fallback.data)
    );

    let alt = options.alt_text.unwrap_or_else(|| String::from(""));
    let loading = options.loading.as_str();
    let decoding = options.decoding.as_str();

    html.push_str(&format!(
        r#"<img src="{}" alt="{}" width="{}" height="{}" loading="{}" decoding="{}">"#,
        fallback_src, alt, fallback.width, fallback.height, loading, decoding
    ));

    html.push_str("</picture>");

    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_variant(width: u32, format: ImageFormat) -> ImageVariant {
        ImageVariant {
            width,
            height: width / 2,
            format,
            data: vec![0u8; 100], // Dummy data
            size_bytes: 100,
        }
    }

    #[test]
    fn test_generate_sizes_full_width() {
        let sizes = generate_sizes_attribute(LayoutMode::FullWidth, 1024);
        assert_eq!(sizes, "100vw");
    }

    #[test]
    fn test_generate_sizes_fixed() {
        let sizes = generate_sizes_attribute(LayoutMode::Fixed(500), 1024);
        assert_eq!(sizes, "500px");
    }

    #[test]
    fn test_generate_sizes_percentage() {
        let sizes = generate_sizes_attribute(LayoutMode::Percentage(75), 1024);
        assert_eq!(sizes, "75vw");
    }

    #[test]
    fn test_generate_sizes_auto() {
        let sizes = generate_sizes_attribute(LayoutMode::Auto, 1536);
        assert!(sizes.contains("1536px"));
        assert!(sizes.contains("100vw"));
    }

    #[test]
    fn test_group_by_format() {
        let variants = vec![
            create_test_variant(640, ImageFormat::Jpeg),
            create_test_variant(1024, ImageFormat::Jpeg),
            create_test_variant(640, ImageFormat::WebP),
        ];

        let grouped = group_by_format(&variants);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get(&ImageFormat::Jpeg).unwrap().len(), 2);
        assert_eq!(grouped.get(&ImageFormat::WebP).unwrap().len(), 1);
    }

    #[test]
    fn test_generate_picture_html() {
        let variants = vec![
            create_test_variant(640, ImageFormat::Jpeg),
            create_test_variant(640, ImageFormat::WebP),
        ];

        let options = HtmlOptions {
            alt_text: Some("Test image".to_string()),
            ..Default::default()
        };

        let result = generate_picture_html(&variants, options);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("<picture>"));
        assert!(html.contains("</picture>"));
        assert!(html.contains("<img"));
        assert!(html.contains(r#"alt="Test image""#));
        assert!(html.contains("loading=\"lazy\""));
    }

    #[test]
    fn test_generate_picture_html_empty() {
        let variants = vec![];
        let options = HtmlOptions::default();
        let result = generate_picture_html(&variants, options);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_loading_as_str() {
        assert_eq!(Loading::Eager.as_str(), "eager");
        assert_eq!(Loading::Lazy.as_str(), "lazy");
    }

    #[test]
    fn test_decoding_as_str() {
        assert_eq!(Decoding::Sync.as_str(), "sync");
        assert_eq!(Decoding::Async.as_str(), "async");
        assert_eq!(Decoding::Auto.as_str(), "auto");
    }
}
