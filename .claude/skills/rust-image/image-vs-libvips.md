# image Crate vs libvips (Sharp)

Comparing native Rust (`image` crate) with C-based libvips for high-performance image processing.

## Executive Summary

| Aspect | image crate | libvips |
|--------|-------------|---------|
| Philosophy | Safe, general-purpose | Maximum throughput |
| Safety | Memory safe | Unsafe FFI boundary |
| Setup | `cargo add image` | System library + FFI bindings |
| Performance | Good | 2-10x faster for complex pipelines |
| Memory | Loads full image | Streaming, minimal RAM |
| Best for | Most applications | High-throughput servers |

## Architecture Difference

### image crate: Buffer Model

Loads the entire image into memory, processes, then saves:

```rust
// Full image in RAM
let img = image::open("large.jpg")?;  // 100MB TIFF = 100MB+ RAM
let resized = img.resize(800, 600, FilterType::Lanczos3);
resized.save("output.jpg")?;
```

### libvips: Pipeline/Streaming Model

Builds operation graph, streams pixels through in tiles:

```
load -> crop -> resize -> sharpen -> save
         |
         v (only loads tiles as needed)
```

**Result:** Processing a 500MB TIFF may use only 20MB RAM.

## When to Choose Each

### Choose image crate when:

- **Simplicity and safety** are priorities
- Building CLI tools, desktop apps, simple web services
- Not processing gigapixel images
- Need easy deployment (no system dependencies)
- Tight integration with Rust ecosystem
- Metadata needs are basic

### Choose libvips when:

- **Performance is critical** - CDN, image API server
- Processing extremely large images (100s of megapixels)
- Need 5-10x performance over pure Rust
- Require robust EXIF/IPTC/XMP read/write
- Need HEIC, AVIF, PDF, SVG support
- Have resources for complex build setup

## Format Support

| Format | image crate | libvips |
|--------|-------------|---------|
| JPEG | Yes | Yes |
| PNG | Yes | Yes |
| WebP | Yes | Yes |
| GIF | Yes (animated) | Yes |
| TIFF | Yes | Yes (better) |
| AVIF | Yes (limited) | Yes |
| HEIC | No | Yes |
| PDF | No | Yes (renders) |
| SVG | No | Yes (renders) |
| OpenEXR | Yes | Yes |

## Resizing Performance

### image crate

```rust
// Decodes full image, resizes in memory
let img = image::open("4k.jpg")?;  // Full 4K decode
let thumb = img.thumbnail(150, 150);
thumb.save("thumb.jpg")?;
```

### libvips

```rust
// Shrink-on-load: decodes at reduced resolution
// Never allocates full 4K pixels
vips_thumbnail("4k.jpg", 150, 150);
```

For JPEG, libvips uses DCT scaling during decode - significantly less CPU.

## Metadata Handling

| Feature | image crate | libvips |
|---------|-------------|---------|
| EXIF read | Basic | Full (via exiv2) |
| EXIF write | No | Yes |
| ICC profiles | Yes (v0.25+) | Yes |
| XMP | Limited | Full |
| Auto-rotation | Manual | Automatic |
| Metadata copy | Manual | Single function |

### libvips Auto-Rotation

libvips automatically rotates based on EXIF orientation:

```rust
// libvips - automatic
let img = vips_image_new_from_file("rotated.jpg", None)?;
// Already correctly oriented

// image crate - manual
let img = image::open("rotated.jpg")?;
let exif = extract_exif(&img);
let oriented = apply_orientation(img, exif.orientation);
```

## Integration Complexity

### image crate

```toml
[dependencies]
image = "0.25"
```

Done. Works on any platform, cross-compiles easily.

### libvips

1. **Install system library:**
   ```bash
   # Ubuntu
   apt install libvips-dev

   # macOS
   brew install vips
   ```

2. **Add Rust bindings:**
   ```toml
   [dependencies]
   libvips = "1"
   ```

3. **Build challenges:**
   - Dynamic linking by default
   - Static linking requires building 20+ dependencies
   - Cross-compilation is difficult
   - Adds C/C++ attack surface

## Performance Benchmarks

For "thumbnail a 4K JPEG":

| Library | Time | Peak RAM |
|---------|------|----------|
| image crate | ~150ms | ~50MB |
| libvips | ~30ms | ~10MB |

For "chain 5 operations on 100MP image":

| Library | Time | Peak RAM |
|---------|------|----------|
| image crate | ~5s | ~500MB |
| libvips | ~0.5s | ~30MB |

## Code Comparison

### Resize + Sharpen + Save

**image crate:**
```rust
let img = image::open("input.jpg")?;
let resized = img.resize(800, 600, FilterType::Lanczos3);
let sharpened = resized.unsharpen(1.0, 10);
sharpened.save("output.jpg")?;
```

**libvips (via bindings):**
```rust
let img = VipsImage::new_from_file("input.jpg")?;
let resized = img.thumbnail(800, 600)?;
let sharpened = resized.sharpen()?;
sharpened.write_to_file("output.jpg")?;
```

## Decision Matrix

| Scenario | Recommendation |
|----------|----------------|
| Simple CLI tool | image crate |
| Desktop app | image crate |
| Basic web service | image crate |
| CDN/image API | libvips |
| Processing 100s images/second | libvips |
| Gigapixel images | libvips |
| Need HEIC/PDF/SVG | libvips |
| Easy deployment | image crate |
| Security-critical | image crate |

## Middle Ground: zune-image

If `image` is too slow but libvips is too complex:

- 2-4x faster JPEG/PNG than older `image` versions
- Still pure Rust (safe, easy deployment)
- Limited format support

## Related

- [image crate](./image-crate.md)
- [zune-image](./zune-image.md)
- [Alternatives](./alternatives.md)
