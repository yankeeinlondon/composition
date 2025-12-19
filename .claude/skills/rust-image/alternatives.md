# Alternative Image Processing Libraries

Beyond `image` and `zune-image`, Rust has specialized libraries for specific use cases.

## Photon

High-performance library optimized for both native and WebAssembly environments.

```toml
[dependencies]
photon-rs = "0.3"
```

**Strengths:**
- WASM-first design for browser image editors
- 80+ built-in effects and filters
- HSL, LCh, sRGB color space manipulation
- Cropping, resizing, watermarking

**Use when:** Building browser-based image editors or need preset filter effects.

**Resources:**
- [Photon Website](https://silvia-odwyer.github.io/photon/)
- [docs.rs/photon-rs](https://docs.rs/photon-rs/)

## fast_image_resize

The fastest option when resizing is your only need.

```toml
[dependencies]
fast_image_resize = "3"
```

**Strengths:**
- Heavy SIMD acceleration (SSE4.1, AVX2, NEON)
- High-quality algorithms: Lanczos3, Catmull-Rom, Mitchell
- Zero-copy API for memory efficiency
- Gamma-correct resizing (linear color space conversion)

**Performance:** Significantly faster than `image` crate for resize operations.

**Use when:** Thumbnail generation is your bottleneck.

**Resources:**
- [GitHub](https://github.com/Cykooz/fast_image_resize)
- [docs.rs/fast_image_resize](https://docs.rs/fast_image_resize/)

## tiny-skia

Pure Rust 2D graphics library, subset of Skia functionality.

```toml
[dependencies]
tiny-skia = "0.11"
```

**Strengths:**
- All Porter-Duff blending modes
- High-quality anti-aliased rasterization
- Shape and text overlay
- 100% safe Rust

**Use when:** Compositing images, adding overlays, drawing shapes, or need advanced blending.

**Resources:**
- [GitHub](https://github.com/RazrFalcon/tiny-skia)
- [docs.rs/tiny-skia](https://docs.rs/tiny-skia/)

## Rust CV

Collection of crates for computer vision, not just image manipulation.

**Strengths:**
- Feature extraction (ORB keypoints, descriptors)
- Camera calibration, stereo vision
- Structure from motion
- Pure Rust alternative to OpenCV

**Use when:** Need semantic understanding, feature detection, or photogrammetry.

**Resources:**
- [Rust CV Website](https://rust-cv.github.io/)
- [GitHub](https://github.com/rust-cv/cv)

## libvips via FFI

C library with Rust bindings for extreme performance.

```toml
[dependencies]
libvips = "1"  # Or use vips-rs
```

**Strengths:**
- Streaming/pipeline architecture (minimal RAM for huge images)
- Shrink-on-load for JPEGs (decode at reduced resolution)
- HEIC, AVIF, PDF, SVG, OpenEXR support
- Professional EXIF/IPTC/XMP handling with exiv2

**Trade-offs:**
- Requires system library installation
- Complex build setup, difficult cross-compilation
- `unsafe` FFI boundary

**Use when:** Processing gigapixel images or need 5-10x performance over pure Rust.

## Comparison Summary

| Library | Primary Use Case | Key Strength |
|---------|------------------|--------------|
| **image** | General purpose | Ecosystem standard, broad formats |
| **zune-image** | Performance | Fastest JPEG/PNG decode |
| **photon** | Web/Filters | WASM + 80 preset effects |
| **fast_image_resize** | Thumbnails | SIMD resize performance |
| **tiny-skia** | Compositing | Blending and anti-aliasing |
| **Rust CV** | Computer vision | Feature detection |
| **libvips** | High throughput | Streaming architecture |

## Decision Guide

1. **Default choice:** `image` crate
2. **Need faster JPEG/PNG:** `zune-image`
3. **Only resizing:** `fast_image_resize`
4. **WASM/browser:** `photon`
5. **Drawing/compositing:** `tiny-skia`
6. **Feature detection:** Rust CV
7. **Extreme scale/performance:** libvips

## Related

- [image crate](./image-crate.md)
- [zune-image](./zune-image.md)
- [image vs libvips](./image-vs-libvips.md)
