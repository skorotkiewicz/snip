-- Derive fork counts from rows, including when forks are deleted.
ALTER TABLE snippets DROP COLUMN forks;

CREATE TRIGGER detach_deleted_snippet_forks
AFTER DELETE ON snippets
BEGIN
    UPDATE snippets SET forked_from = NULL WHERE forked_from = OLD.id;
END;

CREATE INDEX idx_snippets_user_created ON snippets(user_id, created_at DESC, id DESC);

-- Fixed-window limits apply without an external service and survive restarts.
CREATE TABLE rate_limits (
    scope TEXT NOT NULL,
    identifier TEXT NOT NULL,
    requests INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    PRIMARY KEY (scope, identifier)
);
CREATE INDEX idx_rate_limits_expiry ON rate_limits(expires_at);
