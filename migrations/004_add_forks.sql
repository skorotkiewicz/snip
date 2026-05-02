-- Add fork count and forked_from reference to snippets
ALTER TABLE snippets ADD COLUMN forks INTEGER DEFAULT 0;
ALTER TABLE snippets ADD COLUMN forked_from INTEGER;

-- Optional: Create index for forked_from to find all forks of a snippet
CREATE INDEX IF NOT EXISTS idx_snippets_forked_from ON snippets(forked_from) WHERE forked_from IS NOT NULL;
