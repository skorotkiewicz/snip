-- Add views counter to snippets
ALTER TABLE snippets ADD COLUMN views INTEGER DEFAULT 0;

-- Create stars table for tracking snippet stars
CREATE TABLE IF NOT EXISTS stars (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    snippet_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (snippet_id) REFERENCES snippets(id) ON DELETE CASCADE,
    UNIQUE(user_id, snippet_id)
);

-- Index for fast star count lookups
CREATE INDEX IF NOT EXISTS idx_stars_snippet_id ON stars(snippet_id);
CREATE INDEX IF NOT EXISTS idx_stars_user_id ON stars(user_id);
