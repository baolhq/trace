-- Nodes: one row per .md file. Body lives on disk, not here.
CREATE TABLE IF NOT EXISTS nodes
(
    id           TEXT PRIMARY KEY,
    path         TEXT    NOT NULL UNIQUE,
    title        TEXT    NOT NULL,
    created_at   INTEGER NOT NULL,
    modified_at  INTEGER NOT NULL,
    content_hash TEXT    NOT NULL,
    byte_size    INTEGER NOT NULL,
    is_favorite  INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_nodes_modified ON nodes (modified_at DESC);
CREATE INDEX IF NOT EXISTS idx_nodes_favorite ON nodes (is_favorite) WHERE is_favorite = 1;

-- Links: directed edges. target_id NULL = unresolved.
CREATE TABLE IF NOT EXISTS links
(
    source_id  TEXT    NOT NULL,
    target_id  TEXT,
    target_raw TEXT    NOT NULL,
    link_type  INTEGER NOT NULL,
    PRIMARY KEY (source_id, target_raw),
    FOREIGN KEY (source_id) REFERENCES nodes (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_links_target ON links (target_id);
CREATE INDEX IF NOT EXISTS idx_links_source ON links (source_id);

CREATE TABLE IF NOT EXISTS tags
(
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);
CREATE TABLE IF NOT EXISTS node_tags
(
    node_id TEXT    NOT NULL,
    tag_id  INTEGER NOT NULL,
    PRIMARY KEY (node_id, tag_id),
    FOREIGN KEY (node_id) REFERENCES nodes (id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_node_tags_tag ON node_tags (tag_id);

CREATE TABLE IF NOT EXISTS logs
(
    id        INTEGER PRIMARY KEY,
    name      TEXT NOT NULL,
    parent_id INTEGER,
    sort_key  REAL NOT NULL DEFAULT 0,
    FOREIGN KEY (parent_id) REFERENCES logs (id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS log_members
(
    log_id   INTEGER NOT NULL,
    node_id  TEXT    NOT NULL,
    sort_key REAL    NOT NULL DEFAULT 0,
    PRIMARY KEY (log_id, node_id),
    FOREIGN KEY (log_id) REFERENCES logs (id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES nodes (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_log_members_node ON log_members (node_id);

-- Recent nodes (capped, app-managed).
CREATE TABLE IF NOT EXISTS recent_nodes
(
    node_id   TEXT PRIMARY KEY,
    opened_at INTEGER NOT NULL,
    FOREIGN KEY (node_id) REFERENCES nodes (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_recent_opened ON recent_nodes (opened_at DESC);

-- Schema version + scan checkpoint.
CREATE TABLE IF NOT EXISTS app_meta
(
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
