-- Create policies table for Hodei authorization framework
CREATE TABLE IF NOT EXISTS policies (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP
);

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS idx_policies_created_at ON policies(created_at);

-- Comments
COMMENT ON TABLE policies IS 'Cedar policies storage for Hodei authorization framework';
COMMENT ON COLUMN policies.id IS 'UUID policy identifier';
COMMENT ON COLUMN policies.content IS 'Cedar policy content in text format';
