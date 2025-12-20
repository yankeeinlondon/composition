//! HTML generation for audio players
//!
//! This module provides functionality to generate HTML5 `<audio>` player markup
//! from processed audio files.
//!
//! # Examples
//!
//! ```no_run
//! use lib::audio::types::{AudioOutput, AudioFormat, AudioMetadata};
//! use lib::audio::html::{generate_audio_html, AudioHtmlOptions};
//!
//! let output = AudioOutput {
//!     format: AudioFormat::Mp3,
//!     metadata: AudioMetadata::default(),
//!     path: "audio/abc123.mp3".to_string(),
//!     base64_data: None,
//!     display_name: "Podcast Episode".to_string(),
//! };
//!
//! let options = AudioHtmlOptions {
//!     inline: false,
//!     class: None,
//! };
//!
//! let html = generate_audio_html(&output, &options);
//! ```

use crate::audio::types::AudioOutput;

/// Options for HTML generation
#[derive(Debug, Clone, Default)]
pub struct AudioHtmlOptions {
    /// Whether to use inline base64 data URIs (true) or file references (false)
    pub inline: bool,
    /// Optional custom CSS class to add to the audio player container
    pub class: Option<String>,
}

/// Generate HTML5 audio player markup from processed audio output
///
/// # Arguments
///
/// * `output` - The processed audio output containing format, metadata, and path/data
/// * `options` - HTML generation options (inline mode, custom CSS class)
///
/// # Returns
///
/// A string containing the complete HTML5 audio player markup
///
/// # Examples
///
/// ```no_run
/// use lib::audio::types::{AudioOutput, AudioFormat, AudioMetadata};
/// use lib::audio::html::{generate_audio_html, AudioHtmlOptions};
///
/// let output = AudioOutput {
///     format: AudioFormat::Mp3,
///     metadata: AudioMetadata {
///         duration_secs: Some(123.45),
///         ..Default::default()
///     },
///     path: "audio/abc123.mp3".to_string(),
///     base64_data: None,
///     display_name: "Episode 1".to_string(),
/// };
///
/// let html = generate_audio_html(&output, &AudioHtmlOptions::default());
/// assert!(html.contains("<audio"));
/// assert!(html.contains("Episode 1"));
/// ```
pub fn generate_audio_html(output: &AudioOutput, options: &AudioHtmlOptions) -> String {
    // Determine the source attribute (file reference or data URI)
    let src = if options.inline {
        // Use base64 data URI if available
        if let Some(base64) = &output.base64_data {
            format!("data:{};base64,{}", output.format.mime_type(), base64)
        } else {
            // Fallback to file reference if base64 is not available
            html_escape(&output.path)
        }
    } else {
        // Use file reference
        html_escape(&output.path)
    };

    // Format duration as mm:ss
    let duration_html = if let Some(duration_secs) = output.metadata.duration_secs {
        let minutes = (duration_secs / 60.0).floor() as u32;
        let seconds = (duration_secs % 60.0).floor() as u32;
        format!(
            r#"<span class="audio-duration">{}:{:02}</span>"#,
            minutes, seconds
        )
    } else {
        String::new()
    };

    // Determine container class
    let container_class = if let Some(custom_class) = &options.class {
        format!("audio-player {}", html_escape(custom_class))
    } else {
        "audio-player".to_string()
    };

    // Escape display name to prevent XSS
    let display_name = html_escape(&output.display_name);

    // Generate HTML structure
    format!(
        r#"<div class="{}">
  <audio controls preload="metadata">
    <source src="{}" type="{}">
    Your browser does not support the audio element.
  </audio>
  <div class="audio-info">
    <span class="audio-name">{}</span>
    {}
  </div>
</div>"#,
        container_class,
        src,
        output.format.mime_type(),
        display_name,
        duration_html
    )
}

/// Escape HTML special characters to prevent XSS attacks
///
/// This function escapes the following characters:
/// - `&` → `&amp;`
/// - `<` → `&lt;`
/// - `>` → `&gt;`
/// - `"` → `&quot;`
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// A new string with HTML special characters escaped
///
/// # Examples
///
/// ```
/// use lib::audio::html::html_escape;
///
/// assert_eq!(html_escape("Hello"), "Hello");
/// assert_eq!(html_escape("<script>"), "&lt;script&gt;");
/// assert_eq!(html_escape("A & B"), "A &amp; B");
/// assert_eq!(html_escape(r#"Click "here""#), "Click &quot;here&quot;");
/// ```
pub fn html_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::types::{AudioFormat, AudioMetadata};

    #[test]
    fn html_escape_preserves_safe_characters() {
        assert_eq!(html_escape("Hello World"), "Hello World");
        assert_eq!(html_escape("abc123"), "abc123");
        assert_eq!(html_escape(""), "");
    }

    #[test]
    fn html_escape_escapes_ampersand() {
        assert_eq!(html_escape("A & B"), "A &amp; B");
        assert_eq!(html_escape("&"), "&amp;");
        assert_eq!(html_escape("&&"), "&amp;&amp;");
    }

    #[test]
    fn html_escape_escapes_angle_brackets() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("<"), "&lt;");
        assert_eq!(html_escape(">"), "&gt;");
        assert_eq!(html_escape("<div>"), "&lt;div&gt;");
    }

    #[test]
    fn html_escape_escapes_quotes() {
        assert_eq!(html_escape(r#"Click "here""#), "Click &quot;here&quot;");
        assert_eq!(html_escape(r#"""#), "&quot;");
    }

    #[test]
    fn html_escape_prevents_xss() {
        // Test common XSS vectors
        assert_eq!(
            html_escape(r#"<script>alert('XSS')</script>"#),
            "&lt;script&gt;alert('XSS')&lt;/script&gt;"
        );
        assert_eq!(
            html_escape(r#"" onclick="alert('XSS')""#),
            "&quot; onclick=&quot;alert('XSS')&quot;"
        );
        assert_eq!(
            html_escape("<img src=x onerror=alert('XSS')>"),
            "&lt;img src=x onerror=alert('XSS')&gt;"
        );
    }

    #[test]
    fn html_escape_handles_complex_strings() {
        assert_eq!(
            html_escape(r#"<tag attr="value"> & "text""#),
            r#"&lt;tag attr=&quot;value&quot;&gt; &amp; &quot;text&quot;"#
        );
    }

    #[test]
    fn generate_audio_html_file_reference_mode() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata {
                duration_secs: Some(123.0),
                bitrate: None,
                sample_rate: None,
                channels: None,
                title: None,
                artist: None,
                album: None,
            },
            path: "audio/abc123.mp3".to_string(),
            base64_data: None,
            display_name: "Test Audio".to_string(),
        };

        let options = AudioHtmlOptions {
            inline: false,
            class: None,
        };

        let html = generate_audio_html(&output, &options);

        // Verify HTML structure
        assert!(html.contains(r#"<div class="audio-player">"#));
        assert!(html.contains(r#"<audio controls preload="metadata">"#));
        assert!(html.contains(r#"<source src="audio/abc123.mp3" type="audio/mpeg">"#));
        assert!(html.contains("Your browser does not support the audio element."));
        assert!(html.contains(r#"<span class="audio-name">Test Audio</span>"#));
        assert!(html.contains(r#"<span class="audio-duration">2:03</span>"#));
    }

    #[test]
    fn generate_audio_html_inline_mode_with_base64() {
        let output = AudioOutput {
            format: AudioFormat::Wav,
            metadata: AudioMetadata {
                duration_secs: Some(5.5),
                ..Default::default()
            },
            path: "audio/def456.wav".to_string(),
            base64_data: Some("AAAABBBBCCCC".to_string()),
            display_name: "Short Clip".to_string(),
        };

        let options = AudioHtmlOptions {
            inline: true,
            class: None,
        };

        let html = generate_audio_html(&output, &options);

        // Verify inline data URI
        assert!(html.contains(r#"src="data:audio/wav;base64,AAAABBBBCCCC""#));
        assert!(html.contains(r#"type="audio/wav""#));
        assert!(html.contains(r#"<span class="audio-name">Short Clip</span>"#));
        assert!(html.contains(r#"<span class="audio-duration">0:05</span>"#));
    }

    #[test]
    fn generate_audio_html_inline_mode_without_base64_fallback() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata::default(),
            path: "audio/fallback.mp3".to_string(),
            base64_data: None, // No base64 data available
            display_name: "Fallback".to_string(),
        };

        let options = AudioHtmlOptions {
            inline: true,
            class: None,
        };

        let html = generate_audio_html(&output, &options);

        // Should fallback to file reference
        assert!(html.contains(r#"src="audio/fallback.mp3""#));
    }

    #[test]
    fn generate_audio_html_custom_class() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata::default(),
            path: "audio/test.mp3".to_string(),
            base64_data: None,
            display_name: "Test".to_string(),
        };

        let options = AudioHtmlOptions {
            inline: false,
            class: Some("custom-player".to_string()),
        };

        let html = generate_audio_html(&output, &options);

        assert!(html.contains(r#"<div class="audio-player custom-player">"#));
    }

    #[test]
    fn generate_audio_html_escapes_display_name() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata::default(),
            path: "audio/test.mp3".to_string(),
            base64_data: None,
            display_name: r#"<script>alert("XSS")</script>"#.to_string(),
        };

        let html = generate_audio_html(&output, &AudioHtmlOptions::default());

        // Display name should be escaped
        assert!(html.contains("&lt;script&gt;alert(&quot;XSS&quot;)&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn generate_audio_html_escapes_path() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata::default(),
            path: r#"audio/test" onclick="alert('XSS')".mp3"#.to_string(),
            base64_data: None,
            display_name: "Test".to_string(),
        };

        let html = generate_audio_html(&output, &AudioHtmlOptions::default());

        // Path should be escaped
        assert!(html.contains("&quot; onclick=&quot;alert('XSS')&quot;"));
        assert!(!html.contains(r#"" onclick=""#));
    }

    #[test]
    fn generate_audio_html_escapes_custom_class() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata::default(),
            path: "audio/test.mp3".to_string(),
            base64_data: None,
            display_name: "Test".to_string(),
        };

        let options = AudioHtmlOptions {
            inline: false,
            class: Some(r#"malicious" onclick="alert('XSS')""#.to_string()),
        };

        let html = generate_audio_html(&output, &options);

        // Custom class should be escaped
        assert!(html.contains(r#"class="audio-player malicious&quot; onclick=&quot;alert('XSS')&quot;""#));
    }

    #[test]
    fn duration_formatting_zero_seconds() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata {
                duration_secs: Some(0.0),
                ..Default::default()
            },
            path: "audio/test.mp3".to_string(),
            base64_data: None,
            display_name: "Zero Duration".to_string(),
        };

        let html = generate_audio_html(&output, &AudioHtmlOptions::default());
        assert!(html.contains(r#"<span class="audio-duration">0:00</span>"#));
    }

    #[test]
    fn duration_formatting_under_one_minute() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata {
                duration_secs: Some(59.0),
                ..Default::default()
            },
            path: "audio/test.mp3".to_string(),
            base64_data: None,
            display_name: "59 Seconds".to_string(),
        };

        let html = generate_audio_html(&output, &AudioHtmlOptions::default());
        assert!(html.contains(r#"<span class="audio-duration">0:59</span>"#));
    }

    #[test]
    fn duration_formatting_exactly_one_minute() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata {
                duration_secs: Some(60.0),
                ..Default::default()
            },
            path: "audio/test.mp3".to_string(),
            base64_data: None,
            display_name: "One Minute".to_string(),
        };

        let html = generate_audio_html(&output, &AudioHtmlOptions::default());
        assert!(html.contains(r#"<span class="audio-duration">1:00</span>"#));
    }

    #[test]
    fn duration_formatting_over_one_hour() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata {
                duration_secs: Some(3661.0), // 61 minutes, 1 second
                ..Default::default()
            },
            path: "audio/test.mp3".to_string(),
            base64_data: None,
            display_name: "Long Audio".to_string(),
        };

        let html = generate_audio_html(&output, &AudioHtmlOptions::default());
        assert!(html.contains(r#"<span class="audio-duration">61:01</span>"#));
    }

    #[test]
    fn duration_none_omits_duration_span() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata {
                duration_secs: None,
                ..Default::default()
            },
            path: "audio/test.mp3".to_string(),
            base64_data: None,
            display_name: "No Duration".to_string(),
        };

        let html = generate_audio_html(&output, &AudioHtmlOptions::default());
        assert!(!html.contains("audio-duration"));
    }

    // Snapshot tests using insta
    #[test]
    fn snapshot_file_reference_mode() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata {
                duration_secs: Some(125.5),
                bitrate: Some(192000),
                sample_rate: Some(44100),
                channels: Some(2),
                title: Some("Test Track".to_string()),
                artist: Some("Test Artist".to_string()),
                album: Some("Test Album".to_string()),
            },
            path: "audio/abc123def456.mp3".to_string(),
            base64_data: None,
            display_name: "Test Track".to_string(),
        };

        let html = generate_audio_html(&output, &AudioHtmlOptions::default());
        insta::assert_snapshot!(html);
    }

    #[test]
    fn snapshot_inline_mode() {
        let output = AudioOutput {
            format: AudioFormat::Wav,
            metadata: AudioMetadata {
                duration_secs: Some(3.2),
                bitrate: None,
                sample_rate: Some(48000),
                channels: Some(1),
                title: None,
                artist: None,
                album: None,
            },
            path: "audio/short.wav".to_string(),
            base64_data: Some("VGVzdEJhc2U2NERhdGE=".to_string()),
            display_name: "Short Sound Effect".to_string(),
        };

        let options = AudioHtmlOptions {
            inline: true,
            class: None,
        };

        let html = generate_audio_html(&output, &options);
        insta::assert_snapshot!(html);
    }

    #[test]
    fn snapshot_custom_class() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata {
                duration_secs: Some(180.0),
                ..Default::default()
            },
            path: "audio/podcast.mp3".to_string(),
            base64_data: None,
            display_name: "Podcast Episode 1".to_string(),
        };

        let options = AudioHtmlOptions {
            inline: false,
            class: Some("podcast-player dark-theme".to_string()),
        };

        let html = generate_audio_html(&output, &options);
        insta::assert_snapshot!(html);
    }

    #[test]
    fn snapshot_no_duration() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata {
                duration_secs: None,
                ..Default::default()
            },
            path: "audio/unknown.mp3".to_string(),
            base64_data: None,
            display_name: "Unknown Duration".to_string(),
        };

        let html = generate_audio_html(&output, &AudioHtmlOptions::default());
        insta::assert_snapshot!(html);
    }
}
