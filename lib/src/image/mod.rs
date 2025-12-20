mod source;
mod processing;
mod metadata;
pub mod html;
mod cache;

pub use source::{load_image, ImageSource};
pub use processing::{process_image, ImageOptions, ImageVariant, ImageFormat, detect_transparency};
pub use metadata::{extract_metadata, ImageMetadata};
pub use html::{generate_picture_html, LayoutMode};
pub use cache::get_or_process_image;

use crate::types::Breakpoint;

/// Tailwind CSS breakpoints for responsive images
pub const BREAKPOINTS: &[(Breakpoint, u32)] = &[
    (Breakpoint::Sm, 640),
    (Breakpoint::Md, 768),
    (Breakpoint::Lg, 1024),
    (Breakpoint::Xl, 1280),
    (Breakpoint::Xxl, 1536),
];

/// Output from smart image processing
#[derive(Debug, Clone)]
pub struct SmartImageOutput {
    pub resource_hash: String,
    pub original_width: u32,
    pub original_height: u32,
    pub has_transparency: bool,
    pub variants: Vec<ImageVariant>,
    pub blur_placeholder: String,  // base64 data URI
    pub html: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoints_ascending() {
        for i in 0..BREAKPOINTS.len() - 1 {
            assert!(BREAKPOINTS[i].1 < BREAKPOINTS[i + 1].1);
        }
    }
}
