//! Integration tests for YouTube embedding feature
//!
//! These tests verify end-to-end functionality including:
//! - Full markdown to HTML conversion with YouTube directives
//! - Multiple YouTube embeds in same document
//! - YouTube embeds with other DarkMatter directives
//! - Error propagation end-to-end
//! - Asset deduplication across multiple embeds

use lib::parse::parse_directive;
use lib::render::{render_youtube_embed, youtube_css, youtube_js};
use lib::types::{DarkMatterNode, WidthSpec};

#[test]
fn test_single_youtube_embed_full_pipeline() {
    // Parse the directive
    let result = parse_directive("::youtube dQw4w9WgXcQ 800px", 1);
    assert!(result.is_ok());

    let node = result.unwrap();
    assert!(node.is_some());

    match node.unwrap() {
        DarkMatterNode::YouTube { video_id, width } => {
            assert_eq!(video_id, "dQw4w9WgXcQ");
            assert_eq!(width, WidthSpec::Pixels(800));

            // Render the HTML
            let html = render_youtube_embed(&video_id, &width);
            assert!(html.contains("dQw4w9WgXcQ"));
            assert!(html.contains(r#"data-width="800px""#));
            assert!(html.contains("dm-youtube-container"));
        }
        _ => panic!("Expected YouTube node"),
    }
}

#[test]
fn test_multiple_youtube_embeds_in_document() {
    // Parse multiple YouTube directives
    let directives = [
        "::youtube dQw4w9WgXcQ",
        "::youtube jNQXAC9IVRw 640px",
        "::youtube 9bZkp7q19f0 90%",
    ];

    let mut nodes = Vec::new();
    for (idx, directive) in directives.iter().enumerate() {
        let result = parse_directive(directive, idx + 1);
        assert!(result.is_ok(), "Failed to parse: {}", directive);
        let node = result.unwrap();
        assert!(node.is_some(), "No node returned for: {}", directive);
        nodes.push(node.unwrap());
    }

    // Verify all nodes parsed correctly
    assert_eq!(nodes.len(), 3);

    // Render all embeds
    let mut html_output = String::new();
    for node in &nodes {
        match node {
            DarkMatterNode::YouTube { video_id, width } => {
                html_output.push_str(&render_youtube_embed(video_id, width));
                html_output.push('\n');
            }
            _ => panic!("Expected YouTube node"),
        }
    }

    // Verify all embeds present
    assert!(html_output.contains("dQw4w9WgXcQ"));
    assert!(html_output.contains("jNQXAC9IVRw"));
    assert!(html_output.contains("9bZkp7q19f0"));

    // Verify different widths
    assert!(html_output.contains(r#"data-width="512px""#)); // default
    assert!(html_output.contains(r#"data-width="640px""#));
    assert!(html_output.contains(r#"data-width="90%""#));
}

#[test]
fn test_asset_deduplication_simulation() {
    // Simulate orchestration layer behavior with multiple embeds
    let video_ids = vec!["dQw4w9WgXcQ", "jNQXAC9IVRw", "9bZkp7q19f0"];
    let mut output = String::new();
    let mut youtube_assets_included = false;

    // Render each embed
    for video_id in &video_ids {
        let embed_html = render_youtube_embed(video_id, &WidthSpec::default());
        output.push_str(&embed_html);
        output.push('\n');

        // Include assets only once (simulating orchestration layer)
        if !youtube_assets_included {
            output.push_str(&format!(
                "\n<style id=\"dm-youtube\">{}</style>\n",
                youtube_css()
            ));
            output.push_str(&format!(
                "\n<script id=\"dm-youtube\">{}</script>\n",
                youtube_js()
            ));
            youtube_assets_included = true;
        }
    }

    // Verify CSS included only once
    let css_count = output.matches(r#"<style id="dm-youtube">"#).count();
    assert_eq!(
        css_count, 1,
        "CSS should be included only once, found {} times",
        css_count
    );

    // Verify JS included only once
    let js_count = output.matches(r#"<script id="dm-youtube">"#).count();
    assert_eq!(
        js_count, 1,
        "JS should be included only once, found {} times",
        js_count
    );

    // Verify all embeds present (each has <div class="dm-youtube-container">)
    assert_eq!(output.matches(r#"<div class="dm-youtube-container""#).count(), 3);
    assert!(output.contains("dQw4w9WgXcQ"));
    assert!(output.contains("jNQXAC9IVRw"));
    assert!(output.contains("9bZkp7q19f0"));
}

#[test]
fn test_youtube_with_different_url_formats() {
    let url_formats = vec![
        ("::youtube https://www.youtube.com/watch?v=dQw4w9WgXcQ", "dQw4w9WgXcQ"),
        ("::youtube https://youtu.be/dQw4w9WgXcQ", "dQw4w9WgXcQ"),
        ("::youtube https://www.youtube.com/embed/dQw4w9WgXcQ", "dQw4w9WgXcQ"),
        ("::youtube https://youtube.com/v/dQw4w9WgXcQ", "dQw4w9WgXcQ"),
        ("::youtube dQw4w9WgXcQ", "dQw4w9WgXcQ"),
    ];

    for (directive, expected_id) in url_formats {
        let result = parse_directive(directive, 1);
        assert!(result.is_ok(), "Failed to parse: {}", directive);

        let node = result.unwrap();
        assert!(node.is_some(), "No node for: {}", directive);

        match node.unwrap() {
            DarkMatterNode::YouTube { video_id, width: _ } => {
                assert_eq!(
                    video_id, expected_id,
                    "Wrong video ID for directive: {}",
                    directive
                );
            }
            _ => panic!("Expected YouTube node for: {}", directive),
        }
    }
}

#[test]
fn test_youtube_with_different_width_formats() {
    let width_formats = vec![
        ("::youtube dQw4w9WgXcQ 512px", WidthSpec::Pixels(512)),
        ("::youtube dQw4w9WgXcQ 32rem", WidthSpec::Rems(32.0)),
        ("::youtube dQw4w9WgXcQ 32.5rem", WidthSpec::Rems(32.5)),
        ("::youtube dQw4w9WgXcQ 80%", WidthSpec::Percentage(80)),
        ("::youtube dQw4w9WgXcQ 100%", WidthSpec::Percentage(100)),
        ("::youtube dQw4w9WgXcQ 0%", WidthSpec::Percentage(0)),
        ("::youtube dQw4w9WgXcQ", WidthSpec::Pixels(512)), // default
    ];

    for (directive, expected_width) in width_formats {
        let result = parse_directive(directive, 1);
        assert!(result.is_ok(), "Failed to parse: {}", directive);

        let node = result.unwrap();
        assert!(node.is_some(), "No node for: {}", directive);

        match node.unwrap() {
            DarkMatterNode::YouTube { video_id: _, width } => {
                assert_eq!(
                    width, expected_width,
                    "Wrong width for directive: {}",
                    directive
                );
            }
            _ => panic!("Expected YouTube node for: {}", directive),
        }
    }
}

#[test]
fn test_error_propagation_invalid_video_id() {
    let result = parse_directive("::youtube invalid-id", 1);
    assert!(result.is_err(), "Should error on invalid video ID");

    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("Could not extract video ID"),
        "Error message should mention video ID extraction failure: {}",
        err_msg
    );
}

#[test]
fn test_error_propagation_invalid_width() {
    let result = parse_directive("::youtube dQw4w9WgXcQ 150%", 1);
    assert!(result.is_err(), "Should error on invalid width (>100%)");

    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("Invalid percentage"),
        "Error message should mention invalid percentage: {}",
        err_msg
    );
}

#[test]
fn test_error_propagation_malformed_url() {
    let result = parse_directive("::youtube https://vimeo.com/123456", 1);
    assert!(result.is_err(), "Should error on non-YouTube URL");

    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("Could not extract video ID"),
        "Error message should mention extraction failure: {}",
        err_msg
    );
}

#[test]
fn test_youtube_html_structure_complete() {
    let html = render_youtube_embed("dQw4w9WgXcQ", &WidthSpec::Pixels(512));

    // Verify complete HTML structure
    assert!(html.contains("<div class=\"dm-youtube-container\""));
    assert!(html.contains("<div class=\"dm-youtube-wrapper\">"));
    assert!(html.contains("<iframe"));
    assert!(html.contains("class=\"dm-youtube-player\""));
    assert!(html.contains("<button class=\"dm-youtube-maximize\""));
    assert!(html.contains("<svg"));
    assert!(html.contains("<div class=\"dm-youtube-backdrop\""));

    // Verify iframe attributes
    assert!(html.contains("src=\"https://www.youtube.com/embed/dQw4w9WgXcQ?enablejsapi=1\""));
    assert!(html.contains("frameborder=\"0\""));
    assert!(html.contains("allowfullscreen"));

    // Verify ARIA labels
    assert!(html.contains(r#"aria-label="YouTube video player""#));
    assert!(html.contains(r#"aria-label="Maximize video""#));

    // Verify data attributes
    assert!(html.contains(r#"data-video-id="dQw4w9WgXcQ""#));
    assert!(html.contains(r#"data-width="512px""#));
}

#[test]
fn test_youtube_css_structure() {
    let css = youtube_css();

    // Verify key CSS selectors exist
    assert!(css.contains(".dm-youtube-container"));
    assert!(css.contains(".dm-youtube-wrapper"));
    assert!(css.contains(".dm-youtube-player"));
    assert!(css.contains(".dm-youtube-maximize"));
    assert!(css.contains(".dm-youtube-backdrop"));

    // Verify modal state
    assert!(css.contains(".dm-youtube-container.modal"));

    // Verify 16:9 aspect ratio
    assert!(css.contains("padding-bottom: 56.25%"));

    // Verify responsive design
    assert!(css.contains("@media"));
}

#[test]
fn test_youtube_js_structure() {
    let js = youtube_js();

    // Verify API loading
    assert!(js.contains("https://www.youtube.com/iframe_api"));
    assert!(js.contains("window.YT"));

    // Verify player management
    assert!(js.contains("new Map()"));
    assert!(js.contains("players"));

    // Verify event handlers
    assert!(js.contains("maximizeVideo"));
    assert!(js.contains("minimizeVideo"));
    assert!(js.contains("keydown"));
    assert!(js.contains("Escape"));

    // Verify backdrop handling
    assert!(js.contains("dm-youtube-backdrop"));

    // Verify player state preservation
    assert!(js.contains("playerState"));
    assert!(js.contains("getPlayerState"));
}

#[test]
fn test_youtube_assets_are_static() {
    // Verify assets are initialized once and reused
    let css1 = youtube_css();
    let css2 = youtube_css();
    assert!(std::ptr::eq(css1, css2), "CSS should be same reference");

    let js1 = youtube_js();
    let js2 = youtube_js();
    assert!(std::ptr::eq(js1, js2), "JS should be same reference");
}

#[test]
fn test_mixed_darkmatter_directives() {
    // Test parsing multiple different DarkMatter directives in sequence
    let directives = vec![
        ("::youtube dQw4w9WgXcQ", true),
        ("::file ./some-file.md", true),
        ("::table ./data.csv", true),
        ("::youtube jNQXAC9IVRw 640px", true),
        ("::summarize ./doc.md", true),
        ("::youtube 9bZkp7q19f0 90%", true),
    ];

    for (directive, should_parse) in directives {
        let result = parse_directive(directive, 1);
        if should_parse {
            assert!(
                result.is_ok(),
                "Failed to parse directive: {}",
                directive
            );
            assert!(
                result.unwrap().is_some(),
                "No node returned for: {}",
                directive
            );
        }
    }
}

#[test]
fn test_url_with_query_parameters() {
    // YouTube URLs with additional query parameters
    let result = parse_directive(
        "::youtube https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=share",
        1,
    );
    assert!(result.is_ok());

    match result.unwrap().unwrap() {
        DarkMatterNode::YouTube { video_id, width: _ } => {
            assert_eq!(video_id, "dQw4w9WgXcQ");
        }
        _ => panic!("Expected YouTube node"),
    }
}

#[test]
fn test_render_preserves_video_id_integrity() {
    // Ensure video ID is not modified during rendering
    let test_ids = vec![
        "dQw4w9WgXcQ",
        "jNQXAC9IVRw",
        "9bZkp7q19f0",
        "aBcDeFgHiJk", // Mixed case
        "0123456789_", // Numbers and underscore
        "abcdefgh-jk", // With hyphen
    ];

    for video_id in test_ids {
        let html = render_youtube_embed(video_id, &WidthSpec::default());

        // Video ID should appear exactly twice (in data attribute and iframe src)
        let id_count = html.matches(video_id).count();
        assert_eq!(
            id_count, 2,
            "Video ID '{}' should appear exactly twice in HTML",
            video_id
        );

        // Verify in expected locations
        assert!(
            html.contains(&format!(r#"data-video-id="{}""#, video_id)),
            "Missing data-video-id for {}",
            video_id
        );
        assert!(
            html.contains(&format!(
                "https://www.youtube.com/embed/{}?enablejsapi=1",
                video_id
            )),
            "Missing iframe src for {}",
            video_id
        );
    }
}

#[test]
fn test_width_specification_css_conversion() {
    let test_cases = vec![
        (WidthSpec::Pixels(512), "512px"),
        (WidthSpec::Pixels(800), "800px"),
        (WidthSpec::Rems(32.0), "32rem"),
        (WidthSpec::Rems(32.5), "32.5rem"),
        (WidthSpec::Percentage(80), "80%"),
        (WidthSpec::Percentage(100), "100%"),
    ];

    for (width_spec, expected_css) in test_cases {
        let html = render_youtube_embed("dQw4w9WgXcQ", &width_spec);
        assert!(
            html.contains(&format!(r#"data-width="{}""#, expected_css)),
            "Expected data-width=\"{}\" for {:?}",
            expected_css,
            width_spec
        );
    }
}

#[test]
fn test_no_html_injection_in_video_id() {
    // Malicious video IDs should fail validation before rendering
    let malicious_ids = vec![
        "<script>alert('xss')</script>",
        "dQw4w9WgXcQ\"><script>alert('xss')</script>",
        "'; DROP TABLE videos; --",
    ];

    for malicious_id in malicious_ids {
        // These should fail during parsing (video ID validation)
        let directive = format!("::youtube {}", malicious_id);
        let result = parse_directive(&directive, 1);

        // Should either fail validation or not match directive pattern
        assert!(
            result.is_err() || result.unwrap().is_none(),
            "Malicious ID should not parse: {}",
            malicious_id
        );
    }
}

#[test]
fn test_asset_content_non_empty() {
    let css = youtube_css();
    let js = youtube_js();

    assert!(!css.is_empty(), "CSS should not be empty");
    assert!(!js.is_empty(), "JS should not be empty");

    // Verify minimum content length (sanity check)
    assert!(css.len() > 100, "CSS seems too short");
    assert!(js.len() > 100, "JS seems too short");
}

#[test]
fn test_concurrent_rendering() {
    // Test that rendering can be done concurrently (thread-safe LazyLock)
    use std::sync::Arc;
    use std::thread;

    let video_ids = Arc::new(vec!["dQw4w9WgXcQ", "jNQXAC9IVRw", "9bZkp7q19f0"]);

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let ids = Arc::clone(&video_ids);
            thread::spawn(move || {
                let video_id = ids[i];
                render_youtube_embed(video_id, &WidthSpec::default())
            })
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    assert_eq!(results.len(), 3);
    for (i, html) in results.iter().enumerate() {
        assert!(html.contains(video_ids[i]));
    }
}
