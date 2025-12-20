# Breakpoints

In several areas of the system the concept of "breakpoints" is leveraged. This idea and in particular the naming convention used borrows from the **Tailwind CSS** library.

## Breakpoint Definitions

The system supports 7 responsive breakpoints:

- `micro` - 320px (20rem) - Mobile portrait
- `xs` - 640px (40rem) - Mobile landscape
- `sm` - 640px (40rem) - Small devices
- `md` - 768px (48rem) - Medium devices (tablets)
- `lg` - 1024px (64rem) - Large devices (laptops)
- `xl` - 1280px (80rem) - Extra large devices (desktops)
- `xxl` - 1536px (96rem) - 2X extra large devices (large desktops)

Note: `xs` and `sm` have the same pixel width (640px) but serve different semantic purposes. `xs` represents the minimum viable mobile landscape width, while `sm` aligns with Tailwind CSS conventions.

## Image Processing and Retina Support

When using [smart images](../design/smart-image.md), the image processing system generates variants for both standard (1x) and retina (2x) displays:

### Standard (1x) Variants
- `micro` - 320px
- `xs` - 640px
- `sm` - 640px
- `md` - 768px
- `lg` - 1024px
- `xl` - 1280px
- `xxl` - 1536px

### Retina (2x) Variants
For HiDPI/retina displays, variants are generated at 2x the standard width:

- `micro@2x` - 640px
- `xs@2x` - 1280px
- `sm@2x` - 1280px
- `md@2x` - 1536px
- `lg@2x` - 2048px
- `xl@2x` - 2560px
- `xxl@2x` - 3072px

### Browser Selection

The generated HTML includes all variants in the `srcset` attribute with width descriptors. Browsers automatically select the most appropriate variant based on:

1. Device pixel ratio (DPR)
2. Viewport width
3. Network conditions (in some browsers)

Example generated `srcset`:

```html
<source
  type="image/avif"
  srcset="data:image/avif;base64,... 320w,
          data:image/avif;base64,... 640w,
          data:image/avif;base64,... 1280w,
          data:image/avif;base64,... 1536w,
          ..."
/>
```

The width descriptor (e.g., `320w`, `640w`) tells the browser the actual width of each image, allowing it to make optimal selection decisions.
