# dmi2svg

This library uses the [`contour_tracing`](https://github.com/STPR/contour_tracing)
library to turn a [`DMI`](https://github.com/spacestation13/dmi-rust) file into a set of SVGs, one for each icon state.

The purpose of this library is primarily for integration into [`rust_g`](https://github.com/tgstation/rust-g)
as a way to create SVG assets for [Space Station 13](spacestation13.com) clients, however it can be used
with two standalone binaries in [src/bin](src/bin).

[dmi2css](src/bin/dmi2css.rs) allows you to invoke the library and write the output into a CSS file directly as embedded
data-uris, in this format:

```css
.spritesheet_name.state_name{background-image: url("data:image/svg+xml;base64,...")}
```

## Parallelism

This library uses [`rayon`](https://github.com/rayon-rs/rayon) to process all icon states in parallel.

Additionally, each *color* within an icon state is processed in parallel, which is a huge boost to performance.

To explain, the `contour_tracing` library requires a greyscale image in order to do it's tracing,
specifically a `Vec<Vec<i8>>` for each pixel.

This means that to support multi-color sprites, we have to run the algorithm for each unique color present in an
icon state and coalesce them into different [`<path>`](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/path)
elements, with the [`fill`](https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/fill) attribute set to the
original color.

However, this limitation actually ends up working in our benefit; because we have full control over the processing,
we can use `rayon` again to process every path in parallel and just collect them into one string at the end. This is
by far the best increase to performance, as it means that we can 'render' multiple parts of the state at once.