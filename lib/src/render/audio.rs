use crate::audio::{process_audio, generate_audio_html, AudioHtmlOptions, AudioInput, AudioSource, AudioProcessingConfig, AudioCache};
use crate::error::RenderError;
use crate::types::DarkMatterNode;
use std::path::{Path, PathBuf};
use surrealdb::{Surreal, engine::local::Db};
use tracing::instrument;

/// Process audio directives in a list of nodes
///
/// This function finds Audio nodes and processes them into HTML,
/// returning a new list with Audio nodes replaced by Text nodes containing HTML.
#[instrument(skip(nodes, db))]
pub async fn process_audio_nodes(
    nodes: &[DarkMatterNode],
    output_dir: &Path,
    db: &Surreal<Db>,
    inline_mode: bool,
    base_path: Option<&PathBuf>,
) -> Result<Vec<DarkMatterNode>, RenderError> {
    let mut result = Vec::new();
    let config = AudioProcessingConfig::default();
    let audio_cache = AudioCache::new(db.clone());

    for node in nodes {
        match node {
            DarkMatterNode::Audio { source, name } => {
                // Resolve relative paths
                let resolved_path = if Path::new(source).is_relative() {
                    if let Some(base) = base_path {
                        base.parent()
                            .ok_or_else(|| RenderError::InvalidPath(base.display().to_string()))?
                            .join(source)
                    } else {
                        std::env::current_dir()
                            .map_err(|e| RenderError::IoError(e.to_string()))?
                            .join(source)
                    }
                } else {
                    PathBuf::from(source)
                };

                // Create AudioInput
                let input = AudioInput {
                    source: AudioSource::Local(resolved_path.clone()),
                    name: name.clone(),
                };

                // Process audio
                match process_audio(input, output_dir, &audio_cache, inline_mode, &config).await {
                    Ok(output) => {
                        // Generate HTML
                        let html = generate_audio_html(&output, &AudioHtmlOptions::default());
                        result.push(DarkMatterNode::Text(html));
                    }
                    Err(e) => {
                        // Emit error HTML instead of failing the entire render
                        let error_html = format!(
                            r#"<div class="audio-error" style="border: 2px solid #ef4444; background: #fee2e2; color: #991b1b; padding: 1rem; border-radius: 0.5rem; margin: 1rem 0;">
                                <strong>Audio Error:</strong> {}
                            </div>"#,
                            html_escape(&e.to_string())
                        );
                        result.push(DarkMatterNode::Text(error_html));
                    }
                }
            }
            other => {
                result.push(other.clone());
            }
        }
    }

    Ok(result)
}

/// HTML escape function
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape(r#"x="y""#), "x=&quot;y&quot;");
    }
}
