#!/bin/bash
# Populate tickit with demo data

cd ~/dev/ricardodantas/tickit

# Create config directory if needed
mkdir -p ~/.config/tickit

# Remove old database
rm -f ~/.config/tickit/tickit.sqlite

# Build if needed
cargo build --release 2>/dev/null

# The database will be created on first run
# We'll use sqlite3 directly to insert demo data after first run initializes the schema

# Run tickit briefly to initialize DB
timeout 2 cargo run --release 2>/dev/null || true

# Now populate with demo data
sqlite3 ~/.config/tickit/tickit.sqlite << 'EOF'
-- Insert Lists
INSERT INTO lists (id, name, icon, is_inbox, position, created_at, updated_at) VALUES
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

-- Get inbox ID
-- INSERT INTO tasks using the actual inbox ID

-- Insert Tasks (using inbox for now, will be assigned to lists)
INSERT INTO tasks (id, title, description, url, priority, completed, list_id, position, created_at, updated_at) VALUES
('bbbb1111-1111-1111-1111-111111111111', 'Review pull request for auth module', 'Check OAuth implementation and security headers', 'https://github.com/example/repo/pull/123', 3, 0, '11111111-1111-1111-1111-111111111111', 0, datetime('now'), datetime('now')),
('bbbb2222-2222-2222-2222-222222222222', 'Fix production memory leak', 'Memory leak in authentication service causing OOM', NULL, 4, 0, '11111111-1111-1111-1111-111111111111', 1, datetime('now'), datetime('now')),
('bbbb3333-3333-3333-3333-333333333333', 'Write API documentation', 'Document REST endpoints for v2 API', 'https://docs.example.com', 2, 0, '11111111-1111-1111-1111-111111111111', 2, datetime('now'), datetime('now')),
('bbbb4444-4444-4444-4444-444444444444', 'Deploy to staging', 'Push latest changes to staging environment', NULL, 2, 1, '11111111-1111-1111-1111-111111111111', 3, datetime('now'), datetime('now')),
('bbbb5555-5555-5555-5555-555555555555', 'Buy groceries', 'Milk, eggs, bread, cheese, vegetables', NULL, 1, 0, '33333333-3333-3333-3333-333333333333', 0, datetime('now'), datetime('now')),
('bbbb6666-6666-6666-6666-666666666666', 'Schedule dentist appointment', 'Annual checkup - call Dr. Smith', NULL, 1, 0, '22222222-2222-2222-2222-222222222222', 0, datetime('now'), datetime('now')),
('bbbb7777-7777-7777-7777-777777777777', 'Plan weekend trip', 'Research hotels and activities', 'https://booking.com', 0, 0, '22222222-2222-2222-2222-222222222222', 1, datetime('now'), datetime('now')),
('bbbb8888-8888-8888-8888-888888888888', 'Update resume', 'Add recent projects and skills', NULL, 2, 1, '22222222-2222-2222-2222-222222222222', 2, datetime('now'), datetime('now'));

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
