# zune-image

A high-performance, modular image processing library designed for speed. Often matches or exceeds libjpeg-turbo performance with SIMD-accelerated routines.

## Installation

```toml
[dependencies]
zune-image = "0.4"
zune-imageprocs = "0.4"  # For resize, filters, etc.
zune-core = "0.4"        # For colorspace, metadata types
```

## Supported Formats

| Format | Decoder | Encoder | no_std |
|--------|---------|---------|--------|
| JPEG | zune-jpeg | jpeg-encoder | Yes |
| PNG | zune-png | zune-png | Yes |
| JPEG-XL | jxl-oxide | zune-jpegxl | Yes* |
| PPM/PGM/PAM | zune-ppm | zune-ppm | Yes |
| QOI | zune-qoi | zune-qoi | Yes |
| Farbfeld | zune-farbfeld | zune-farbfeld | Yes |
| PSD | zune-psd | - | Yes |
| HDR | zune-hdr | zune-hdr | No |
| BMP | zune-bmp | - | - |

**Notable gaps:** No WebP, GIF, TIFF, or AVIF support.

## Core API

### Loading and Saving

```rust
use zune_image::image::Image;

// Load (auto-detects format from header)
let img = Image::open("photo.jpg")?;

// Save (format from extension)
img.save("output.png")?;

// From bytes
let img = Image::read(&bytes, DecoderOptions::default())?;
```

### Resizing

```rust
use zune_image::image::Image;
use zune_imageprocs::resize::{Resize, ResizeMethod};
use zune_image::traits::OperationsTrait;

let mut img = Image::open("input.jpg")?;

// Define resize operation
let resize_op = Resize::new(800, 600, ResizeMethod::Bilinear);

// Execute (mutates in-place)
resize_op.execute(&mut img)?;

img.save("resized.png")?;
```

### Resize Methods

| Method | Speed | Quality | Use Case |
|--------|-------|---------|----------|
| Nearest | Fastest | Low | Drafts, pixel art |
| Bilinear | Fast | Medium | Web thumbnails |
| Bicubic | Moderate | High | General purpose |
| Lanczos3 | Slow | Highest | Maximum sharpness |

### Format Conversion

```rust
use zune_image::image::Image;

// Simply load and save with different extension
let img = Image::open("photo.jpg")?;
img.save("converted.png")?;  // Encoder selected from extension
```

## HDR Support

zune-image natively supports `u8`, `u16`, and `f32` image depths:

```rust
// Check colorspace
let colorspace = img.colorspace();

// Common colorspaces
// RGB - Standard for screens
// RGBA - With alpha channel
// YCbCr - Common in JPEGs (may skip RGB conversion for speed)
```

## Metadata Handling

### Reading Metadata

```rust
use zune_image::image::Image;

let mut img = Image::open("photo.jpg")?;

// Decode only headers (fast, no pixel decode)
img.decode_headers()?;

// Access EXIF bytes
if let Some(exif) = img.metadata().exif() {
    println!("EXIF: {} bytes", exif.len());
    // Parse with kamadak-exif crate
}

// Access ICC profile
if let Some(icc) = img.metadata().icc_profile() {
    println!("ICC profile: {} bytes", icc.len());
}
```

### Parsing EXIF with kamadak-exif

```toml
[dependencies]
zune-image = "0.4"
kamadak-exif = "0.5"
```

```rust
use std::io::Cursor;
use zune_image::image::Image;
use kamadak_exif::{Reader, Tag, In};

let mut img = Image::open("photo.jpg")?;
img.decode_headers()?;

if let Some(raw_exif) = img.metadata().exif() {
    let mut cursor = Cursor::new(raw_exif);
    let exif = Reader::new().read_from_container(&mut cursor)?;

    // Get specific tag
    if let Some(field) = exif.get_field(Tag::Model, In::PRIMARY) {
        println!("Camera: {}", field.display_value());
    }

    // Iterate all fields
    for field in exif.fields() {
        println!("{}: {}", field.tag, field.display_value().with_unit(&exif));
    }
}
```

### Writing Metadata

```rust
use zune_image::image::Image;
use zune_core::metadata::ImageMetadata;

let mut img = Image::open("input.png")?;

let mut metadata = ImageMetadata::default();
metadata.set_exif(exif_bytes);

img.set_metadata(metadata);
img.save("output.jpg")?;  // Encoder attempts to write metadata
```

### Stripping Metadata

```rust
// Clear EXIF for web optimization
img.metadata_mut().set_exif(vec![]);
```

## Performance Characteristics

- **SIMD optimized:** Uses intrinsics for IDCT, alpha pre-multiplication
- **Modular:** Only linked formats are compiled into binary
- **no_std support:** Most decoders work in embedded environments
- **Thread-safe:** Decoders designed for parallel batch processing

## Key Differences from image Crate

| Aspect | zune-image | image crate |
|--------|------------|-------------|
| Format breadth | 8+ focused | 15+ comprehensive |
| PNG speed | 1.7-3.5x faster | Uses fdeflate |
| Binary size | Smaller (modular) | Larger |
| HDR pipeline | Full u8/u16/f32 | Decode/encode only |
| API maturity | Evolving | Very mature |
| Ecosystem | Emerging | De-facto standard |

## When to Choose zune-image

- Maximum JPEG/PNG decode performance is critical
- Building minimal binaries (embedded, WASM)
- HDR image processing workflows
- Batch processing where per-image speed matters
- Don't need WebP, GIF, or TIFF support

## Related

- [Batch processing with Rayon](./batch-processing.md)
- [Comparison with image crate](./image-vs-zune.md)
