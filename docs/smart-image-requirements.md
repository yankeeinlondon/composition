# Smart Image Requirements

## Overview

The smart image feature of the `composition` library is intended to make sure that consumers of the content which this library provides are getting an optimized image loading experience.

The primary idea is that we use **HTML**'s [`<scrset>`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLImageElement/srcset) functionality to provide a _grid_ of image options that optimize for both the viewer's viewport size and the browser's image support.

## Image Formats by Preference

- `avif` images are the best images when they are supported by the browser
- following that `webp` is superior to both `jpg` and `png`
- if we must the we will fall back to `jpg` or `png` formats

The goal is to have a `avif`, `webp`, and a `jpg` variant for each input image (see exceptions in outliers section).

### Outliers

- when a `png` image is presented as an input image we can not use `jpg` as a fallback because it `jpg` does not support transparency (but `avif` and `webp` do); so in these cases the final output images will be `png`,`avif`, and `webp`
- the photoshop `.psd` format is popular amongst designers but terrible for the web; in the rare cases this is what we're passed as input then it is a "nice-to-have" that we can convert to `avif`, `webp`, and `jpg` variants. If this proves too hard we will just not allow this file format.
- the `.gif` format remains popular today and if we can support resizing then we will but we will not convert to other image formats when it is provided as an input.

## Image Sizes

For every input image:

1. We always start by converting the input image to the other image formats at the same resolution as the input image:

    - `{{hash}}.jpg` or `{{hash}}.png`
    - `{{hash}}.avif`
    - `{{hash}}.webp`

2. We will never **upsample** an image but we will always create (_or symlink_) the following variant image files:
   - `{{hash}}-micro.{{img format}}`,
   - `{{hash}}-sm.{{img format}}`,
   - `{{hash}}-med.{{img format}}`,
   - `{{hash}}-lg.{{img format}}`,
   - `{{hash}}-xl.{{img format}}`,
   - `{{hash}}-2xl.{{img format}}`
3. The micro/sm/med/lg/xl/2xl sizes will be measured as horizontal pixels and by default will be set to:
   - **micro** - 320px
   - **xs** - 640px
   - **sm** - 1280px
   - **md** - 1536px
   - **lg** - 2048px
   - **xl** - 2560px
   - **2xl** - 3072px
4. The user will be able to override these settings by setting the breakpoints for pages. For each [breakpoints](./breakpoints.md) level the corresponding image width is _double_ the breakpoint (as many/most displays now display at twice the resolution as the pixels they present to the viewport width). The exception is the `xs` and `micro` sizes which don't exist as **breakpoints**:
   - the `sm` image width will equal the breakpoint value `sm` (not multiplied by 2).
   - the `micro` image width will be half of the `sm` width

    > **Note:** during image processing a file's breakpoints will be made available as frontmatter variables

## Other Image Options

The most important aspect of **smart image** is making sure have the appropriate formats and sizes but the following additional features are desirable and should be considered when deciding which Rust crates we will use for image processing.

1. Metadata Clearing

    - it would be highly desirable to ensure that all metadata contained in the original source image was stripped out of the optimized images we create
    - this would not necessitate any fine grain understanding of the metadata but just an assurance that it could be removed

2. Metadata Reading

    - it would be nice if we could read in metadata from a source image
    - if that were allowed we could add some or all of the attributes to `data-xxx` tags on the eventual HTML reference to the image
    - we could also use some metadata fields to automatically fill in the `alt` text field on the HTML if the user hadn't explicitly stated it

3. Metadata Writing

    - allowing the user to inject a common key/value into metadata could be useful
    - this is less important than the clearing and reading of metadata

4. Pre-blur Image

   - if our library choices allow us to blur an image then it would be highly desirable to generate a scaled down 32x32 pixel image of the source image and then blur it so that what is left is a tiny image file that can be preloaded as a first step before we do a short animated transition to the full image once it's finished loading.
   - if we are able to do this then this file should be generated as `{{hash}}-blur.jpg`

## Optimization Flow

Based on the context from above, when a file is being processed, the smart image references will be parsed out and the `smart_images()` function will be called with a vector of entries containing:

- **input source** (either a local image file _or_ an external image URL)
- **breakpoints** for the file being processed

It will then need to iterate over these images sources -- using [rayon](../.claude/skills/rayon/SKILL.md) to spread the computational load across available CPU cores -- and then:

1. Produce an [xxHash](./hashing.md) from the file or URL descriptor
2. Use the hash to lookup in the [image cache](#image-cache) whether a cache entry exists and is still valid
   - if **yes** then we will use that and no additional processing is necessary
   - if cache existed but was _stale_ then it will be removed from the image cache
3. In cases where image generation is not cached:

    - We must now generate all the image files defined in the [size optimization](#image-sizes) section (and the pre-blur image in [image options](#other-image-options) section if it's implemented)
    - These images will be placed into the file system based on the frontmatter's `output_dir` value (where "output" will be a default value for `output_dir`) and use an offset of `{{output_dir}}/images`
    - Because hashing should avoid image collisions the `{{output_dir}}/images` will be a flat set of optimized images.

## Image Cache

The image cache is stored in the library's [SurrealDB database](./database.md) and will be a simple KV based cache where the _keys_ are the hash of the file or URL descriptor and the value is a hash indicating it's content/recency.

