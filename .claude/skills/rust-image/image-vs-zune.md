# image vs zune-image

When to choose between Rust's two main pure-Rust image processing libraries.

## Quick Decision

| Criterion | Choose `image` | Choose `zune-image` |
|-----------|----------------|---------------------|
| Format needs | WebP, GIF, TIFF, AVIF | Only JPEG, PNG, JPEG-XL, QOI |
| Performance priority | Good enough | Maximum decode speed |
| Binary size | Less concern | Minimal binary |
| Ecosystem | Integrate with other crates | Standalone processing |
| API stability | Very important | Can handle changes |
| no_std | Not needed | Needed |

## Performance Comparison

### JPEG Decoding

| Library | Speed | Notes |
|---------|-------|-------|
| libjpeg-turbo (C) | Baseline | Industry reference |
| zune-jpeg | ~1.0x libjpeg-turbo | SIMD optimized |
| image (v0.25+) | ~1.0x libjpeg-turbo | Uses zune-jpeg backend |
| image (v0.24) | ~2-3x slower | Old pure-Rust decoder |

**Note:** Since v0.25, `image` crate uses `zune-jpeg` internally, closing the gap.

### PNG Decoding

| Library | vs C libpng |
|---------|-------------|
| image crate | 1.8x faster (x86), 1.5x faster (ARM) |
| zune-png | 1.7-3.5x faster than image crate |

### Binary Size

| Configuration | Approximate Size |
|---------------|------------------|
| image (all formats) | 1.5-6 MB |
| image (jpeg+png only) | ~800 KB |
| zune-image (jpeg+png) | ~400 KB |

## Feature Comparison

### Format Support

| Format | image | zune-image |
|--------|-------|------------|
| JPEG | Yes | Yes |
| PNG | Yes | Yes |
| WebP | Yes | No |
| GIF | Yes (animated) | No |
| TIFF | Yes | No |
| AVIF | Yes | No |
| BMP | Yes | Yes |
| JPEG-XL | No | Yes |
| QOI | Yes | Yes |
| HDR | Yes | Yes |
| EXR | Yes | No |
| PSD | No | Yes (decode) |

### API Design

**image crate:**
```rust
// Load/save with method chaining
let img = image::open("input.jpg")?
    .resize(800, 600, FilterType::Lanczos3)
    .grayscale();
img.save("output.png")?;

// DynamicImage enum handles all pixel types
let rgba = img.to_rgba8();
```

**zune-image:**
```rust
// Operations as objects that execute on image
let mut img = Image::open("input.jpg")?;
Resize::new(800, 600, ResizeMethod::Bilinear).execute(&mut img)?;
img.save("output.png")?;

// Explicit colorspace handling
let cs = img.colorspace();
```

### HDR Support

| Aspect | image | zune-image |
|--------|-------|------------|
| Bit depths | u8, u16, f32 | u8, u16, f32 |
| HDR workflow | Decode/encode | Full pipeline |
| Color precision | Per-operation | Maintained throughout |

### Ecosystem Integration

**image crate:**
- wgpu (GPU textures)
- egui (GUI rendering)
- pixels (framebuffers)
- winit (windowing)
- Most graphics crates expect `image` types

**zune-image:**
- Standalone design
- Less ecosystem coupling
- Easier to use in embedded/no_std

## When to Choose

### Choose image crate when:

1. **You need format variety** - WebP, GIF, TIFF, AVIF support
2. **Integrating with other crates** - Most expect `DynamicImage`
3. **API stability matters** - Mature, well-documented
4. **Building typical applications** - CLI tools, web services, desktop apps
5. **Processing animated images** - GIF, APNG support

### Choose zune-image when:

1. **Performance is critical** - Highest JPEG/PNG decode speed
2. **Binary size matters** - WASM, embedded, CLI tools
3. **Only need core formats** - JPEG, PNG, JPEG-XL
4. **no_std environment** - Embedded systems
5. **HDR workflows** - Professional color processing
6. **JPEG-XL support** - Modern codec not in image crate

## Hybrid Approach

The `image` crate (v0.25+) now uses zune-jpeg internally, giving you:
- Ecosystem compatibility of `image`
- JPEG performance of `zune`

For most projects, **start with `image` crate**. Switch to `zune-image` only if:
- Benchmarks show decode is bottleneck
- Need JPEG-XL
- Binary size is critical
- Building for no_std

## Migration Path

### image -> zune-image

```rust
// image
let img = image::open("input.jpg")?;
let resized = img.resize(800, 600, FilterType::Lanczos3);
resized.save("output.png")?;

// zune-image
let mut img = Image::open("input.jpg")?;
Resize::new(800, 600, ResizeMethod::Lanczos3).execute(&mut img)?;
img.save("output.png")?;
```

### zune-image -> image

If you need a format zune doesn't support, you can convert:

```rust
// zune-image pixels to image ImageBuffer
let zune_img = Image::open("input.jpg")?;
// Extract raw pixels and create ImageBuffer
```

## Related

- [image crate](./image-crate.md)
- [zune-image](./zune-image.md)
- [Batch processing](./batch-processing.md)
