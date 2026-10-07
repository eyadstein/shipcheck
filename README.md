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
| `crates/shipcheck-cli` | The `shipcheck` command |
| `crates/shipcheck-api` | HTTP API with PostgreSQL storage |
| `rules/` | YAML rule packs |
| `web/` | React and TypeScript dashboard |

## Quick start

cargo run -p shipcheck-cli -- examples/demo-app

Run the API and the dashboard:
docker compose up -d --wait
cargo run -p shipcheck-api # needs the variables from .env.example
cd web && npm install && npm run dev


More detail lives in `docs/`: rules, taint analysis, design analysis and the API.
