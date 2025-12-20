# YouTube Embedding Examples

This document demonstrates all features of the YouTube embedding directive in the DarkMatter DSL.

## Basic Usage

### Simple Embed with Video ID

The simplest form uses just the video ID (default width: 512px):

::youtube dQw4w9WgXcQ

### Embed with Watch URL

Full YouTube watch URLs are supported:

::youtube https://www.youtube.com/watch?v=jNQXAC9IVRw

### Embed with Short URL

YouTube short URLs (youtu.be) work too:

::youtube https://youtu.be/9bZkp7q19f0

## Width Specifications

### Pixel Width

Specify width in pixels for fixed-size embeds:

::youtube dQw4w9WgXcQ 800px

### Rem Width

Use rem units for responsive sizing relative to root font size:

::youtube jNQXAC9IVRw 40rem

You can also use decimal values:

::youtube 9bZkp7q19f0 32.5rem

### Percentage Width

Use percentage for fluid layouts (0-100% range):

::youtube dQw4w9WgXcQ 90%

Full width example:

::youtube jNQXAC9IVRw 100%

## URL Format Variations

### Watch URL with Query Parameters

URLs with additional query parameters are supported:

::youtube https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=share

### Embed URL Format

Direct embed URLs work:

::youtube https://www.youtube.com/embed/jNQXAC9IVRw

### /v/ URL Format

The older /v/ URL format is supported:

::youtube https://youtube.com/v/9bZkp7q19f0

### URLs without www

URLs work with or without the www subdomain:

::youtube https://youtube.com/watch?v=dQw4w9WgXcQ

## Multiple Embeds

You can include multiple YouTube embeds in the same document. The CSS and JavaScript assets are automatically deduplicated:

::youtube dQw4w9WgXcQ 600px

::youtube jNQXAC9IVRw 600px

::youtube 9bZkp7q19f0 600px

## Mixed Width Formats

Combine different width specifications in the same document:

::youtube dQw4w9WgXcQ 512px

::youtube jNQXAC9IVRw 40rem

::youtube 9bZkp7q19f0 80%

## Features

### Maximize Button

Each embed includes a maximize button (top-right corner, visible on hover) that opens the video in modal view.

### Modal View

When maximized:
- Video displays at 95vw width, centered in the viewport
- Backdrop with blur effect appears behind the video
- Click the backdrop or press Escape to minimize
- Play state is preserved during transition

### Keyboard Navigation

- **Escape**: Exit modal view
- **Tab**: Navigate to maximize button

### Accessibility

All embeds include proper ARIA labels:
- `aria-label="YouTube video player"` on iframe
- `aria-label="Maximize video"` on button

## Error Scenarios

The following examples demonstrate error handling (these will fail during parsing):

### Invalid Video ID Format

```md
::youtube invalid-id
```

Error: Could not extract video ID from 'invalid-id'.
Supported formats: youtube.com/watch?v=ID, youtu.be/ID, or 11-character ID

### Invalid Width Format

```md
::youtube dQw4w9WgXcQ 500
```

Error: Directive doesn't match pattern (width must include unit: px, rem, or %)

### Percentage Over 100

```md
::youtube dQw4w9WgXcQ 150%
```

Error: Invalid percentage '150'. Must be 0-100%

### Zero Width

```md
::youtube dQw4w9WgXcQ 0px
```

Error: Width must be positive

### Non-YouTube URL

```md
::youtube https://vimeo.com/123456
```

Error: Could not extract video ID from 'https://vimeo.com/123456'.
Supported formats: youtube.com/watch?v=ID, youtu.be/ID, or 11-character ID

## Integration with Other DarkMatter Features

YouTube embeds work seamlessly with other DarkMatter directives:

### With Headings

## My Favorite Videos

::youtube dQw4w9WgXcQ 800px

Here's another great video:

::youtube jNQXAC9IVRw 800px

### With Lists

My playlist:

- First video: ::youtube dQw4w9WgXcQ 400px
- Second video: ::youtube jNQXAC9IVRw 400px

### With Tables

| Video | Embed |
|-------|-------|
| Video 1 | ::youtube dQw4w9WgXcQ 300px |
| Video 2 | ::youtube jNQXAC9IVRw 300px |

### With Audio

Combine video and audio embeds:

::audio ./podcast.mp3 "Episode 1"

::youtube dQw4w9WgXcQ

## Browser Compatibility

- Modern browsers with ES6+ support required
- Backdrop blur requires `backdrop-filter` support (graceful degradation on older browsers)
- JavaScript must be enabled for maximize/modal functionality
- YouTube IFrame API loaded automatically

## CSP Considerations

If your site uses Content Security Policy, ensure these directives are allowed:

```
frame-src https://www.youtube.com;
script-src https://www.youtube.com;
```

## Implementation Notes

- CSS and JS assets are included once per document (automatic deduplication)
- LazyLock ensures regex compilation happens only once
- All YouTube embed URLs use HTTPS
- Video IDs are strictly validated (11 chars, alphanumeric + `-` and `_`)
- Play state preserved via YouTube IFrame API during modal transitions

## Performance

- Video ID extraction: <10ms for 1000 URLs (benchmarked)
- HTML generation: Optimized for bulk rendering
- Asset access: LazyLock provides constant-time overhead
- Thread-safe rendering for concurrent processing

---

For more details, see the [DarkMatter DSL Documentation](../docs/features/darkmatter-dsl.md#15-youtube-video-embedding).
