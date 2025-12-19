# Image Metadata Handling

Working with EXIF, ICC profiles, and XMP metadata in Rust image processing.

## Overview

| Library | EXIF Read | EXIF Write | ICC Profile | XMP |
|---------|-----------|------------|-------------|-----|
| image crate | Basic | No | Yes (v0.25+) | Limited |
| zune-image | Raw bytes | Limited | Yes | No |
| kamadak-exif | Full parsing | No | No | No |
| libvips | Full | Full | Full | Full |

## kamadak-exif (Recommended for EXIF)

The standard crate for parsing EXIF metadata.

```toml
[dependencies]
kamadak-exif = "0.5"
```

### Reading EXIF from File

```rust
use std::fs::File;
use std::io::BufReader;
use kamadak_exif::{Reader, Tag, In};

let file = File::open("photo.jpg")?;
let mut bufreader = BufReader::new(file);
let exif = Reader::new().read_from_container(&mut bufreader)?;

// Get specific tag
if let Some(field) = exif.get_field(Tag::Model, In::PRIMARY) {
    println!("Camera: {}", field.display_value());
}

if let Some(field) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
    println!("Date: {}", field.display_value());
}

// Iterate all tags
for field in exif.fields() {
    println!("{}: {}", field.tag, field.display_value().with_unit(&exif));
}
```

### Common EXIF Tags

```rust
use kamadak_exif::Tag;

// Camera info
Tag::Make           // Camera manufacturer
Tag::Model          // Camera model
Tag::LensModel      // Lens used

// Capture settings
Tag::ExposureTime   // Shutter speed
Tag::FNumber        // Aperture
Tag::ISOSpeedRatings
Tag::FocalLength

// Date/time
Tag::DateTimeOriginal
Tag::DateTimeDigitized

// GPS
Tag::GPSLatitude
Tag::GPSLongitude
Tag::GPSAltitude

// Image
Tag::ImageWidth
Tag::ImageLength
Tag::Orientation    // Rotation flag
```

## Integration with zune-image

zune-image provides raw EXIF bytes; use kamadak-exif to parse:

```rust
use std::io::Cursor;
use zune_image::image::Image;
use kamadak_exif::{Reader, Tag, In};

let mut img = Image::open("photo.jpg")?;
img.decode_headers()?;  // Fast - only headers, no pixel decode

if let Some(raw_exif) = img.metadata().exif() {
    let mut cursor = Cursor::new(raw_exif);
    let exif = Reader::new().read_from_container(&mut cursor)?;

    if let Some(field) = exif.get_field(Tag::Model, In::PRIMARY) {
        println!("Camera: {}", field.display_value());
    }
}

// ICC Profile
if let Some(icc) = img.metadata().icc_profile() {
    println!("ICC Profile: {} bytes", icc.len());
}
```

### Writing Metadata with zune-image

```rust
use zune_image::image::Image;
use zune_core::metadata::ImageMetadata;

let mut img = Image::open("input.png")?;

let mut metadata = ImageMetadata::default();
metadata.set_exif(exif_bytes);  // Your EXIF byte array

img.set_metadata(metadata);
img.save("output.jpg")?;  // Encoder attempts to embed
```

### Stripping Metadata

```rust
// For web optimization - remove all EXIF
img.metadata_mut().set_exif(vec![]);
img.save("stripped.jpg")?;
```

## image Crate Metadata

The image crate (v0.25+) has improved metadata support:

```rust
use image::ImageReader;

// Get dimensions without full decode
let (width, height) = image::image_dimensions("large.png")?;

// Access decoder metadata
let reader = ImageReader::open("photo.jpg")?;
let decoder = reader.into_decoder()?;

// ICC profile (if supported by format)
if let Some(icc) = decoder.icc_profile()? {
    println!("ICC: {} bytes", icc.len());
}
```

## Auto-Rotation Based on EXIF

Many cameras store images rotated with an orientation flag:

```rust
use kamadak_exif::{Reader, Tag, In};
use image::DynamicImage;

fn auto_rotate(img: DynamicImage, exif: &kamadak_exif::Exif) -> DynamicImage {
    let orientation = exif
        .get_field(Tag::Orientation, In::PRIMARY)
        .and_then(|f| f.value.get_uint(0))
        .unwrap_or(1);

    match orientation {
        1 => img,                           // Normal
        2 => img.fliph(),                   // Mirrored
        3 => img.rotate180(),               // Upside down
        4 => img.flipv(),                   // Mirrored + upside down
        5 => img.rotate90().fliph(),        // Rotated 90 CW + mirrored
        6 => img.rotate90(),                // Rotated 90 CW
        7 => img.rotate270().fliph(),       // Rotated 90 CCW + mirrored
        8 => img.rotate270(),               // Rotated 90 CCW
        _ => img,
    }
}
```

## Metadata Preservation Across Formats

**Warning:** Metadata is often lost during format conversion.

| Conversion | EXIF | ICC | XMP |
|------------|------|-----|-----|
| JPEG -> JPEG | Preserved* | Preserved* | Preserved* |
| JPEG -> PNG | Often lost | May preserve | Lost |
| PNG -> JPEG | N/A | May preserve | N/A |
| Any -> WebP | Usually lost | Lost | Lost |

*Depends on encoder settings.

### Manual Metadata Preservation

```rust
// 1. Extract before processing
let original_exif = extract_exif("input.jpg")?;

// 2. Process image
let img = image::open("input.jpg")?;
let processed = img.resize(800, 600, FilterType::Lanczos3);

// 3. Save, then reattach metadata (format-specific)
processed.save("output.jpg")?;
reattach_exif("output.jpg", &original_exif)?;  // Custom function
```

## Color Profiles (ICC)

For color-accurate processing:

```rust
// Check if image has embedded profile
if let Some(icc) = img.metadata().icc_profile() {
    // Use lcms2 or little-cms crates for color management
    // Convert to working space (sRGB) before processing
    // Re-embed profile on save
}
```

## Related

- [zune-image](./zune-image.md)
- [image crate](./image-crate.md)
- [Format conversion](./format-conversion.md)
