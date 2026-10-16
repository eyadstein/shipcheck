# Shipcheck

Scan a codebase for legal, security and design problems before you ship it.

Shipcheck checks three lists that most projects get wrong:

- **Legal and compliance**: missing policies, consent, unsubscribe links, accessibility basics.
- **Security**: injection, SSRF, path traversal and similar, found by tracing user input through the syntax tree.
- **Design**: the patterns that make an app look machine-generated.

Every finding has a rule id, a location and a suggested fix. The project gets a score from 0 to 100.

## Layout

| Path | What it is |
|---|---|
| `crates/shipcheck-core` | Shared types and scoring |
| `crates/shipcheck-engine` | Rule loading and the scanner |
| `crates/shipcheck-taint` | Taint analysis for JavaScript, TypeScript and Python |
| `crates/shipcheck-design` | Project level design analysis |
| `crates/shipcheck-ledger` | Compares what the code does with what the privacy policy says |
| `crates/shipcheck-cli` | The `shipcheck` command |
| `crates/shipcheck-api` | HTTP API with PostgreSQL storage |
| `rules/` | YAML rule packs |
| `web/` | React and TypeScript dashboard |
| `action/` | GitHub Action that comments on pull requests |

## Quick start

cargo run -p shipcheck-cli -- examples/demo-app

Run the API and the dashboard:
docker compose up -d --wait
cargo run -p shipcheck-api # needs the variables from .env.example
cd web && npm install && npm run dev


More detail lives in `docs/`: rules, taint analysis, design analysis and the API.

## Originality index

Shipcheck measures how much of a project's design is customized rather than left at common framework defaults. The originality index ranges from 0 to 100.

Run the index against your project:

```bash
cargo run -p shipcheck-cli -- . --originality
```

Output the result as JSON:

```bash
cargo run -p shipcheck-cli -- . --originality --format json
```

### What is measured?

| Component | Weight | What it checks |
|---|---:|---|
| Color palette | 50 | Common Tailwind, Bootstrap, and Material default colors |
| Typography | 30 | Frequently used typefaces such as Inter, Roboto, and Poppins |
| Framework tokens | 20 | Default shadcn/ui theme variables |

The weights are distributed across the components that can be evaluated. Components with insufficient data are excluded from the calculation.

### Improving originality

- Customize framework-default colors and build a consistent brand palette.
- Choose typography that fits your product rather than relying on common defaults.
- Customize shadcn/ui theme tokens instead of leaving every value unchanged.

### Originality finding

`DES-301` is a low-severity finding reported when the palette can be evaluated and the originality index is below 40.

### Limitations

The index measures customization, not visual quality. A design using framework defaults can still look excellent. Some colors stored in images, unreadable theme objects, or certain CSS variable formats may not be detected.

See [docs/originality.md](docs/originality.md) for the full methodology.
