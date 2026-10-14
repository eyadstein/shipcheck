# Rule gym

Every rule has `flag` examples (lines it must catch) and `pass` examples (lines it must
leave alone). The gym rewrites each example in ways that keep its meaning, then checks
that the rule still behaves.

cargo run -p shipcheck-cli -- . --gym
cargo run -p shipcheck-cli -- . --gym --fail-under 90


## The attacks

| Attack | What it does | Applies to |
|---|---|---|
| `double_spaces` | Every space becomes two | Lines with a space |
| `tabs` | Every space becomes a tab | Lines with a space |
| `space_before_paren` | `eval(x)` becomes `eval (x)` | Programming languages only |
| `swap_quotes` | Double and single quotes trade places | Lines with a quote |
| `upper_case` | The line is upper-cased | HTML and CSS only |
| `break_after_paren` | A line break after the first `(` | Lines with a `(` |
| `break_after_equals` | A line break after the first `=` | Lines with a `=` |
| `indent` | Eight spaces in front | Every line |
| `trailing_comment` | `// ok` after the line | Every line |

A variant counts as flagged when any of its lines is, because the scanner reads one line
at a time. Attacks on `flag` examples that are not caught are reported as `missed`. Attacks on
`pass` examples that are caught are reported as `false alarm`.

## Grades

The robustness of a rule is the share of attacks it handled correctly. A is 90 and above,
B is 75, C is 50, D is below. The report lists the weakest rules first, with every attack
that got through.

## What it cannot fix

A rule that reads one line at a time cannot see a construct that is split across lines.
That is why `break_after_paren` and `break_after_equals` remain the usual slips, and why
a multi-line `<img>` tag can look like a missing `alt` attribute. The gym reports these
honestly instead of hiding them.

CI runs the gym with `--fail-under 90`, so a new rule cannot make the rule set easier
to fool without anyone noticing.
