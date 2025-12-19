# Format Conversion

Converting between image formats in Rust with both `image` and `zune-image` crates.

## image Crate

### Basic Conversion

```rust
use image::{ImageFormat, ImageReader};

// Load any supported format, save as another
let img = image::open("input.jpg")?;
img.save("output.png")?;  // Format from extension

// Explicit format (no extension guessing)
img.save_with_format("output", ImageFormat::WebP)?;
```

### To Memory Buffer

```rust
use std::io::Cursor;
use image::ImageFormat;

let img = image::open("photo.jpg")?;

let mut buffer: Vec<u8> = Vec::new();
img.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)?;
```

### Supported Conversions

All 15+ formats can decode to `DynamicImage`, then encode to any writable format:

| From | To (common) |
|------|-------------|
| JPEG, PNG, WebP, GIF, BMP, TIFF | JPEG, PNG, WebP, GIF, BMP, TIFF |
| AVIF, EXR, HDR, QOI | Most formats (with color depth conversion) |
| DDS, ICO, PNM, TGA | Any writable format |

### Color Type Handling

```rust
let img = image::open("input.png")?;

// Convert to specific color type before saving
let rgb8 = img.to_rgb8();   // Drop alpha
let rgba8 = img.to_rgba8(); // Ensure alpha
let gray = img.to_luma8();  // Grayscale

// Save with converted type
rgb8.save("output.jpg")?;  // JPEG doesn't support alpha
```

## zune-image

### Basic Conversion

```rust
use zune_image::image::Image;

// Format auto-detected on load, determined by extension on save
let img = Image::open("photo.jpg")?;
img.save("converted.png")?;
```

### Supported Formats

| Format | Decode | Encode |
|--------|--------|--------|
| JPEG | Yes | Yes |
| PNG | Yes | Yes |
| JPEG-XL | Yes | Yes |
| PPM/PGM/PAM | Yes | Yes |
| QOI | Yes | Yes |
| Farbfeld | Yes | Yes |
| HDR | Yes | Yes |
| PSD | Yes | No |
| BMP | Yes | No |

**Gaps:** No WebP, GIF, TIFF, AVIF encode/decode.

## HDR Conversion Considerations

### Bit Depth Preservation

```rust
// zune-image preserves depth
let hdr_img = Image::open("scene.hdr")?;  // f32 internally
// Operations maintain precision until final encode

// image crate
let hdr = image::open("scene.hdr")?;
let rgb32f = hdr.to_rgb32f();  // Access HDR data
```

### Converting HDR to LDR

```rust
// Tone mapping happens implicitly when saving to LDR format
let hdr = image::open("scene.hdr")?;
hdr.save("output.jpg")?;  // Clamped to 0-255
```

## Common Conversion Patterns

### JPEG to PNG (lossless preservation)

```rust
let img = image::open("photo.jpg")?;
img.save("photo.png")?;
// Note: Already-lossy JPEG artifacts are preserved, not fixed
```

### PNG to JPEG (drop alpha, add compression)

```rust
let img = image::open("transparent.png")?;
let rgb = img.to_rgb8();  // Remove alpha
rgb.save("opaque.jpg")?;
```

### Any to WebP

```rust
use image::ImageFormat;

let img = image::open("input.bmp")?;
img.save_with_format("output.webp", ImageFormat::WebP)?;
```

### Batch Format Conversion

```rust
use rayon::prelude::*;
use std::path::Path;

let files: Vec<_> = get_jpeg_files();

files.par_iter().for_each(|path| {
    if let Ok(img) = image::open(path) {
        let out = path.with_extension("webp");
        let _ = img.save(&out);
    }
});
```

## Metadata Considerations

**Important:** Format conversion may not preserve metadata:

- EXIF: May be lost when converting from JPEG to PNG
- ICC Profiles: Some encoders preserve, others don't
- XMP/IPTC: Usually lost in conversion

To preserve metadata, extract before conversion and reattach:

```rust
// With kamadak-exif + image crate
// 1. Read EXIF from source
// 2. Convert image
// 3. Write EXIF to destination (format-specific)
```

See [Metadata handling](./metadata.md) for details.

## Related

- [image crate](./image-crate.md)
- [zune-image](./zune-image.md)
- [Metadata handling](./metadata.md)
