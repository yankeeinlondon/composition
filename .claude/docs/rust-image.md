---
name: rust-image
description: Comprehensive guide to image processing in Rust - comparing the image crate, zune-image, libvips/Sharp integration, and alternative libraries
created: 2025-12-18
last_updated: 2025-12-18T16:30:00Z
hash: 4f26e7053fd6a061
tags:
  - rust
  - image-processing
  - zune-image
  - libvips
  - performance
  - simd
---

# Image Processing in Rust

Rust offers several approaches to image processing, from pure-Rust libraries emphasizing safety and ease of deployment to high-performance C/C++ integrations for demanding workloads. This guide covers the major options, their trade-offs, and practical implementation guidance.

## Table of Contents

- [Overview](#overview)
- [The image Crate](#the-image-crate)
- [Zune-Image](#zune-image)
- [libvips/Sharp Integration](#libvipssharp-integration)
- [Alternative Libraries](#alternative-libraries)
- [Performance Comparison](#performance-comparison)
- [Batch Processing with Rayon](#batch-processing-with-rayon)
- [Metadata Handling](#metadata-handling)
- [Decision Guide](#decision-guide)
- [Quick Reference](#quick-reference)
- [Resources](#resources)

## Overview

The Rust image processing ecosystem offers distinct philosophies:

| Approach | Philosophy | Best For |
|----------|------------|----------|
| **image crate** | Safety, ergonomics, ecosystem integration | General-purpose, most projects |
| **zune-image** | Speed, SIMD optimization, memory efficiency | High-throughput backends |
| **libvips (Sharp)** | Maximum performance, streaming pipelines | CDNs, massive images, professional workflows |
| **Specialized crates** | Focused functionality | Specific use cases (resizing, WASM, CV) |

## The image Crate

The `image` crate is the de facto standard for Rust image processing with 3,900+ dependent crates. It prioritizes safety, ease of use, and deep ecosystem integration.

### Architecture: The Buffer Model

The `image` crate loads entire images into memory as `DynamicImage` or `ImageBuffer` structs. This intuitive model works well for most use cases but can be memory-intensive for very large files (e.g., a 10,000x10,000px TIFF requires significant RAM).

### Format Support (v0.25+)

The crate supports **15+ formats** through feature flags:

| Format | Decode | Encode | Color Types | Notes |
|--------|--------|--------|-------------|-------|
| **AVIF** | Yes | Yes | 8-16 bit, RGB/RGBA | Uses `ravif` encoder |
| **BMP** | Yes | Yes | RGB, RGBA, L, LA, Indexed | RLE4, RLE8 compression |
| **DDS** | Yes | No | RGB, RGBA | DXT1, DXT3, DXT5 |
| **EXR** | Yes | Yes | RGB32F, RGBA32F | OpenEXR, no DWA compression |
| **Farbfeld** | Yes | Yes | RGB, RGBA | Lossless |
| **GIF** | Yes | Yes | RGB, RGBA | Animation support |
| **HDR** | Yes | Yes | RGB32F | Radiance HDR format |
| **ICO** | Yes | Yes | RGB, RGBA, L, LA | Multiple sub-images |
| **JPEG** | Yes | Yes | Baseline/Progressive | **Uses zune-jpeg backend (v0.25+)** |
| **PNG** | Yes | Yes | All types | APNG support, pure Rust |
| **PNM** | Yes | Yes | RGB, RGBA, L, LA | PBM, PGM, PPM, PAM |
| **QOI** | Yes | Yes | RGB, RGBA | "Quite OK Image" format |
| **TGA** | Yes | Yes | RGB, RGBA, L, LA, Indexed | RLE support |
| **TIFF** | Yes | Yes | Various | LZW, PackBits, Deflate |
| **WebP** | Yes | Yes | RGB, RGBA | Via `image-webp` feature |

**Limitations:** Struggles with "pro" formats like HEIF (requires extra native features) and cannot handle multi-page PDFs or SVGs natively.

### Recent Performance Revolution (v0.25)

Version 0.25 brought massive performance improvements:

- **JPEG decoding**: Switched to zune-jpeg backend, now on par with libjpeg-turbo
- **PNG decoding**: Pure Rust decoder is **1.8x faster than libpng** on x86, **1.5x faster** on ARM
- **PNG encoding**: Uses fdeflate - "dramatically faster than libpng" with better compression
- **Memory safety**: Zero-cost performance while maintaining safety guarantees

### Core Types

```rust
// High-level API
let img = ImageReader::open("image.png")?.decode()?;
img.save("output.jpg")?;

// Dynamic images
let dyn_img: DynamicImage = image::open(path)?;
let rgba: ImageBuffer<Rgba<u8>, _> = dyn_img.to_rgba8();
```

- **`ImageBuffer`**: Statically typed pixel buffers (`ImageBuffer<Rgb<u8>, Vec<u8>>`)
- **`DynamicImage`**: Enum over all supported pixel types
- **`GenericImage / GenericImageView`**: Trait-based image manipulation
- **`ImageDecoder / ImageEncoder`**: Low-level codec traits

### Resizing Algorithms

The crate provides standard filters:

- **Nearest** - Fastest, lowest quality
- **Triangle** - Fast, decent quality
- **CatmullRom** - Good general-purpose
- **Gaussian** - Smooth results
- **Lanczos3** - Highest quality, slowest

### Feature Flags

```toml
[dependencies]
# Minimal - only PNG and JPEG
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }

# Default includes most formats + rayon for multi-threading
image = "0.25"
```

**Performance features:**

- **rayon**: Multi-threading in encoders/decoders
- **nasm**: AVX optimizations for AVIF encoding
- **color_quant**: Color quantization for GIF encoding

### Metadata Support

Historically weak, but version **0.25+** added significant support for:

- ICC profiles
- XMP/IPTC metadata reading
- Fast dimension reading via `image_dimensions()` without full decode

For complex Exif handling, you typically need a secondary crate like `kamadak-exif`. The crate lacks a unified API for editing metadata and does not handle auto-rotation based on Exif orientation tags automatically.

### When to Choose image

- Simplicity and safety are priorities
- Building CLI tools, desktop apps, or simple web services
- Not processing gigapixel images or thousands of images per second
- Want to avoid managing C++ toolchains
- Need tight integration with the Rust ecosystem (e.g., `wgpu`, `egui`)
- Minimal metadata needs

## Zune-Image

A modern, fast, memory-efficient image processing library designed as a high-performance alternative to the `image` crate. It often matches or exceeds `libjpeg-turbo` performance.

### Setup

```toml
[dependencies]
zune-image = "0.4"
zune-imageprocs = "0.4"
zune-core = "0.4"
```

### Format Support

`zune-image` uses a modular architecture where each format has its own sub-crate:

| Format | Decoder | Encoder | no_std Support | Notes |
|--------|---------|---------|----------------|-------|
| **JPEG** | zune-jpeg | jpeg-encoder | Yes | Exceptionally fast |
| **PNG** | zune-png | zune-png | Yes | High-performance SIMD filtering |
| **JPEG-XL** | jxl-oxide | zune-jpegxl | Yes* | *Loses threading in no_std |
| **PPM/PAM/PGM** | zune-ppm | zune-ppm | Yes | Netpbm formats |
| **QOI** | zune-qoi | zune-qoi | Yes | Quite OK Image format |
| **Farbfeld** | zune-farbfeld | zune-farbfeld | Yes | Suckless image format |
| **PSD** | zune-psd | - | Yes | Adobe Photoshop (decode only) |
| **HDR** | zune-hdr | zune-hdr | No** | **Requires floor/exp from std |
| **BMP** | zune-bmp | - | - | Decode only |

**Notable Limitations:** No WebP, GIF, TIFF, or AVIF support. The library focuses on core formats and performance rather than comprehensive format coverage.

### Key Features

- **HDR Support:** Full support for `u8`, `u16`, and `f32` (HDR) image processing
- **SIMD Optimized:** Uses SIMD intrinsics for IDCT in JPEG, alpha pre-multiplication, and more
- **Built-in Filters:** Exposure, contrast, blurring, color space conversion (CMYK, HSL)
- **Modular Registry:** Only loads format code you actually use, keeping binaries lean
- **Fuzz Tested:** Most decoders are fuzz tested with minimal unsafe code (only for SIMD)

### Resizing Images

```rust
use zune_image::image::Image;
use zune_imageprocs::resize::{Resize, ResizeMethod};
use zune_image::traits::OperationsTrait;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load an image
    let mut img = Image::open("input.jpg")?;

    // Define dimensions and method
    let resize_op = Resize::new(800, 600, ResizeMethod::Bilinear);

    // Execute (mutates in-place)
    resize_op.execute(&mut img)?;

    // Save
    img.save("resized.png")?;
    Ok(())
}
```

### Resize Method Comparison

| Method | Speed | Quality | Best Use Case |
|--------|-------|---------|---------------|
| **Nearest** | Fastest | Low | Drafts, pixel art |
| **Bilinear** | Fast | Medium | Fast web thumbnails |
| **Bicubic** | Moderate | High | General purpose |
| **Lanczos3** | Slow | Very High | Maximum sharpness for downscaling |

### Format Conversion

Converting between formats is straightforward - the library auto-detects format from headers:

```rust
use zune_image::image::Image;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Auto-detects format from file header
    let img = Image::open("photo.jpg")?;

    // Picks correct encoder from extension
    img.save("converted_photo.png")?;
    Ok(())
}
```

### Colorspace Handling

`zune-image` is strict about colorspaces. Check `img.colorspace()` before processing:

- **RGB** - Standard for most screens
- **RGBA** - Includes alpha channel (transparency)
- **YCbCr** - Common in JPEGs; `zune-image` often decodes directly to this for performance

### Performance Tips

Enable SIMD features in `Cargo.toml` and compile for a target that supports them (like `avx2` on x86). This can result in **3x-10x speedup** for resizing and decoding.

## libvips/Sharp Integration

For maximum performance, integrating the C-based `libvips` engine (which powers Node.js's Sharp library) offers unmatched throughput and memory efficiency.

### Architecture: The Pipeline/Streaming Model

Unlike the buffer model, `libvips` is **demand-driven**:

1. Operations (crop -> resize -> sharpen) build a directed acyclic graph (DAG)
2. Pixels are pulled through the pipeline in small regions (tiles) as needed
3. This allows processing images much larger than available RAM

**Result:** Often **5x-10x faster** than buffer-based libraries for complex transformations on large files.

### Format Support

`libvips` supports almost everything by linking to system libraries:

- JPEG, PNG, WebP, TIFF, GIF
- **HEIC, AVIF** - Modern formats
- **PDF, SVG** - Renders at desired DPI
- **OpenEXR** - Professional HDR
- **FITS** - Scientific imaging
- Matroska (video frames)

### Resizing Performance

`libvips` uses a highly optimized `vips_thumbnail` operation with **shrink-on-load**:

- For JPEGs, it downsamples during decode (DCT scaling)
- Never decodes full-resolution pixels when not needed
- Drastically lower peak memory usage

**Example:** Resizing a 500MB TIFF might use only 20MB RAM with libvips versus hundreds of megabytes with buffer-based approaches.

### Metadata Handling

Leverages the **exiv2** library for professional-grade metadata:

- Read, write, modify, and remove EXIF, IPTC, and XMP
- **Auto-rotation** based on Exif orientation tags
- Copy metadata between images

### Integration Challenges

Integrating libvips into Rust is complex:

1. **FFI Boundary:** Requires `extern "C"` function wrappers
2. **Tools:**
   - `bindgen` - Generates Rust bindings from C headers (produces `unsafe` code)
   - `cxx` - Modern safe bridge with a shared interface DSL (preferred)
3. **Build System:**
   - `build.rs` must detect and link `libvips` and `exiv2`
   - May need to invoke `cmake` for source builds
   - Cross-compilation becomes significantly harder
4. **Error Handling:** Must catch C++ exceptions at FFI boundary and translate to `Result`
5. **Memory Management:** Must explicitly handle who allocates and frees memory

### Setup Requirements

| Feature | `image` Crate | libvips FFI |
|---------|---------------|-------------|
| **Setup** | Add to `Cargo.toml` | Requires `libvips-dev` installed |
| **Linking** | Fully static | Usually dynamic; static linking is difficult |
| **Safety** | Memory-safe | Requires `unsafe`; C/C++ attack surface |

### When to Choose libvips

- Performance and memory efficiency are absolutely critical
- Building high-throughput image processing servers (CDNs, APIs)
- Processing extremely large images (hundreds of megapixels)
- Need robust read/write access to EXIF, IPTC, XMP
- Working with professional formats (HEIC, OpenEXR, PDF)
- Have expertise to manage complex builds and FFI risks

## Alternative Libraries

### Photon

High-performance library optimized for native and **WebAssembly** environments.

**Features:**

- Over 80 built-in effects and filters
- WASM-first design for browser-based editors
- HSL, LCh, sRGB color space manipulation
- Standard cropping, resizing, watermarking

**Best For:** Browser-based image editors, edge computing filters

**Resources:** [Photon Site](https://silvia-odwyer.github.io/photon/) | [Docs.rs](https://docs.rs/photon-rs/)

### Fast Image Resize (FIR)

If resizing is your primary need, `fast_image_resize` is the most efficient option.

**Features:**

- Heavy SIMD acceleration (SSE4.1, AVX2, NEON)
- High-quality algorithms: Lanczos3, Catmull-Rom, Mitchell, Bilinear
- Zero-copy API for pixel buffers
- Gamma-correct resizing (linear color space conversion)

**Best For:** Thumbnail generation, scaling-focused workloads

**Resources:** [GitHub](https://github.com/shekurzh/rust-fast-image-resize) | [Docs.rs](https://docs.rs/fast_image_resize/)

### Tiny-Skia

High-quality 2D graphics rendering library - a subset of Skia in pure Rust.

**Features:**

- All Porter-Duff blending modes
- High-quality anti-aliased rasterization
- 100% safe Rust
- Complex pixel manipulation and compositing

**Best For:** Drawing, compositing, anti-aliased masking, text overlay

**Resources:** [GitHub](https://github.com/RazrFalcon/tiny-skia) | [Docs.rs](https://docs.rs/tiny-skia/)

### Rust CV

A mono-repo collection of computer vision crates for semantic understanding.

**Features:**

- Feature extraction (keypoints, descriptors, ORB)
- Geometric vision (camera calibration, stereo, structure from motion)
- Pure Rust implementations

**Best For:** Computer vision, feature detection, photogrammetry

**Resources:** [Website](https://rust-cv.github.io/) | [GitHub](https://github.com/rust-cv/cv)

### Comparison Summary

| Library | Primary Use Case | Key Strength |
|---------|------------------|--------------|
| **Zune-Image** | General Purpose / Backend | Speed and format support |
| **Photon** | Web / Filters | WASM compatibility & preset effects |
| **Fast Image Resize** | Resizing / Thumbnails | Raw SIMD performance |
| **Tiny-Skia** | Drawing / Compositing | Anti-aliased rendering & blending |
| **Rust CV** | Computer Vision | Semantic and geometric analysis |

## Performance Comparison

### image vs zune-image

| Aspect | image crate | zune-image |
|--------|-------------|------------|
| **Formats** | 15+ (comprehensive) | 8+ (focused) |
| **JPEG Backend** | zune-jpeg (v0.25+) | Native zune-* decoders |
| **PNG Speed** | 1.8x libpng (fast) | 1.7-3.5x image crate (faster) |
| **HDR Support** | Decode/encode RGB32F | Full u8/u16/f32 pipeline |
| **API Maturity** | Very mature, stable | Evolving, consistent traits |
| **Ecosystem** | De-facto standard (3,904 deps) | Emerging, performance-focused |
| **Compile Time** | Slower (monolithic) | Faster (modular) |
| **Binary Size** | Larger (~1.5-6MB) | Smaller (pay-per-format) |

### image vs libvips

For a standard "create thumbnail from 4K photo" task:

| Library | Approach | Memory Usage |
|---------|----------|--------------|
| **image** | Decode full 4K -> resize -> encode | High (full image in RAM) |
| **libvips** | Shrink during decode -> stream tiles -> encode | Low (tile-based) |

**libvips** is often 2-5x faster and uses significantly less RAM, especially for large images or chained operations.

### Benchmark Considerations

- Single small images: Differences are minimal
- Large batches: zune-image and libvips show significant advantages
- Large files: libvips streaming architecture dominates
- SIMD targets: Ensure you compile with appropriate flags (`-C target-cpu=native`)

## Batch Processing with Rayon

### With the image Crate

```toml
[dependencies]
rayon = "1.8"
image = "0.24"
anyhow = "1.0"
```

```rust
use rayon::prelude::*;
use image::{DynamicImage, ImageFormat, imageops::FilterType};
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;

fn process_image(
    input_path: &Path,
    output_path: &Path,
    width: u32,
    height: u32,
) -> Result<()> {
    let img = image::open(input_path)
        .with_context(|| format!("Failed to open: {}", input_path.display()))?;

    let processed = img
        .resize(width, height, FilterType::Lanczos3)
        .grayscale();

    let format = ImageFormat::from_path(output_path)
        .unwrap_or(ImageFormat::Png);

    processed.save_with_format(output_path, format)
        .with_context(|| format!("Failed to save: {}", output_path.display()))?;

    Ok(())
}

fn process_batch(
    input_dir: &Path,
    output_dir: &Path,
    width: u32,
    height: u32,
) -> Result<()> {
    fs::create_dir_all(output_dir)?;

    let entries: Vec<PathBuf> = fs::read_dir(input_dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .map(|ext| matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png"))
                .unwrap_or(false)
        })
        .collect();

    // Process in parallel with error collection
    let failures: Vec<_> = entries
        .par_iter()
        .map(|input_path| {
            let output_path = output_dir.join(input_path.file_name().unwrap());
            process_image(input_path, &output_path, width, height)
                .map_err(|e| (input_path.clone(), e))
        })
        .filter_map(Result::err)
        .collect();

    println!("Processed: {}", entries.len() - failures.len());
    for (path, error) in failures {
        eprintln!("Failed {}: {}", path.display(), error);
    }

    Ok(())
}
```

**Key patterns:**

- `.par_iter()` converts standard iterator to parallel
- Error isolation: one failure doesn't stop the batch
- Work-stealing: Rayon distributes work automatically

### With zune-image

Batch processing is where `zune-image` truly shines. Its thread-safe decoders combine excellently with Rayon for maximum throughput.

```toml
[dependencies]
zune-image = "0.4"
zune-imageprocs = "0.4"
rayon = "1.8"
walkdir = "2.4"
```

```rust
use rayon::prelude::*;
use std::path::PathBuf;
use walkdir::WalkDir;
use zune_image::image::Image;
use zune_imageprocs::resize::{Resize, ResizeMethod};

fn process_image(path: PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut img = Image::open(&path)?;

    let resize_op = Resize::new(300, 300, ResizeMethod::Bilinear);
    resize_op.execute(&mut img)?;

    let mut out_path = path.clone();
    out_path.set_extension("png");
    let file_name = out_path.file_stem().unwrap().to_str().unwrap();
    out_path.set_file_name(format!("{}_thumb.png", file_name));

    img.save(out_path)?;
    Ok(())
}

fn main() {
    let files: Vec<PathBuf> = WalkDir::new("./input_images")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "jpg"))
        .map(|e| e.path().to_owned())
        .collect();

    println!("Processing {} images...", files.len());

    files.into_par_iter().for_each(|path| {
        match process_image(path.clone()) {
            Ok(_) => println!("Processed {:?}", path),
            Err(e) => eprintln!("Error {:?}: {}", path, e),
        }
    });
}
```

### Pipeline Internals

1. **Decoding:** Byte stream (JPEG/PNG) becomes contiguous pixel buffer (`u8`/`u16`)
2. **Processing:** Operations use SIMD for multiple pixels simultaneously
3. **Encoding:** Final buffer compressed back to file format

### Pixel-Level Parallelism

For single-image filters, you can parallelize at the pixel level:

```rust
// Process rows of an image in parallel
let processed_rows: Vec<_> = img.rows()
    .par_bridge()  // Convert sequential iterator to parallel
    .map(|row| {
        row.map(|pixel| apply_filter(pixel)).collect()
    })
    .collect();
```

This provides **2-3x speedup** typical on multi-core systems for I/O-bound batch processing and CPU-intensive transformations.

### Metadata Gotcha

When batch processing, **metadata is not always preserved** if the target format doesn't support source metadata. Example: JPEG with Exif -> PPM loses all Exif (PPM has no Exif support).

## Metadata Handling

### With zune-image

#### Reading Metadata

```rust
use zune_image::image::Image;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = Image::open("image_with_exif.jpg")?;

    // Decode only headers
    img.decode_headers()?;

    // Access raw Exif bytes
    if let Some(exif) = img.metadata().exif() {
        println!("Found Exif: {} bytes", exif.len());
    }

    // Access ICC Profile
    if let Some(icc) = img.metadata().icc_profile() {
        println!("Found ICC: {} bytes", icc.len());
    }

    Ok(())
}
```

#### Writing Metadata

```rust
use zune_image::image::Image;
use zune_core::metadata::ImageMetadata;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = Image::open("input.png")?;

    let mut metadata = ImageMetadata::default();
    let my_exif = vec![0u8; 10]; // Real Exif bytes
    metadata.set_exif(my_exif);

    img.set_metadata(metadata);
    img.save("output_with_metadata.jpg")?;

    Ok(())
}
```

### With kamadak-exif

For detailed Exif parsing, combine `zune-image` with `kamadak-exif`:

```toml
[dependencies]
zune-image = "0.4"
kamadak-exif = "0.5"
```

```rust
use std::io::Cursor;
use zune_image::image::Image;
use kamadak_exif::{Context, Reader, Tag};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = Image::open("photo.jpg")?;
    img.decode_headers()?;

    if let Some(raw_exif) = img.metadata().exif() {
        let mut cursor = Cursor::new(raw_exif);
        let reader = Reader::new();

        match reader.read_from_container(&mut cursor) {
            Ok(exif) => {
                for field in exif.fields() {
                    println!("{}: {}", field.tag, field.display_value().with_unit(&exif));
                }

                if let Some(field) = exif.get_field(Tag::Model, Context::Standard) {
                    println!("Camera Model: {}", field.display_value());
                }
            }
            Err(e) => eprintln!("Failed to parse Exif: {}", e),
        }
    }

    Ok(())
}
```

### Best Practices

- **Stripping Metadata:** For web optimization, clear with `img.metadata_mut().set_exif(vec![])`
- **ICC Profiles:** Keep when converting color spaces (HDR -> JPEG) for correct colors
- **Auto-rotation:** Handle manually with `image` crate; automatic with libvips

## Decision Guide

### Choose image crate when

- Safety and simplicity are priorities
- Building CLI tools, desktop apps, simple web services
- Standard web formats (JPEG, PNG, GIF, WebP)
- Want zero external dependencies
- Need ecosystem integration (`wgpu`, `egui`, etc.)

### Choose zune-image when

- Performance matters for backend processing
- Processing many images in batches
- Want pure Rust with SIMD optimization
- Need HDR/high bit-depth support
- Modern format support (JPEG XL, QOI)
- Need `no_std` support

### Choose libvips when

- Maximum throughput is critical (CDNs, APIs)
- Processing gigapixel or multi-GB images
- Need streaming/pipeline architecture
- Professional metadata handling required
- Working with PDF, SVG, HEIC, OpenEXR

### Choose specialized crates when

- **Resizing only:** fast_image_resize
- **WASM/browser:** Photon
- **Drawing/compositing:** tiny-skia
- **Computer vision:** rust-cv

### Summary Decision Table

| Use Case | Best Choice | Why? |
|----------|-------------|------|
| **Simple Web Backend** | `image` crate | Easy deployment; no system dependencies |
| **Massive Image Processing** | `libvips` | Pipeline architecture saves RAM |
| **Security-Critical** | `image` crate | Pure Rust minimizes buffer overflow risks |
| **Professional Photography** | `libvips` | Superior EXIF/ICC handling and CMYK support |
| **Maximum JPEG/PNG Speed** | `zune-image` | Custom SIMD-optimized decoders |
| **Embedded/no_std** | `zune-image` | Modular no_std support |

## Quick Reference

### Common Operations Comparison

| Operation | image | zune-image | libvips |
|-----------|-------|------------|---------|
| Add to project | `cargo add image` | `cargo add zune-image` | Install libvips-dev + FFI |
| Load image | `image::open(path)` | `Image::open(path)` | `VipsImage::new_from_file()` |
| Resize | `img.resize(w, h, filter)` | `Resize::new().execute(&mut img)` | `vips_thumbnail()` |
| Save | `img.save(path)` | `img.save(path)` | `img.write_to_file()` |
| Memory model | Full buffer | Full buffer + SIMD | Streaming tiles |

### Feature Matrix

| Feature | image | zune-image | libvips |
|---------|-------|------------|---------|
| Pure Rust | Yes | Yes | No (C FFI) |
| SIMD optimization | Limited | Extensive | Extensive |
| Streaming | No | No | Yes |
| HEIC/AVIF | Feature flag | No | Yes |
| PDF/SVG | No | No | Yes |
| Metadata write | Limited | Basic | Full |
| WASM | Possible | Not focus | No |
| no_std | No | Partial | No |
| Memory efficiency | Moderate | Good | Excellent |

## Resources

### Official Documentation

- [image crate docs.rs](https://docs.rs/image/)
- [zune-image docs.rs](https://docs.rs/zune-image/)
- [libvips documentation](https://www.libvips.org/API/current/)

### GitHub Repositories

- [image-rs/image](https://github.com/image-rs/image)
- [etemesi254/zune-image](https://github.com/etemesi254/zune-image)
- [libvips/libvips](https://github.com/libvips/libvips)
- [fast_image_resize](https://github.com/shekurzh/rust-fast-image-resize)
- [tiny-skia](https://github.com/RazrFalcon/tiny-skia)
- [Photon](https://silvia-odwyer.github.io/photon/)
- [rust-cv](https://rust-cv.github.io/)

### Related Crates

- [kamadak-exif](https://crates.io/crates/kamadak-exif) - Exif parsing
- [rayon](https://crates.io/crates/rayon) - Parallel processing
- [walkdir](https://crates.io/crates/walkdir) - Directory traversal
