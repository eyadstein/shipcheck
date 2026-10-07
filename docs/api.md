# API

Start PostgreSQL with `docker compose up -d --wait`, copy `.env.example` values into
your environment, then run `cargo run -p shipcheck-api`.

| Method | Path | Auth | Purpose |
|---|---|---|---|
| GET | `/health` | none | Checks that the database answers |
| POST | `/api/v1/scans` | `x-api-key` | Stores a report |
| GET | `/api/v1/scans?project=&limit=` | none | Lists scans, newest first (limit 1 to 200) |
| GET | `/api/v1/scans/{id}` | none | One scan with its findings |
| GET | `/api/v1/projects` | none | Latest score for every project |

## Storing a report

```json
{ "project": "owner/repo", "git_ref": "main", "findings": [ ... ] }
```

`findings` uses the same objects as `shipcheck --format json`. A `score` sent by the
client is ignored: the server computes it from the findings.

## Safety notes

- Writes need the key in the `x-api-key` header, checked before the body is read.
- The key must be at least 16 characters, and the server refuses to start without one.
- Requests over 8 MB and reports over 10000 findings are rejected.
- Database errors are logged and never returned to clients.
- Reads are open for now. Put the API behind a login before exposing it publicly.

## Browser access (CORS)

Set `CORS_ORIGINS` to a comma separated list of origins, for example
`https://dashboard.example,http://localhost:5173`. Those origins may send `GET`
requests. Writes are never allowed from a browser origin, so the API key stays on
servers and CI. Entries must be plain origins: no paths, no wildcards.
