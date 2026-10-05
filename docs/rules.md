# Writing rules

Rules live in `rules/<category>/*.yml`. Each file is a YAML list of rules.

```yaml
- id: SEC-004
  category: security        # legal | security | design
  severity: high            # info | low | medium | high | critical
  message: What is wrong.
  fix: How to fix it.
  extensions: [js, ts]      # optional, empty means every text file
  match:
    kind: line_regex
    pattern: '\.innerHTML\s*='
  examples:
    flag: ['el.innerHTML = x;']
    pass: ['el.textContent = x;']
```

## Matcher kinds

- `line_regex`: flags every line matching `pattern`. Add `unless` to skip lines that also match it.
- `missing_file`: flags the project when none of the `any_of` file names exist.
- `requires_pattern`: flags the project when `when` matches somewhere but `expect` matches in no file.
  The finding points at the first place `when` matched.

## Examples

`flag` lines must be flagged and `pass` lines must not be. For `requires_pattern`
the examples test the `when` pattern. `cargo test` checks every example of every
shipped rule, so a rule that disagrees with its own examples fails the build.
