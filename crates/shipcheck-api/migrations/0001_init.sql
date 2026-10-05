CREATE TABLE scans (
    id         UUID PRIMARY KEY,
    project    TEXT NOT NULL,
    git_ref    TEXT,
    score      INTEGER NOT NULL CHECK (score BETWEEN 0 AND 100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX scans_project_created_idx ON scans (project, created_at DESC);

CREATE TABLE findings (
    id       BIGSERIAL PRIMARY KEY,
    scan_id  UUID NOT NULL REFERENCES scans (id) ON DELETE CASCADE,
    rule_id  TEXT NOT NULL,
    category TEXT NOT NULL CHECK (category IN ('legal', 'security', 'design')),
    severity TEXT NOT NULL CHECK (severity IN ('info', 'low', 'medium', 'high', 'critical')),
    message  TEXT NOT NULL,
    file     TEXT NOT NULL,
    line     INTEGER NOT NULL CHECK (line >= 0),
    fix      TEXT NOT NULL DEFAULT ''
);

CREATE INDEX findings_scan_idx ON findings (scan_id);
