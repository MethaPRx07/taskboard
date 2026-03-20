-- Migration 005: Create tasks table

CREATE TABLE tasks (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    calendar_id UUID         NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    creator_id  UUID         NOT NULL REFERENCES users(id),
    title       VARCHAR(500) NOT NULL,
    description TEXT,
    status      VARCHAR(50)  NOT NULL DEFAULT 'todo',
    priority    VARCHAR(50)  NOT NULL DEFAULT 'medium',
    due_date    TIMESTAMPTZ,
    start_date  TIMESTAMPTZ,
    all_day     BOOLEAN      NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_task_status   CHECK (status   IN ('todo', 'in_progress', 'done', 'cancelled')),
    CONSTRAINT valid_task_priority CHECK (priority IN ('low', 'medium', 'high', 'urgent'))
);

CREATE INDEX idx_tasks_calendar_id ON tasks(calendar_id);
CREATE INDEX idx_tasks_creator_id  ON tasks(creator_id);
CREATE INDEX idx_tasks_status      ON tasks(status);
CREATE INDEX idx_tasks_due_date    ON tasks(due_date);
CREATE INDEX idx_tasks_start_date  ON tasks(start_date);

CREATE TRIGGER trg_tasks_updated_at
    BEFORE UPDATE ON tasks
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
