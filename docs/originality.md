# Originality index

A number from 0 to 100 for how much of a design is the project's own. Zero means
everything is a framework default, 100 means nothing is.

cargo run -p shipcheck-cli -- . --originality
cargo run -p shipcheck-cli -- . --originality --format json


## What is measured

| Part | Weight | Counts as a default |
|---|---|---|
| Palette | 50 | A stock Tailwind color class such as `bg-blue-500`, or a hex or `rgb()` color within a few points of a well-known default from Tailwind, Bootstrap or Material |
| Typography | 30 | A typeface that a very large share of projects pick, such as Inter, Roboto or Poppins |
| Framework tokens | 20 | The shadcn/ui theme variables left at their defaults |

The weights are shared out between the parts that were found. A part with nothing to
judge is left out. A palette needs at least three distinct colors, and pure black and
white are ignored. Fonts come from `font-family` declarations, font links, `next/font`
imports, Tailwind config and `--font-*` custom properties.

## Making a design your own

Rename the colors. `bg-blue-500` is the framework's color, `bg-brand-500` is yours, and
the index sees the difference. The same goes for choosing a typeface that is not on the
popular list and changing the theme tokens that shadcn/ui ships with.

## Finding

`DES-301` (low) is reported when the palette was judged and the index is below 40.

## Limits

It is a measure of defaults, not of taste. A design can be built entirely from defaults and
look good, and a custom palette can look bad. Colors that live in images, in a theme
object the scanner cannot read, or in CSS variables that hold bare numbers (such as
`--primary: 222 47% 11%`) are not seen, apart from the shadcn/ui token check.
