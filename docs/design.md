# Design analysis

`shipcheck-design` looks for the patterns that make an app look machine-generated.
It reads stylesheets and page files (css, scss, html, jsx, tsx, vue, svelte, astro,
mdx, js, ts), skips `node_modules`, `dist`, `build`, `vendor` and minified files,
and reports at most one finding per rule per file, or per project for project-wide rules.

| Rule | What it looks for |
|---|---|
| DES-201 | Gradient text (`bg-clip-text` plus transparent text) |
| DES-202 | Glassmorphism (blur plus translucent background) |
| DES-203 | Thick colored accent border on cards |
| DES-204 | Inter is the only typeface in the project |
| DES-205 | Dark theme text below 4.5:1 contrast |
| DES-206 | Three icon feature boxes in a row |
| DES-207 | Pill badge above the main headline |
| DES-208 | Lucide icons imported in six or more files |
| DES-209 | Untouched shadcn/ui theme variables |
| DES-210 | Fade-in on scroll libraries and classes |
| DES-211 | Cursor-following glow or beam |
| DES-212 | Buttons that only change opacity on hover |
| DES-213 | Off-grid spacing values |
| DES-214 | Eight or more em dashes in page copy |
| DES-215 | Five or more generic buzzwords |
| DES-216 | Serif italic accent words |
| DES-217 | Space Grotesk with Instrument Serif |
| DES-218 | Grain or noise overlay on a gradient |

The simpler line rules DES-001 to DES-003 live in `rules/design/design.yml`.

## Limits

These are heuristics. Several look for patterns within a few lines of each other,
and thresholds such as "six files" or "eight em dashes" are tuned to avoid noise,
not proven. Treat findings as prompts to review, not verdicts.
