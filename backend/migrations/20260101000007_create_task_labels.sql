-- Migration 007: Create task_labels table

CREATE TABLE task_labels (
    task_id UUID        NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    label   VARCHAR(100) NOT NULL,
    color   VARCHAR(20)  NOT NULL DEFAULT '#808080',

    PRIMARY KEY (task_id, label)
);
