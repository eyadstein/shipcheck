# Taint analysis

Shipcheck parses JavaScript, TypeScript and Python with tree-sitter and tracks
user input from sources to sinks.

- Sources: request data such as `req.query`, `request.args`, `location.hash`, `input()`.
- Sanitizers: calls such as `parseInt`, `path.basename`, `shlex.quote`, `escape`.
- Sinks: SQL, shell, code evaluation, HTML, outgoing URLs, file paths,
  unsafe deserializers, redirects and template engines.

Findings use rule ids SEC-101 to SEC-109. Sources, sanitizers and sinks are plain
data in `crates/shipcheck-taint/src/catalog.rs`.

## Limits

- Flow is tracked within one file, in source order.
- Taint created inside a function does not leave that function, and function
  parameters are not treated as tainted.
- Data is not followed across files or through calls to your own functions.
- Branches and loops are not modeled, so treat results as leads to review.
