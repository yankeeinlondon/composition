use crate::error::{CompositionError, Result};
use image::DynamicImage;
use std::path::{Path, PathBuf};
use std::fs;

/// Source of an image (local file or remote URL)
#[derive(Debug, Clone)]
pub enum ImageSource {
    Local(PathBuf),
    Remote(String),
}

impl ImageSource {
    /// Create from a string (auto-detect local vs remote)
    pub fn from_str(s: &str) -> Self {
        if s.starts_with("http://") || s.starts_with("https://") {
            ImageSource::Remote(s.to_string())
        } else {
            ImageSource::Local(PathBuf::from(s))
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        match self {
            ImageSource::Local(path) => path.to_str().unwrap_or(""),
            ImageSource::Remote(url) => url,
        }
    }
}

/// Load an image from a source (local or remote)
pub fn load_image(source: &ImageSource) -> Result<DynamicImage> {
    match source {
        ImageSource::Local(path) => load_local_image(path),
        ImageSource::Remote(url) => load_remote_image(url),
    }
}

/// Load a local image file
fn load_local_image(path: &Path) -> Result<DynamicImage> {
    if !path.exists() {
        return Err(CompositionError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Image file not found: {}", path.display()),
        )));
    }

    let bytes = fs::read(path).map_err(|e| CompositionError::Io(e))?;

    image::load_from_memory(&bytes).map_err(|e| {
        CompositionError::Render(crate::error::RenderError::ImageProcessing(
            format!("Failed to load image from {}: {}", path.display(), e)
        ))
    })
}

/// Load a remote image from a URL
fn load_remote_image(url: &str) -> Result<DynamicImage> {
    let response = reqwest::blocking::get(url).map_err(|e| {
        CompositionError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to fetch remote image: {}", e),
        ))
    })?;

    if !response.status().is_success() {
        return Err(CompositionError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("HTTP error fetching image: {}", response.status()),
        )));
    }

    let bytes = response.bytes().map_err(|e| {
        CompositionError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to read remote image bytes: {}", e),
        ))
    })?;

    image::load_from_memory(&bytes).map_err(|e| {
        CompositionError::Render(crate::error::RenderError::ImageProcessing(
            format!("Failed to decode remote image from {}: {}", url, e)
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_source_from_str_url() {
        let source = ImageSource::from_str("https://example.com/image.jpg");
        assert!(matches!(source, ImageSource::Remote(_)));
    }

    #[test]
    fn test_image_source_from_str_local() {
        let source = ImageSource::from_str("/path/to/image.jpg");
        assert!(matches!(source, ImageSource::Local(_)));
    }

    #[test]
    fn test_image_source_as_str() {
        let source = ImageSource::Remote("https://example.com/test.jpg".to_string());
        assert_eq!(source.as_str(), "https://example.com/test.jpg");

        let source = ImageSource::Local(PathBuf::from("/test.jpg"));
        assert_eq!(source.as_str(), "/test.jpg");
    }
}
