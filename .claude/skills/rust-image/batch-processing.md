# Batch Image Processing with Rayon

Combine Rust image libraries with Rayon for parallel file processing. Rayon's work-stealing scheduler automatically distributes work across CPU cores.

## image Crate + Rayon

### Setup

```toml
[dependencies]
image = "0.25"
rayon = "1.8"
anyhow = "1.0"
```

### Complete Batch Processor

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

    // Collect valid image files
    let entries: Vec<PathBuf> = fs::read_dir(input_dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .map(|ext| matches!(ext.to_lowercase().as_str(),
                    "jpg" | "jpeg" | "png" | "webp"))
                .unwrap_or(false)
        })
        .collect();

    println!("Processing {} images...", entries.len());

    // Process in parallel, collect failures
    let failures: Vec<_> = entries
        .par_iter()
        .map(|input_path| {
            let output_path = output_dir.join(
                input_path.file_name().expect("Invalid filename")
            );
            process_image(input_path, &output_path, width, height)
                .map_err(|e| (input_path.clone(), e))
        })
        .filter_map(Result::err)
        .collect();

    let success_count = entries.len() - failures.len();
    println!("Successfully processed: {}", success_count);

    for (path, error) in &failures {
        eprintln!("Failed {}: {}", path.display(), error);
    }

    Ok(())
}
```

## zune-image + Rayon

### Setup

```toml
[dependencies]
zune-image = "0.4"
zune-imageprocs = "0.4"
rayon = "1.8"
walkdir = "2.4"
```

### High-Performance Batch Processor

```rust
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zune_image::image::Image;
use zune_imageprocs::resize::{Resize, ResizeMethod};

fn process_image(path: PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut img = Image::open(&path)?;

    let resize_op = Resize::new(300, 300, ResizeMethod::Bilinear);
    resize_op.execute(&mut img)?;

    // Create output path: photo.jpg -> photo_thumb.png
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

## Key Patterns

### Parallel Iteration

```rust
// Sequential
for file in files {
    process(file);
}

// Parallel (just add .par_iter())
files.par_iter().for_each(|file| {
    process(file);
});
```

### Error Isolation

Each file processes independently - one failure doesn't stop the batch:

```rust
let results: Vec<_> = files
    .par_iter()
    .map(|f| process(f))
    .collect();

let (successes, failures): (Vec<_>, Vec<_>) = results
    .into_iter()
    .partition(Result::is_ok);
```

### Pixel-Level Parallelism

For single large images, parallelize row processing:

```rust
use rayon::prelude::*;

let processed_rows: Vec<_> = img.rows()
    .par_bridge()  // Convert sequential iterator to parallel
    .map(|row| {
        row.map(|pixel| apply_filter(pixel)).collect()
    })
    .collect();
```

## Performance Expectations

- **I/O bound batches:** 2-3x speedup on multi-core systems
- **CPU-intensive transforms:** Near-linear scaling with core count
- **Automatic load balancing:** Rayon's work-stealing handles uneven image sizes

## Important Considerations

1. **Memory pressure:** Each thread loads an image into memory. For large images, consider limiting thread count:

   ```rust
   rayon::ThreadPoolBuilder::new()
       .num_threads(4)
       .build_global()
       .unwrap();
   ```

2. **Error handling:** Use `Send + Sync` error types for cross-thread errors
3. **Metadata loss:** Format conversion may not preserve EXIF/ICC profiles by default

## Related

- [image crate](./image-crate.md)
- [zune-image](./zune-image.md)
- [Metadata handling](./metadata.md)
