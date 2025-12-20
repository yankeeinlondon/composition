---
title: "Music Album Example"
description: "Example of embedding music tracks in a markdown document"
author: "Composition Library"
date: "2025-12-19"
---

# Album: Digital Harmonies

**Artist:** The Rust Orchestra
**Release Date:** 2025
**Genre:** Electronic / Experimental

## Track Listing

### 1. Compile Time
Duration: 3:42
A rhythmic exploration of type systems and borrow checking.

::audio ./tracks/01-compile-time.mp3 "Track 1: Compile Time"

### 2. Async Dreams
Duration: 4:15
Ambient soundscapes inspired by concurrent programming.

::audio ./tracks/02-async-dreams.mp3 "Track 2: Async Dreams"

### 3. Zero-Cost Abstractions
Duration: 2:58
Fast-paced electronic beats with no runtime overhead.

::audio ./tracks/03-zero-cost-abstractions.mp3 "Track 3: Zero-Cost Abstractions"

### 4. Memory Safety
Duration: 5:21
A meditative piece on the beauty of guaranteed memory safety.

::audio ./tracks/04-memory-safety.mp3 "Track 4: Memory Safety"

## Bonus Track

### 5. Fearless Concurrency
Available exclusively in this digital edition.

::audio ./tracks/05-fearless-concurrency-bonus.wav "Bonus Track: Fearless Concurrency"

## Album Notes

This album showcases the audio player's ability to:
- Handle multiple audio files in sequence
- Display custom track names
- Extract and display duration from audio metadata
- Support both MP3 and WAV formats
- Maintain consistent styling across multiple players

## Technical Details

**Audio Format:** The album uses MP3 encoding at 320kbps for tracks 1-4, and lossless WAV for the bonus track.

**Caching:** When rendered with the Composition library, metadata extraction happens once and is cached, making subsequent renders nearly instantaneous.

**Inline Mode:** Use the `--inline` flag to embed the audio as base64 data URIs, creating a completely self-contained HTML file (useful for sharing, but increases file size by ~33%).

## About This Example

**Note:** The audio files referenced in this example are placeholders. To use this example:

1. Replace placeholder paths with actual audio files
2. Ensure files are in MP3 or WAV format
3. Render the document using: `compose examples/audio/music-example.md --output ./output/`
4. Open the generated HTML in your browser

**ID3 Tags:** If your audio files contain ID3 tags (title, artist, album), they will be extracted and can be used as fallback display names if no custom name is provided.
