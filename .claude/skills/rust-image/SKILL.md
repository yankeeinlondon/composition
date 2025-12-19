---
name: rust-image
description: Expert knowledge for image processing in Rust using the image crate, zune-image, and alternatives. Use when loading, saving, resizing, converting formats, batch processing with Rayon, or handling image metadata in Rust applications.
last_updated: 2025-12-18T20:30:00Z
hash: 022f91cf777899b2
---

# Rust Image Processing

Expert guidance for image manipulation in Rust covering library selection, format support, performance optimization, and batch processing.

## Library Selection Decision Tree

| Need | Best Choice | Why |
|------|-------------|-----|
| General purpose, broad format support | `image` crate | Ecosystem standard, 15+ formats, stable API |
| Maximum JPEG/PNG performance | `zune-image` | 2-4x faster decoding, SIMD optimized |
| High-throughput server pipeline | `libvips` via FFI | Streaming architecture, minimal RAM |
| WASM/browser processing | `photon` | WASM-first, 80+ built-in effects |
| Only resizing needed | `fast_image_resize` | Fastest SIMD resize, gamma-correct |
| 2D drawing/compositing | `tiny-skia` | Porter-Duff blending, anti-aliasing |

## Core Principles

- Prefer `image` crate for most projects (ecosystem compatibility, safety, battle-tested)
- Use `zune-image` when JPEG/PNG performance is critical or binary size matters
- `image` v0.25+ uses zune-jpeg backend internally (performance convergence)
- Enable `rayon` feature for multi-threaded encoding/decoding
- Use feature flags to minimize binary size: `default-features = false`
- For batch processing, combine with Rayon's `par_iter()` for file-level parallelism
- Check colorspace before processing (RGB vs YCbCr vs RGBA)
- Metadata is not preserved by default across format conversions

## Quick Start

### image crate

```toml
[dependencies]
image = "0.25"
```

```rust
use image::{DynamicImage, ImageFormat, imageops::FilterType};

// Load and save
let img = image::open("input.jpg")?;
img.save("output.png")?;

// Resize with quality filter
let resized = img.resize(800, 600, FilterType::Lanczos3);

// Convert to specific pixel type
let rgba = img.to_rgba8();
```

### zune-image

```toml
[dependencies]
zune-image = "0.4"
zune-imageprocs = "0.4"
```

```rust
use zune_image::image::Image;
use zune_imageprocs::resize::{Resize, ResizeMethod};

let mut img = Image::open("input.jpg")?;
let resize_op = Resize::new(800, 600, ResizeMethod::Bilinear);
resize_op.execute(&mut img)?;
img.save("output.png")?;
```

## Topics

### Library Deep Dives

- [image crate](./image-crate.md) - The ecosystem standard
- [zune-image](./zune-image.md) - High-performance alternative
- [Other alternatives](./alternatives.md) - Photon, fast_image_resize, tiny-skia, Rust CV

### Common Tasks

- [Format conversion](./format-conversion.md) - Converting between image formats
- [Batch processing](./batch-processing.md) - Parallel processing with Rayon
- [Metadata handling](./metadata.md) - EXIF, ICC profiles, XMP

### Architecture Decisions

- [image vs zune-image](./image-vs-zune.md) - When to choose which
- [image vs libvips](./image-vs-libvips.md) - Native Rust vs C FFI

## Resize Filter Selection

| Filter | Speed | Quality | Use Case |
|--------|-------|---------|----------|
| Nearest | Fastest | Low | Pixel art, drafts |
| Triangle/Bilinear | Fast | Medium | Web thumbnails |
| CatmullRom | Moderate | High | General purpose |
| Lanczos3 | Slow | Highest | Maximum sharpness, downscaling |

## Resources

- [image crate docs](https://docs.rs/image/)
- [image GitHub](https://github.com/image-rs/image)
- [zune-image docs](https://docs.rs/zune-image/)
- [zune-image GitHub](https://github.com/etemesi254/zune-image)
