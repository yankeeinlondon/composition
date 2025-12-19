# Breakpoints

In several areas of the system the concept of "breakpoints" is leveraged. This idea and in particular the naming convention used borrows from the **Tailwind CSS** library:

- `sm` - 640px (40rem)
- `md` - 768px (48rem)
- `lg` - 1024px (64rem)
- `xl` - 1280px (80rem)
- `2xl` - 1536px (96rem)

## Image Widths

When using [smart images](../design/smart-image.md), the image sizes will provide images at no larger than twice the breakpoint size but they also provide a `xs` and `micro` sized image which is:

- `xs` is equal to the `sm` breakpoint width
- `micro` is equal to half of the `sm` breakpoint width
