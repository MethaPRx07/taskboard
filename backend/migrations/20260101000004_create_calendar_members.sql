-- Migration 004: Create calendar_members table
-- Many-to-many between calendars and users with role-based access

CREATE TABLE calendar_members (
    calendar_id UUID        NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    user_id     UUID        NOT NULL REFERENCES users(id)     ON DELETE CASCADE,
    role        VARCHAR(50) NOT NULL DEFAULT 'viewer',   -- owner | editor | viewer
    joined_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (calendar_id, user_id),
    CONSTRAINT valid_member_role CHECK (role IN ('owner', 'editor', 'viewer'))
);

CREATE INDEX idx_calendar_members_user_id ON calendar_members(user_id);
