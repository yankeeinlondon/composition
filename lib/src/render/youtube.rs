//! YouTube embed rendering with maximize/modal functionality
//!
//! This module generates self-contained HTML for YouTube video embeds with:
//! - 16:9 aspect ratio in default state
//! - Maximize button for modal view
//! - Backdrop blur effect in modal state
//! - Keyboard navigation (Escape to close)
//! - Play state preservation via YouTube IFrame API
//!
//! # Examples
//!
//! ```rust
//! use lib::render::youtube::render_youtube_embed;
//! use lib::types::WidthSpec;
//!
//! let html = render_youtube_embed("dQw4w9WgXcQ", &WidthSpec::Pixels(512));
//! assert!(html.contains("dm-youtube-container"));
//! ```

use crate::types::WidthSpec;
use std::sync::LazyLock;

/// Renders YouTube embed HTML for a given video ID and width.
///
/// This function generates the HTML structure for a single YouTube embed.
/// CSS and JS assets are managed separately by the orchestration layer.
///
/// # Arguments
///
/// * `video_id` - The YouTube video ID (11 characters)
/// * `width` - Width specification for the container
///
/// # Returns
///
/// HTML string containing iframe, maximize button, and backdrop elements
pub fn render_youtube_embed(video_id: &str, width: &WidthSpec) -> String {
    generate_container_html(video_id, width)
}

/// Returns the CSS required for YouTube embeds (called by orchestration layer)
pub fn youtube_css() -> &'static str {
    &YOUTUBE_CSS
}

/// Returns the JavaScript required for YouTube embeds (called by orchestration layer)
pub fn youtube_js() -> &'static str {
    &YOUTUBE_JS
}

/// Generate the container HTML with iframe and controls
fn generate_container_html(video_id: &str, width: &WidthSpec) -> String {
    let width_css = width_to_css(width);

    format!(
        r#"<div class="dm-youtube-container" data-video-id="{}" data-width="{}">
  <div class="dm-youtube-wrapper">
    <iframe
      class="dm-youtube-player"
      src="https://www.youtube.com/embed/{}?enablejsapi=1"
      frameborder="0"
      allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
      allowfullscreen
      aria-label="YouTube video player">
    </iframe>
    <button class="dm-youtube-maximize" aria-label="Maximize video">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"></path>
      </svg>
    </button>
  </div>
</div>
<div class="dm-youtube-backdrop" style="display: none;"></div>"#,
        video_id,
        width_css,
        video_id
    )
}

/// Convert WidthSpec to CSS width value
fn width_to_css(width: &WidthSpec) -> String {
    width.to_string()
}

/// CSS styles for YouTube embeds (LazyLock for one-time initialization)
static YOUTUBE_CSS: LazyLock<String> = LazyLock::new(|| {
    r#"
/* YouTube Embed Styles */
.dm-youtube-container {
  position: relative;
  width: var(--youtube-width, 512px);
  margin: 1.5rem 0;
  transition: all 300ms ease-in-out;
}

.dm-youtube-wrapper {
  position: relative;
  width: 100%;
  padding-bottom: 56.25%; /* 16:9 aspect ratio */
  overflow: hidden;
  border-radius: 8px;
  background: #000;
}

.dm-youtube-player {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  border: none;
}

.dm-youtube-maximize {
  position: absolute;
  top: 12px;
  right: 12px;
  background: rgba(0, 0, 0, 0.7);
  border: none;
  border-radius: 4px;
  color: white;
  cursor: pointer;
  padding: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 200ms ease-in-out, background 200ms ease-in-out;
  z-index: 10;
}

.dm-youtube-wrapper:hover .dm-youtube-maximize {
  opacity: 1;
}

.dm-youtube-maximize:hover {
  background: rgba(0, 0, 0, 0.9);
}

.dm-youtube-maximize:focus {
  opacity: 1;
  outline: 2px solid #3b82f6;
  outline-offset: 2px;
}

/* Modal state */
.dm-youtube-container.modal {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 95vw;
  max-width: 1600px;
  z-index: 9999;
  margin: 0;
}

.dm-youtube-container.modal .dm-youtube-maximize {
  opacity: 1;
}

.dm-youtube-container.modal .dm-youtube-maximize svg {
  transform: rotate(45deg);
}

/* Backdrop */
.dm-youtube-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(8px);
  z-index: 9998;
  cursor: pointer;
}

.dm-youtube-backdrop.show {
  display: block !important;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .dm-youtube-container.modal {
    width: 98vw;
  }
}
"#.to_string()
});

/// JavaScript for YouTube embed interactions (LazyLock for one-time initialization)
static YOUTUBE_JS: LazyLock<String> = LazyLock::new(|| {
    r#"
(function() {
  'use strict';

  // Load YouTube IFrame API
  if (!window.YT) {
    const tag = document.createElement('script');
    tag.src = 'https://www.youtube.com/iframe_api';
    const firstScriptTag = document.getElementsByTagName('script')[0];
    firstScriptTag.parentNode.insertBefore(tag, firstScriptTag);
  }

  // Track player instances
  const players = new Map();

  // Track original positions for modal mode
  const originalStates = new Map();

  // Initialize players when API is ready
  window.onYouTubeIframeAPIReady = function() {
    const iframes = document.querySelectorAll('.dm-youtube-player');
    iframes.forEach((iframe, index) => {
      const container = iframe.closest('.dm-youtube-container');
      const videoId = container.dataset.videoId;

      // Create unique player ID if not exists
      if (!iframe.id) {
        iframe.id = `youtube-player-${index}`;
      }

      const player = new YT.Player(iframe.id, {
        videoId: videoId,
        events: {
          onReady: (event) => {
            players.set(container, event.target);
          }
        }
      });
    });
  };

  // Handle maximize button clicks
  document.addEventListener('click', (e) => {
    const maximizeBtn = e.target.closest('.dm-youtube-maximize');
    if (!maximizeBtn) return;

    const container = maximizeBtn.closest('.dm-youtube-container');
    const backdrop = document.querySelector('.dm-youtube-backdrop');

    if (container.classList.contains('modal')) {
      // Minimize
      minimizeVideo(container, backdrop);
    } else {
      // Maximize
      maximizeVideo(container, backdrop);
    }
  });

  // Handle backdrop clicks
  document.addEventListener('click', (e) => {
    if (e.target.classList.contains('dm-youtube-backdrop')) {
      const modalContainer = document.querySelector('.dm-youtube-container.modal');
      if (modalContainer) {
        minimizeVideo(modalContainer, e.target);
      }
    }
  });

  // Handle Escape key
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      const modalContainer = document.querySelector('.dm-youtube-container.modal');
      const backdrop = document.querySelector('.dm-youtube-backdrop.show');
      if (modalContainer && backdrop) {
        minimizeVideo(modalContainer, backdrop);
      }
    }
  });

  function maximizeVideo(container, backdrop) {
    // Store original position and width
    const rect = container.getBoundingClientRect();
    originalStates.set(container, {
      parent: container.parentElement,
      nextSibling: container.nextElementSibling,
      width: container.dataset.width,
      top: rect.top,
      left: rect.left
    });

    // Add modal class and show backdrop
    container.classList.add('modal');
    if (backdrop) {
      backdrop.classList.add('show');
      backdrop.style.display = 'block';
    }

    // Preserve play state
    const player = players.get(container);
    if (player && typeof player.getPlayerState === 'function') {
      const state = player.getPlayerState();
      container.dataset.playerState = state;
    }
  }

  function minimizeVideo(container, backdrop) {
    const originalState = originalStates.get(container);

    // Remove modal class and hide backdrop
    container.classList.remove('modal');
    if (backdrop) {
      backdrop.classList.remove('show');
      backdrop.style.display = 'none';
    }

    // Restore original position
    if (originalState) {
      if (originalState.nextSibling) {
        originalState.parent.insertBefore(container, originalState.nextSibling);
      } else {
        originalState.parent.appendChild(container);
      }
      originalStates.delete(container);
    }

    // Preserve play state
    const player = players.get(container);
    const savedState = container.dataset.playerState;
    if (player && savedState && typeof player.playVideo === 'function') {
      if (savedState === '1') { // YT.PlayerState.PLAYING
        player.playVideo();
      }
    }
  }

  // Set width CSS custom property based on data attribute
  document.addEventListener('DOMContentLoaded', () => {
    const containers = document.querySelectorAll('.dm-youtube-container');
    containers.forEach(container => {
      const width = container.dataset.width;
      if (width) {
        container.style.setProperty('--youtube-width', width);
      }
    });
  });
})();
"#.to_string()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_youtube_embed_contains_video_id() {
        let html = render_youtube_embed("dQw4w9WgXcQ", &WidthSpec::Pixels(512));
        assert!(html.contains("dQw4w9WgXcQ"));
        assert!(html.contains(r#"data-video-id="dQw4w9WgXcQ""#));
    }

    #[test]
    fn test_render_youtube_embed_contains_iframe() {
        let html = render_youtube_embed("test123456", &WidthSpec::Pixels(512));
        assert!(html.contains("<iframe"));
        assert!(html.contains("dm-youtube-player"));
        assert!(html.contains("https://www.youtube.com/embed/test123456?enablejsapi=1"));
    }

    #[test]
    fn test_render_youtube_embed_has_maximize_button() {
        let html = render_youtube_embed("test123456", &WidthSpec::Pixels(512));
        assert!(html.contains("dm-youtube-maximize"));
        assert!(html.contains(r#"aria-label="Maximize video""#));
        assert!(html.contains("<svg"));
    }

    #[test]
    fn test_render_youtube_embed_has_backdrop() {
        let html = render_youtube_embed("test123456", &WidthSpec::Pixels(512));
        assert!(html.contains("dm-youtube-backdrop"));
        assert!(html.contains(r#"style="display: none;""#));
    }

    #[test]
    fn test_render_youtube_embed_has_aria_labels() {
        let html = render_youtube_embed("test123456", &WidthSpec::Pixels(512));
        assert!(html.contains(r#"aria-label="YouTube video player""#));
        assert!(html.contains(r#"aria-label="Maximize video""#));
    }

    #[test]
    fn test_width_to_css_pixels() {
        let width = WidthSpec::Pixels(512);
        assert_eq!(width_to_css(&width), "512px");
    }

    #[test]
    fn test_width_to_css_rems() {
        let width = WidthSpec::Rems(32.0);
        assert_eq!(width_to_css(&width), "32rem");
    }

    #[test]
    fn test_width_to_css_percentage() {
        let width = WidthSpec::Percentage(80);
        assert_eq!(width_to_css(&width), "80%");
    }

    #[test]
    fn test_youtube_css_contains_container_styles() {
        let css = youtube_css();
        assert!(css.contains(".dm-youtube-container"));
        assert!(css.contains(".dm-youtube-wrapper"));
        assert!(css.contains("padding-bottom: 56.25%")); // 16:9 aspect ratio
    }

    #[test]
    fn test_youtube_css_contains_modal_styles() {
        let css = youtube_css();
        assert!(css.contains(".dm-youtube-container.modal"));
        assert!(css.contains("position: fixed"));
        assert!(css.contains("width: 95vw"));
        assert!(css.contains("z-index: 9999"));
    }

    #[test]
    fn test_youtube_css_contains_backdrop_styles() {
        let css = youtube_css();
        assert!(css.contains(".dm-youtube-backdrop"));
        assert!(css.contains("backdrop-filter: blur(8px)"));
        assert!(css.contains("z-index: 9998"));
    }

    #[test]
    fn test_youtube_css_contains_transitions() {
        let css = youtube_css();
        assert!(css.contains("transition:"));
    }

    #[test]
    fn test_youtube_js_loads_api() {
        let js = youtube_js();
        assert!(js.contains("https://www.youtube.com/iframe_api"));
        assert!(js.contains("window.YT"));
    }

    #[test]
    fn test_youtube_js_handles_maximize() {
        let js = youtube_js();
        assert!(js.contains("dm-youtube-maximize"));
        assert!(js.contains("maximizeVideo"));
        assert!(js.contains("minimizeVideo"));
    }

    #[test]
    fn test_youtube_js_handles_escape_key() {
        let js = youtube_js();
        assert!(js.contains("keydown"));
        assert!(js.contains("Escape"));
    }

    #[test]
    fn test_youtube_js_handles_backdrop_click() {
        let js = youtube_js();
        assert!(js.contains("dm-youtube-backdrop"));
        assert!(js.contains("click"));
    }

    #[test]
    fn test_youtube_js_preserves_player_state() {
        let js = youtube_js();
        assert!(js.contains("playerState"));
        assert!(js.contains("getPlayerState"));
    }

    #[test]
    fn test_youtube_js_uses_map_for_players() {
        let js = youtube_js();
        assert!(js.contains("new Map()"));
        assert!(js.contains("players"));
    }

    #[test]
    fn test_lazylock_css_initialized_once() {
        let css1 = youtube_css();
        let css2 = youtube_css();
        assert!(std::ptr::eq(css1, css2));
    }

    #[test]
    fn test_lazylock_js_initialized_once() {
        let js1 = youtube_js();
        let js2 = youtube_js();
        assert!(std::ptr::eq(js1, js2));
    }

    // Snapshot tests
    #[test]
    fn test_render_default_width_snapshot() {
        let html = render_youtube_embed("dQw4w9WgXcQ", &WidthSpec::default());
        insta::assert_snapshot!(html);
    }

    #[test]
    fn test_render_custom_pixel_width_snapshot() {
        let html = render_youtube_embed("dQw4w9WgXcQ", &WidthSpec::Pixels(800));
        insta::assert_snapshot!(html);
    }

    #[test]
    fn test_render_rem_width_snapshot() {
        let html = render_youtube_embed("dQw4w9WgXcQ", &WidthSpec::Rems(32.0));
        insta::assert_snapshot!(html);
    }

    #[test]
    fn test_render_percentage_width_snapshot() {
        let html = render_youtube_embed("dQw4w9WgXcQ", &WidthSpec::Percentage(80));
        insta::assert_snapshot!(html);
    }
}
