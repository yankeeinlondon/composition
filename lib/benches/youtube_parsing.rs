//! Benchmark tests for YouTube parsing and rendering
//!
//! Performance targets:
//! - Video ID extraction: 1000 URLs < 10ms
//! - HTML generation: 100 embeds (reasonable time)
//! - Verify LazyLock regex compilation overhead

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use lib::parse::darkmatter::parse_directive;
use lib::render::youtube::{render_youtube_embed, youtube_css, youtube_js};
use lib::types::WidthSpec;

/// Benchmark video ID extraction from various URL formats
fn bench_video_id_extraction(c: &mut Criterion) {
    let test_urls = [
        "::youtube https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        "::youtube https://youtu.be/dQw4w9WgXcQ",
        "::youtube https://www.youtube.com/embed/dQw4w9WgXcQ",
        "::youtube https://youtube.com/v/dQw4w9WgXcQ",
        "::youtube dQw4w9WgXcQ",
    ];

    let mut group = c.benchmark_group("video_id_extraction");

    for (idx, url) in test_urls.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("url_format", idx),
            url,
            |b, directive| {
                b.iter(|| {
                    let _ = parse_directive(black_box(directive), 1);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark video ID extraction at scale (1000 URLs)
fn bench_video_id_extraction_bulk(c: &mut Criterion) {
    // Generate 1000 test URLs
    let mut test_urls = Vec::with_capacity(1000);
    let formats = [
        "::youtube https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        "::youtube https://youtu.be/jNQXAC9IVRw",
        "::youtube https://www.youtube.com/embed/9bZkp7q19f0",
        "::youtube https://youtube.com/v=M7lc1UVf-VE",
        "::youtube aBcDeFgHiJk",
    ];

    for i in 0..1000 {
        test_urls.push(formats[i % formats.len()]);
    }

    c.bench_function("video_id_extraction_1000_urls", |b| {
        b.iter(|| {
            for url in &test_urls {
                let _ = parse_directive(black_box(url), 1);
            }
        });
    });
}

/// Benchmark HTML generation for single embed
fn bench_html_generation_single(c: &mut Criterion) {
    let video_id = "dQw4w9WgXcQ";
    let widths = [
        WidthSpec::Pixels(512),
        WidthSpec::Pixels(800),
        WidthSpec::Rems(32.0),
        WidthSpec::Percentage(80),
    ];

    let mut group = c.benchmark_group("html_generation_single");

    for (idx, width) in widths.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("width_format", idx),
            width,
            |b, w| {
                b.iter(|| {
                    let _ = render_youtube_embed(black_box(video_id), black_box(w));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark HTML generation at scale (100 embeds)
fn bench_html_generation_bulk(c: &mut Criterion) {
    let video_ids: Vec<&str> = (0..100).map(|_| "dQw4w9WgXcQ").collect();
    let width = WidthSpec::default();

    c.bench_function("html_generation_100_embeds", |b| {
        b.iter(|| {
            for video_id in &video_ids {
                let _ = render_youtube_embed(black_box(video_id), black_box(&width));
            }
        });
    });
}

/// Benchmark CSS/JS asset access (verify LazyLock overhead)
fn bench_asset_access(c: &mut Criterion) {
    c.bench_function("css_access_first_call", |b| {
        b.iter(|| {
            let _ = youtube_css();
        });
    });

    c.bench_function("js_access_first_call", |b| {
        b.iter(|| {
            let _ = youtube_js();
        });
    });
}

/// Benchmark full parse-and-render pipeline
fn bench_full_pipeline(c: &mut Criterion) {
    let directive = "::youtube https://www.youtube.com/watch?v=dQw4w9WgXcQ 800px";

    c.bench_function("full_pipeline_parse_and_render", |b| {
        b.iter(|| {
            if let Ok(Some(lib::types::DarkMatterNode::YouTube { video_id, width })) = parse_directive(black_box(directive), 1) {
                let _ = render_youtube_embed(black_box(&video_id), black_box(&width));
            }
        });
    });
}

/// Benchmark width specification parsing
fn bench_width_parsing(c: &mut Criterion) {
    let directives = [
        "::youtube dQw4w9WgXcQ 512px",
        "::youtube dQw4w9WgXcQ 32rem",
        "::youtube dQw4w9WgXcQ 32.5rem",
        "::youtube dQw4w9WgXcQ 80%",
        "::youtube dQw4w9WgXcQ", // default width
    ];

    let mut group = c.benchmark_group("width_parsing");

    for (idx, directive) in directives.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("width_format", idx),
            directive,
            |b, d| {
                b.iter(|| {
                    let _ = parse_directive(black_box(d), 1);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark regex compilation overhead (LazyLock)
fn bench_regex_compilation_overhead(c: &mut Criterion) {
    // This benchmark verifies that regex is compiled once via LazyLock
    // Subsequent accesses should be extremely fast (just a pointer lookup)

    let test_directives = vec![
        "::youtube dQw4w9WgXcQ",
        "::youtube https://www.youtube.com/watch?v=jNQXAC9IVRw",
        "::youtube https://youtu.be/9bZkp7q19f0",
    ];

    c.bench_function("regex_repeated_access", |b| {
        b.iter(|| {
            // Parse multiple directives to trigger regex access
            for directive in &test_directives {
                let _ = parse_directive(black_box(directive), 1);
            }
        });
    });
}

/// Benchmark concurrent rendering (thread safety)
fn bench_concurrent_rendering(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let video_ids = Arc::new(vec![
        "dQw4w9WgXcQ",
        "jNQXAC9IVRw",
        "9bZkp7q19f0",
        "M7lc1UVf-VE",
    ]);

    c.bench_function("concurrent_rendering_4_threads", |b| {
        b.iter(|| {
            let ids = Arc::clone(&video_ids);
            let handles: Vec<_> = (0..4)
                .map(|i| {
                    let ids_clone = Arc::clone(&ids);
                    thread::spawn(move || {
                        render_youtube_embed(ids_clone[i], &WidthSpec::default())
                    })
                })
                .collect();

            for handle in handles {
                let _ = handle.join();
            }
        });
    });
}

/// Benchmark asset deduplication overhead
fn bench_asset_deduplication(c: &mut Criterion) {
    let video_ids = vec!["dQw4w9WgXcQ", "jNQXAC9IVRw", "9bZkp7q19f0"];

    c.bench_function("asset_deduplication_simulation", |b| {
        b.iter(|| {
            let mut output = String::new();
            let mut assets_included = false;

            for video_id in &video_ids {
                let html = render_youtube_embed(black_box(video_id), &WidthSpec::default());
                output.push_str(&html);

                if !assets_included {
                    output.push_str(&format!("<style>{}</style>", youtube_css()));
                    output.push_str(&format!("<script>{}</script>", youtube_js()));
                    assets_included = true;
                }
            }

            black_box(output);
        });
    });
}

criterion_group!(
    benches,
    bench_video_id_extraction,
    bench_video_id_extraction_bulk,
    bench_html_generation_single,
    bench_html_generation_bulk,
    bench_asset_access,
    bench_full_pipeline,
    bench_width_parsing,
    bench_regex_compilation_overhead,
    bench_concurrent_rendering,
    bench_asset_deduplication,
);

criterion_main!(benches);
