# GitHub Action

The action scans a directory, posts the result as a pull request comment, writes the
full report to the job summary, and fails the job when the score is too low.

```yaml
name: Shipcheck
on:
  pull_request:

permissions:
  contents: read
  pull-requests: write

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: eyadstein/shipcheck/action@main
        with:
          fail-under: "80"
```

## Inputs

| Input | Default | Meaning |
|---|---|---|
| `path` | `.` | Directory to scan, relative to the workspace |
| `fail-under` | `0` | Fail the job below this score. `0` never fails |
| `rules` | built in | Directory with your own rule files |
| `comment` | `true` | Post the pull request comment |
| `github-token` | `github.token` | Token for the comment, needs `pull-requests: write` |
| `api-url` | empty | Shipcheck API to upload the report to |
| `api-key` | empty | Key for that API, keep it in a repository secret |
| `project` | repository | Project name used for the upload |

Outputs: `score` and `findings`.

## How it works

1. Builds the scanner from the action's repository with `cargo build --release`.
2. Scans `path` and saves the JSON report.
3. `src/run.ts` reads the report, writes the job summary, posts or updates one comment
   (found through a hidden marker), optionally uploads to the API, and sets the exit code.

The script runs on Node 24 directly, using its built-in TypeScript support, so there is
no build step and no runtime dependency.

## Notes

- Linux runners only for now. The first run compiles the scanner, which takes a few minutes.
- Pull requests from forks get a read-only token. The comment is skipped with a warning
  and the report still appears in the job summary.
- File names and rule text are escaped before they go into the comment, so a pull request
  cannot inject markdown, links or mentions through them.
- The upload failing never fails the job. Only the score threshold does.

This repository's first pull request was opened to try the action.
