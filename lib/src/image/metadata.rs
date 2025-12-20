use crate::error::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Image metadata extracted from EXIF
#[derive(Debug, Clone, Default)]
pub struct ImageMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub focal_length: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub iso: Option<String>,
    pub date_taken: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub copyright: Option<String>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub custom: HashMap<String, String>,
}

impl ImageMetadata {
    /// Generate alt text from metadata
    pub fn to_alt_text(&self) -> Option<String> {
        // Prioritize description, then concatenate keywords
        if let Some(desc) = &self.description {
            Some(desc.clone())
        } else if !self.keywords.is_empty() {
            Some(self.keywords.join(", "))
        } else {
            None
        }
    }
}

/// Extract EXIF metadata from an image file
pub fn extract_metadata(path: &Path) -> Result<ImageMetadata> {
    let file = File::open(path).ok();
    if file.is_none() {
        return Ok(ImageMetadata::default());
    }

    let file = file.unwrap();
    let mut buf_reader = BufReader::new(file);
    let exif_reader = exif::Reader::new();
    let exif = exif_reader.read_from_container(&mut buf_reader).ok();

    if exif.is_none() {
        return Ok(ImageMetadata::default());
    }

    let exif = exif.unwrap();
    let mut metadata = ImageMetadata::default();

    // Extract common fields
    if let Some(field) = exif.get_field(exif::Tag::Make, exif::In::PRIMARY) {
        metadata.camera_make = Some(field.display_value().to_string());
    }

    if let Some(field) = exif.get_field(exif::Tag::Model, exif::In::PRIMARY) {
        metadata.camera_model = Some(field.display_value().to_string());
    }

    if let Some(field) = exif.get_field(exif::Tag::LensModel, exif::In::PRIMARY) {
        metadata.lens_model = Some(field.display_value().to_string());
    }

    if let Some(field) = exif.get_field(exif::Tag::FocalLength, exif::In::PRIMARY) {
        metadata.focal_length = Some(field.display_value().to_string());
    }

    if let Some(field) = exif.get_field(exif::Tag::FNumber, exif::In::PRIMARY) {
        metadata.aperture = Some(field.display_value().to_string());
    }

    if let Some(field) = exif.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY) {
        metadata.shutter_speed = Some(field.display_value().to_string());
    }

    if let Some(field) = exif.get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY) {
        metadata.iso = Some(field.display_value().to_string());
    }

    if let Some(field) = exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY) {
        metadata.date_taken = Some(field.display_value().to_string());
    }

    if let Some(field) = exif.get_field(exif::Tag::ImageDescription, exif::In::PRIMARY) {
        metadata.description = Some(field.display_value().to_string());
    }

    if let Some(field) = exif.get_field(exif::Tag::Copyright, exif::In::PRIMARY) {
        metadata.copyright = Some(field.display_value().to_string());
    }

    // Extract GPS coordinates if available
    if let (Some(lat_field), Some(lon_field)) = (
        exif.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY),
        exif.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY),
    ) {
        // Parse GPS coordinates (simplified - would need proper parsing in production)
        let lat_str = lat_field.display_value().to_string();
        let lon_str = lon_field.display_value().to_string();

        // For now, just store as strings in custom
        metadata.custom.insert("gps_latitude".to_string(), lat_str);
        metadata.custom.insert("gps_longitude".to_string(), lon_str);
    }

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_default() {
        let metadata = ImageMetadata::default();
        assert!(metadata.camera_make.is_none());
        assert!(metadata.keywords.is_empty());
    }

    #[test]
    fn test_to_alt_text_from_description() {
        let metadata = ImageMetadata {
            description: Some("A beautiful sunset".to_string()),
            ..Default::default()
        };
        assert_eq!(metadata.to_alt_text(), Some("A beautiful sunset".to_string()));
    }

    #[test]
    fn test_to_alt_text_from_keywords() {
        let metadata = ImageMetadata {
            keywords: vec!["sunset".to_string(), "beach".to_string()],
            ..Default::default()
        };
        assert_eq!(metadata.to_alt_text(), Some("sunset, beach".to_string()));
    }

    #[test]
    fn test_to_alt_text_none() {
        let metadata = ImageMetadata::default();
        assert_eq!(metadata.to_alt_text(), None);
    }

    #[test]
    fn test_extract_metadata_nonexistent() {
        let path = Path::new("/nonexistent/image.jpg");
        let result = extract_metadata(path);
        assert!(result.is_ok());
        // Should return default metadata for nonexistent files
    }
}
