# Adopting Shipcheck on an existing project

A first scan of an old codebase can list hundreds of findings. Three tools let a team
start anyway.

## Project settings

Put a `shipcheck.yml` (or `.shipcheck.yml`) in the folder you scan:

```yaml
ignore_rules: [LEGAL-002]        # switch rules off for the whole project
ignore_paths: [vendor/, dist]    # ignore findings under these paths
fail_under: 80                   # minimum score, when --fail-under is not given
```

Unknown keys are rejected, so a typo does not silently do nothing.

## Comments in the code

```js
eval(userInput); // shipcheck-ignore: SEC-001 input is checked above

// shipcheck-ignore: SEC-004, SEC-001
el.innerHTML = trustedHtml;
```

A comment covers its own line and the next one. `shipcheck-ignore-file: SEC-001`
anywhere in a file covers the whole file. Findings that belong to the whole project,
such as a missing privacy policy, cannot be silenced in code. Use `shipcheck.yml`.

## Baselines

shipcheck . --write-baseline .shipcheck-baseline.json
git add .shipcheck-baseline.json
shipcheck . --baseline .shipcheck-baseline.json --fail-under 80


The baseline accepts today's findings, so later scans only report new ones and the score
is computed from those. A finding is remembered by its rule, its file and the text of its
line, not its line number, so adding code above it does not make it look new. Two
identical lines count as two findings. Fix an old finding and it simply stops appearing.
Write a fresh baseline now and then to drop entries that no longer exist.

## Order

Findings go through the project settings first, then the comments, then the baseline.
The text report ends with how many were set aside by each, and the JSON report has
`ignored`, `suppressed` and `baselined` counts next to `score` and `findings`.
