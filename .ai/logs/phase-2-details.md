# Phase 2: HTML Renderer Implementation - Detailed Notes

**Date:** 2025-12-20
**Status:** Complete
**Owner:** Rust Developer Sub-Agent

## Overview

Successfully implemented HTML generation for YouTube embeds with inline CSS/JS assets, maximize/modal functionality, and comprehensive test coverage.

## Files Created/Modified

### Created Files

1. **`lib/src/render/youtube.rs`** (489 lines)
   - Main rendering module for YouTube embeds
   - Public API:
     - `render_youtube_embed(video_id: &str, width: &WidthSpec) -> String`
     - `youtube_css() -> &'static str`
     - `youtube_js() -> &'static str`
   - Private functions:
     - `generate_container_html(video_id: &str, width: &WidthSpec) -> String`
     - `width_to_css(width: &WidthSpec) -> String`
   - LazyLock static assets:
     - `YOUTUBE_CSS: LazyLock<String>`
     - `YOUTUBE_JS: LazyLock<String>`

2. **Snapshot Files** (4 files created in `lib/src/render/snapshots/`)
   - `lib__render__youtube__tests__render_default_width_snapshot.snap`
   - `lib__render__youtube__tests__render_custom_pixel_width_snapshot.snap`
   - `lib__render__youtube__tests__render_rem_width_snapshot.snap`
   - `lib__render__youtube__tests__render_percentage_width_snapshot.snap`

### Modified Files

1. **`lib/src/render/mod.rs`**
   - Added `mod youtube;` declaration
   - Added public exports: `render_youtube_embed`, `youtube_css`, `youtube_js`

2. **`lib/src/render/html.rs`**
   - Added import: `use super::youtube::render_youtube_embed;`
   - Updated `DarkMatterNode::YouTube` match arm from stub error to actual rendering:
     ```rust
     DarkMatterNode::YouTube { video_id, width } => {
         Ok(render_youtube_embed(video_id, width))
     }
     ```

## Technical Implementation Details

### HTML Structure

Generated HTML follows this structure:

```html
<div class="dm-youtube-container" data-video-id="{ID}" data-width="{WIDTH}">
  <div class="dm-youtube-wrapper">
    <iframe
      class="dm-youtube-player"
      src="https://www.youtube.com/embed/{ID}?enablejsapi=1"
      frameborder="0"
      allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
      allowfullscreen
      aria-label="YouTube video player">
    </iframe>
    <button class="dm-youtube-maximize" aria-label="Maximize video">
      <svg width="24" height="24" viewBox="0 0 24 24">
        <!-- Maximize icon SVG path -->
      </svg>
    </button>
  </div>
</div>
<div class="dm-youtube-backdrop" style="display: none;"></div>
```

**Design Decisions:**

1. **Container structure** - Two-level nesting (`container` > `wrapper`) allows for proper aspect ratio maintenance while supporting modal positioning
2. **Data attributes** - Used for storing video ID and width for JavaScript interaction
3. **Separate backdrop** - Placed outside container to prevent z-index stacking issues
4. **SVG icon inline** - Maximize icon embedded directly to avoid external asset dependencies
5. **ARIA labels** - Added to iframe and button for accessibility

### CSS Implementation

**Key Features:**

1. **Aspect Ratio Maintenance**
   - Used `padding-bottom: 56.25%` on wrapper (16:9 ratio)
   - Absolute positioning of iframe within wrapper

2. **Modal State**
   - Fixed positioning at 95vw width, max-width 1600px
   - Centered using `transform: translate(-50%, -50%)`
   - High z-index (9999) to overlay content

3. **Transitions**
   - 300ms ease-in-out for smooth state changes
   - Applied to all properties for container transform

4. **Backdrop Effect**
   - Full viewport coverage
   - `backdrop-filter: blur(8px)` for visual depth
   - Semi-transparent black background
   - z-index 9998 (below modal, above content)

5. **Button Styling**
   - Hidden by default (opacity: 0)
   - Shown on hover or focus
   - Positioned top-right with absolute positioning
   - Rotates 45° in modal state to indicate "close" action

6. **Responsive Design**
   - Media query at 768px reduces modal width to 98vw
   - Uses CSS custom property `--youtube-width` for flexible sizing

**CSS Custom Property Pattern:**

Instead of inline styles, width is set via JavaScript using custom properties:
```javascript
container.style.setProperty('--youtube-width', width);
```

This approach:
- Keeps inline styles minimal
- Allows CSS cascade to work properly
- Makes it easier to override in custom stylesheets

### JavaScript Implementation

**Key Features:**

1. **YouTube IFrame API Loading**
   - Checks for existing `window.YT` to avoid duplicate loads
   - Dynamically injects API script if needed
   - Uses `onYouTubeIframeAPIReady` callback

2. **Player Instance Tracking**
   - Uses `Map<container, player>` for O(1) lookup
   - Stores YT.Player instances for state preservation
   - Unique iframe IDs generated if not present

3. **State Preservation**
   - Captures player state before maximize/minimize
   - Stores in `data-player-state` attribute
   - Restores play state after transition

4. **Original Position Tracking**
   - Uses `Map<container, originalState>` to store:
     - Parent element
     - Next sibling (for insertion)
     - Original width
     - Original position (top, left)
   - Restores DOM position on minimize

5. **Event Handling**
   - **Maximize button click** - Toggles between modal/normal state
   - **Backdrop click** - Minimizes modal
   - **Escape key** - Minimizes modal
   - All handlers use event delegation on document

6. **Width Custom Property**
   - Sets `--youtube-width` CSS variable on DOMContentLoaded
   - Reads from `data-width` attribute

**JavaScript Architecture Decisions:**

1. **IIFE wrapper** - Prevents global scope pollution
2. **Strict mode** - Catches common errors
3. **Map data structures** - Better performance than object lookup
4. **Event delegation** - Single listeners on document instead of per-element
5. **Defensive coding** - Checks for element existence before operations

### LazyLock Usage

Both CSS and JS use `std::sync::LazyLock` for one-time initialization:

```rust
static YOUTUBE_CSS: LazyLock<String> = LazyLock::new(|| {
    r#"/* CSS content */"#.to_string()
});
```

**Benefits:**

1. **Thread-safe** - Safe for concurrent access (rayon parallel rendering)
2. **One-time initialization** - Computed only once per program execution
3. **Lazy evaluation** - Only initialized when first accessed
4. **Memory efficient** - Single allocation shared across all calls
5. **Pattern consistency** - Matches existing codebase patterns (e.g., regex in parser)

**Test Verification:**

Tests confirm LazyLock behavior:
```rust
#[test]
fn test_lazylock_css_initialized_once() {
    let css1 = youtube_css();
    let css2 = youtube_css();
    assert!(std::ptr::eq(css1, css2)); // Same memory address
}
```

## Test Coverage

### Unit Tests (24 tests in youtube.rs)

**Rendering Tests:**
- `test_render_youtube_embed_contains_video_id` - Verifies video ID presence
- `test_render_youtube_embed_contains_iframe` - Checks iframe structure
- `test_render_youtube_embed_has_maximize_button` - Validates button presence
- `test_render_youtube_embed_has_backdrop` - Confirms backdrop element
- `test_render_youtube_embed_has_aria_labels` - Accessibility checks

**Width Conversion Tests:**
- `test_width_to_css_pixels` - 512px → "512px"
- `test_width_to_css_rems` - 32.0rem → "32rem"
- `test_width_to_css_percentage` - 80% → "80%"

**CSS Tests:**
- `test_youtube_css_contains_container_styles` - Basic structure
- `test_youtube_css_contains_modal_styles` - Modal positioning
- `test_youtube_css_contains_backdrop_styles` - Backdrop effects
- `test_youtube_css_contains_transitions` - Animation presence

**JavaScript Tests:**
- `test_youtube_js_loads_api` - API script loading
- `test_youtube_js_handles_maximize` - Maximize/minimize functions
- `test_youtube_js_handles_escape_key` - Keyboard navigation
- `test_youtube_js_handles_backdrop_click` - Click-to-close
- `test_youtube_js_preserves_player_state` - State preservation
- `test_youtube_js_uses_map_for_players` - Data structure usage

**LazyLock Tests:**
- `test_lazylock_css_initialized_once` - Pointer equality check
- `test_lazylock_js_initialized_once` - Pointer equality check

**Snapshot Tests (4 tests):**
- `test_render_default_width_snapshot` - 512px default width
- `test_render_custom_pixel_width_snapshot` - 800px custom width
- `test_render_rem_width_snapshot` - 32rem width
- `test_render_percentage_width_snapshot` - 80% width

### Test Results

```
running 58 tests
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured
```

**Coverage includes:**
- All YouTube rendering tests (24 tests)
- All YouTube parsing tests from Phase 1 (28 tests)
- All WidthSpec type tests from Phase 1 (6 tests)

**Code Quality:**
- Clippy: ✅ No warnings (`cargo clippy -- -D warnings`)
- Format: ✅ Properly formatted
- All acceptance criteria met

## Key Implementation Decisions

### 1. Asset Management Strategy

**Decision:** Rendering functions return HTML only; CSS/JS are separate public functions.

**Rationale:**
- Follows separation of concerns principle
- Allows orchestration layer to deduplicate assets (Phase 3)
- Makes testing easier (can test HTML structure independently)
- Matches existing codebase patterns (see popover, disclosure modules)

**Alternative Considered:**
- Including CSS/JS inline with every embed (rejected due to duplication)

### 2. Width Specification Approach

**Decision:** Use CSS custom properties set via JavaScript.

**Rationale:**
- Keeps HTML cleaner (no inline width styles)
- Allows CSS cascade to work properly
- Easier to override in user stylesheets
- Centralizes width logic in JS initialization

**Alternative Considered:**
- Inline `style="width: {width}"` on container (rejected for flexibility)

### 3. Modal Positioning Strategy

**Decision:** Move container to fixed position, track original position in Map.

**Rationale:**
- Avoids z-index issues with parent containers
- Preserves DOM structure for restoration
- Clean separation between normal and modal states
- Matches common modal implementation patterns

**Alternative Considered:**
- Clone container and insert in body (rejected due to complexity)

### 4. Player State Preservation

**Decision:** Use YouTube IFrame API to track and restore play state.

**Rationale:**
- Provides seamless user experience (video continues playing)
- Leverages official YouTube API capabilities
- Simple data attribute storage for state

**Alternative Considered:**
- Not preserving state (rejected for poor UX)
- Recording timestamp and seeking (rejected as unnecessary complexity)

### 5. Snapshot Testing

**Decision:** Use insta crate for HTML output verification.

**Rationale:**
- Catches unintended HTML structure changes
- Documents expected output format
- Quick visual review during development
- Matches project dev-dependencies (already present)

**Alternative Considered:**
- String assertion tests only (rejected as less maintainable)

## Accessibility Features

1. **ARIA Labels**
   - `aria-label="YouTube video player"` on iframe
   - `aria-label="Maximize video"` on button

2. **Keyboard Navigation**
   - Escape key to close modal
   - Focus outline on button (2px solid blue)
   - Button remains visible on focus

3. **Screen Readers**
   - Semantic HTML (button, not div with click handler)
   - Proper role attributes

4. **Color Contrast**
   - White icon on dark semi-transparent background
   - Sufficient contrast ratio for visibility

## Performance Considerations

1. **LazyLock Initialization**
   - CSS and JS compiled once per program execution
   - No regex compilation overhead (static strings)
   - Thread-safe for parallel rendering

2. **String Allocation**
   - Single format! call for HTML generation
   - Reuses WidthSpec Display trait (no extra allocation)
   - Minimal temporary strings

3. **Event Handling**
   - Event delegation (3 listeners vs N per embed)
   - Map lookups O(1) for player and state retrieval

4. **DOM Operations**
   - Batch DOM changes during maximize/minimize
   - CSS transitions for smooth performance
   - No forced reflows in JavaScript

## Browser Compatibility

**Modern Browser Support:**
- Chrome/Edge: Full support
- Firefox: Full support
- Safari: Full support (including iOS Safari)

**Progressive Enhancement:**
- `backdrop-filter` has fallback via solid background
- CSS custom properties supported in all modern browsers
- YouTube IFrame API cross-browser compatible

**Degradation:**
- If JavaScript disabled: iframe still playable, no maximize
- If CSS disabled: iframe still visible (16:9 ratio lost)

## Security Considerations

1. **XSS Prevention**
   - Video IDs validated in parser (Phase 1)
   - No user content in JavaScript strings
   - No eval() or innerHTML usage

2. **HTTPS Enforcement**
   - YouTube embed URLs use https://
   - IFrame API loaded via HTTPS

3. **CSP Compatibility**
   - Inline scripts may require CSP adjustments
   - Future: Could add nonce support for stricter CSP

4. **Sandboxing**
   - YouTube iframe inherently sandboxed by browser
   - `allow` attribute specifies permitted features

## Integration Points

### With Parser (Phase 1)

Parser provides `DarkMatterNode::YouTube { video_id, width }`:
```rust
DarkMatterNode::YouTube {
    video_id: String,  // 11-character validated ID
    width: WidthSpec,  // Pixels, Rems, or Percentage
}
```

Renderer consumes these values directly.

### With Orchestration Layer (Phase 3, Future)

Orchestrator will:
1. Track if YouTube assets already included
2. Call `youtube_css()` and `youtube_js()` once per document
3. Insert `<style id="dm-youtube">` and `<script id="dm-youtube">` in HTML head
4. Call `render_youtube_embed()` for each YouTube node

Pattern (to be implemented in Phase 3):
```rust
let mut youtube_assets_included = false;

for node in nodes {
    match node {
        DarkMatterNode::YouTube { video_id, width } => {
            output.push_str(&render_youtube_embed(video_id, width));

            if !youtube_assets_included {
                output.push_str(&format!("<style id='dm-youtube'>{}</style>", youtube_css()));
                output.push_str(&format!("<script id='dm-youtube'>{}</script>", youtube_js()));
                youtube_assets_included = true;
            }
        }
        // ...
    }
}
```

## Known Limitations

1. **Asset Deduplication Not Yet Implemented**
   - Currently, each call to `render_youtube_embed()` is independent
   - Phase 3 will implement orchestration-level deduplication
   - CSS/JS will be included once per document, not per embed

2. **No Playlist Support**
   - Only single video embeds supported
   - Deferred to v2 per plan decisions

3. **No Timestamp Parameters**
   - Start time (`?t=30s`) not yet implemented
   - Easy future enhancement (add to iframe src)

4. **No Custom Themes**
   - Single neutral design
   - User can override via custom CSS

5. **Modal Overlay Positioning**
   - Modal is always centered, not customizable
   - Could add data attributes for positioning preferences

## Files in Codebase

### Primary Implementation
- `/Volumes/coding/personal/composition/lib/src/render/youtube.rs` (489 lines)

### Modified Files
- `/Volumes/coding/personal/composition/lib/src/render/mod.rs` (export declarations)
- `/Volumes/coding/personal/composition/lib/src/render/html.rs` (renderer integration)

### Snapshot Files
- `/Volumes/coding/personal/composition/lib/src/render/snapshots/lib__render__youtube__tests__render_default_width_snapshot.snap`
- `/Volumes/coding/personal/composition/lib/src/render/snapshots/lib__render__youtube__tests__render_custom_pixel_width_snapshot.snap`
- `/Volumes/coding/personal/composition/lib/src/render/snapshots/lib__render__youtube__tests__render_rem_width_snapshot.snap`
- `/Volumes/coding/personal/composition/lib/src/render/snapshots/lib__render__youtube__tests__render_percentage_width_snapshot.snap`

## Next Steps (Phase 3)

1. **Asset Deduplication**
   - Modify `lib/src/render/html.rs` orchestration layer
   - Track `youtube_assets_included` boolean
   - Insert CSS/JS once per document

2. **Integration Testing**
   - Create `tests/youtube_integration.rs`
   - Test multiple embeds in same document
   - Verify asset deduplication works correctly

3. **Documentation Updates**
   - Update `docs/features/darkmatter-dsl.md` with YouTube section
   - Add examples to demonstrate usage

## Conclusion

Phase 2 is **complete** with all acceptance criteria met:

✅ Generate valid HTML5 markup
✅ CSS and JS use LazyLock for one-time initialization
✅ Maintain 16:9 aspect ratio in both states
✅ Support custom width specifications
✅ Include ARIA labels for accessibility
✅ Maximize button positioned correctly (top-right)
✅ Modal centers in viewport at 95vw width
✅ Unit tests in `#[cfg(test)]` module
✅ Snapshot tests verify complete HTML structure

**Test Results:**
- 58 tests passing (24 rendering + 28 parsing + 6 types)
- 0 clippy warnings
- 4 snapshot tests approved

**Code Quality:**
- Idiomatic Rust patterns
- Comprehensive documentation
- Performance optimized
- Accessibility compliant
- Security conscious

Ready to proceed to Phase 3: Asset Deduplication Integration.
