#!/bin/bash
# Populate tickit with demo data

cd ~/dev/ricardodantas/tickit

# Create config directory if needed
mkdir -p ~/.config/tickit

# Remove old database
rm -f ~/.config/tickit/tickit.sqlite

# Create the database with schema and demo data
sqlite3 ~/.config/tickit/tickit.sqlite << 'EOF'
-- Lists table
CREATE TABLE IF NOT EXISTS lists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT NOT NULL DEFAULT '📋',
    color TEXT,
    is_inbox INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Tags table
CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    url TEXT,
    priority TEXT NOT NULL DEFAULT 'medium',
    completed INTEGER NOT NULL DEFAULT 0,
    list_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,
    due_date TEXT,
    FOREIGN KEY (list_id) REFERENCES lists(id) ON DELETE CASCADE
);

-- Task-Tag junction table
CREATE TABLE IF NOT EXISTS task_tags (
    task_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (task_id, tag_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_tasks_list ON tasks(list_id);
CREATE INDEX IF NOT EXISTS idx_tasks_completed ON tasks(completed);
CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority);

-- Insert Inbox (required)
INSERT INTO lists (id, name, icon, is_inbox, sort_order, created_at, updated_at) VALUES
('00000000-0000-0000-0000-000000000000', 'Inbox', '📥', 1, 0, datetime('now'), datetime('now'));

-- Insert Lists
INSERT INTO lists (id, name, icon, is_inbox, sort_order, created_at, updated_at) VALUES
('11111111-1111-1111-1111-111111111111', 'Work', '💼', 0, 1, datetime('now'), datetime('now')),
('22222222-2222-2222-2222-222222222222', 'Personal', '🏠', 0, 2, datetime('now'), datetime('now')),
('33333333-3333-3333-3333-333333333333', 'Shopping', '🛒', 0, 3, datetime('now'), datetime('now'));

-- Insert Tags
INSERT INTO tags (id, name, color, created_at) VALUES
('aaaa1111-1111-1111-1111-111111111111', 'urgent', '#ff5555', datetime('now')),
('aaaa2222-2222-2222-2222-222222222222', 'bug', '#ff79c6', datetime('now')),
('aaaa3333-3333-3333-3333-333333333333', 'feature', '#50fa7b', datetime('now')),
('aaaa4444-4444-4444-4444-444444444444', 'docs', '#8be9fd', datetime('now')),
('aaaa5555-5555-5555-5555-555555555555', 'review', '#ffb86c', datetime('now'));

-- Insert Tasks (priority as text: none, low, medium, high, urgent)
INSERT INTO tasks (id, title, description, url, priority, completed, list_id, created_at, updated_at) VALUES
('bbbb1111-1111-1111-1111-111111111111', 'Review pull request for auth module', 'Check OAuth implementation and security headers', 'https://github.com/example/repo/pull/123', 'high', 0, '11111111-1111-1111-1111-111111111111', datetime('now'), datetime('now')),
('bbbb2222-2222-2222-2222-222222222222', 'Fix production memory leak', 'Memory leak in authentication service causing OOM', NULL, 'urgent', 0, '11111111-1111-1111-1111-111111111111', datetime('now'), datetime('now')),
('bbbb3333-3333-3333-3333-333333333333', 'Write API documentation', 'Document REST endpoints for v2 API', 'https://docs.example.com', 'medium', 0, '11111111-1111-1111-1111-111111111111', datetime('now'), datetime('now')),
('bbbb4444-4444-4444-4444-444444444444', 'Deploy to staging', 'Push latest changes to staging environment', NULL, 'medium', 1, '11111111-1111-1111-1111-111111111111', datetime('now'), datetime('now')),
('bbbb5555-5555-5555-5555-555555555555', 'Buy groceries', 'Milk, eggs, bread, cheese, vegetables', NULL, 'low', 0, '33333333-3333-3333-3333-333333333333', datetime('now'), datetime('now')),
('bbbb6666-6666-6666-6666-666666666666', 'Schedule dentist appointment', 'Annual checkup - call Dr. Smith', NULL, 'low', 0, '22222222-2222-2222-2222-222222222222', datetime('now'), datetime('now')),
('bbbb7777-7777-7777-7777-777777777777', 'Plan weekend trip', 'Research hotels and activities', 'https://booking.com', 'none', 0, '22222222-2222-2222-2222-222222222222', datetime('now'), datetime('now')),
('bbbb8888-8888-8888-8888-888888888888', 'Update resume', 'Add recent projects and skills', NULL, 'medium', 1, '22222222-2222-2222-2222-222222222222', datetime('now'), datetime('now')),
('bbbb9999-9999-9999-9999-999999999999', 'Reply to emails', 'Clear inbox backlog', NULL, 'low', 0, '00000000-0000-0000-0000-000000000000', datetime('now'), datetime('now'));

-- Insert Task-Tag relationships
INSERT INTO task_tags (task_id, tag_id) VALUES
('bbbb1111-1111-1111-1111-111111111111', 'aaaa5555-5555-5555-5555-555555555555'),
('bbbb2222-2222-2222-2222-222222222222', 'aaaa1111-1111-1111-1111-111111111111'),
('bbbb2222-2222-2222-2222-222222222222', 'aaaa2222-2222-2222-2222-222222222222'),
('bbbb3333-3333-3333-3333-333333333333', 'aaaa4444-4444-4444-4444-444444444444'),
('bbbb3333-3333-3333-3333-333333333333', 'aaaa3333-3333-3333-3333-333333333333');

EOF

echo "✅ Demo data populated!"
echo "Run 'cargo run --release' to see the demo"
