# The image Crate

The de-facto standard for image processing in Rust. Pure Rust, memory-safe, with comprehensive format support and a mature API.

## Installation

```toml
[dependencies]
image = "0.25"

# Minimal build (specific formats only)
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }
```

## Supported Formats

| Format | Decode | Encode | Notes |
|--------|--------|--------|-------|
| AVIF | Yes | Yes | Requires nightly for encoding |
| BMP | Yes | Yes | RLE compression support |
| DDS | Yes | No | DXT1/3/5 compression |
| EXR | Yes | Yes | OpenEXR, no DWA compression |
| Farbfeld | Yes | Yes | Lossless |
| GIF | Yes | Yes | Animation support |
| HDR | Yes | Yes | Radiance HDR (RGB32F) |
| ICO | Yes | Yes | Multi-image support |
| JPEG | Yes | Yes | Uses zune-jpeg backend (v0.25+) |
| PNG | Yes | Yes | APNG support, pure Rust |
| PNM | Yes | Yes | PBM, PGM, PPM, PAM |
| QOI | Yes | Yes | Quite OK Image |
| TGA | Yes | Yes | RLE compression |
| TIFF | Yes | Yes | LZW, PackBits, Deflate |
| WebP | Yes | Yes | Via image-webp feature |

## Core API

### Loading Images

```rust
use image::{ImageReader, DynamicImage};

// Simple open
let img: DynamicImage = image::open("photo.jpg")?;

// With format detection from reader
let img = ImageReader::open("photo.jpg")?.decode()?;

// From bytes with format hint
let img = image::load_from_memory_with_format(
    &bytes,
    image::ImageFormat::Jpeg
)?;

// Just get dimensions (fast, no full decode)
let (width, height) = image::image_dimensions("large.png")?;
```

### Saving Images

```rust
// Format from extension
img.save("output.png")?;

// Explicit format
img.save_with_format("output", image::ImageFormat::Jpeg)?;

// To bytes
let mut bytes: Vec<u8> = Vec::new();
img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
```

### Type Conversions

```rust
let dyn_img: DynamicImage = image::open("photo.jpg")?;

// Convert to specific pixel type
let rgba8 = dyn_img.to_rgba8();      // ImageBuffer<Rgba<u8>, Vec<u8>>
let rgb8 = dyn_img.to_rgb8();
let luma8 = dyn_img.to_luma8();      // Grayscale
let rgba16 = dyn_img.to_rgba16();    // 16-bit
```

### Image Operations

```rust
use image::imageops::{self, FilterType};

// Resize (maintains aspect ratio)
let resized = img.resize(800, 600, FilterType::Lanczos3);

// Resize exact (may distort)
let resized = img.resize_exact(800, 600, FilterType::CatmullRom);

// Thumbnail (fast, maintains aspect)
let thumb = img.thumbnail(150, 150);

// Crop (x, y, width, height)
let cropped = img.crop_imm(100, 100, 400, 300);

// Transformations
let rotated = img.rotate90();
let flipped = img.fliph();
let blurred = img.blur(2.5);
let gray = img.grayscale();
let bright = img.brighten(20);
let contrast = img.adjust_contrast(10.0);
```

### Direct Pixel Access

```rust
use image::{Rgb, RgbImage};

let mut img: RgbImage = RgbImage::new(100, 100);

// Set pixel
img.put_pixel(10, 10, Rgb([255, 0, 0]));

// Get pixel
let pixel = img.get_pixel(10, 10);

// Iterate pixels
for (x, y, pixel) in img.enumerate_pixels_mut() {
    *pixel = Rgb([x as u8, y as u8, 128]);
}
```

## Performance Tips

1. **Use feature flags** to exclude unused formats and reduce binary size
2. **Enable rayon** feature (default) for multi-threaded encoding
3. **Use `thumbnail()`** instead of `resize()` when quality is less critical
4. **Get dimensions first** with `image_dimensions()` before loading large files
5. **v0.25+ uses zune-jpeg** backend for JPEG (major performance improvement)

## PNG Performance (v0.25+)

The pure Rust PNG decoder is now:
- **1.8x faster** than C libpng on x86
- **1.5x faster** on ARM
- Uses fdeflate encoder (faster than libpng with better compression)

## Related

- [Batch processing with Rayon](./batch-processing.md)
- [Format conversion patterns](./format-conversion.md)
- [Comparison with zune-image](./image-vs-zune.md)
