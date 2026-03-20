use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ─── Database rows ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Task {
    pub id:          Uuid,
    pub calendar_id: Uuid,
    pub creator_id:  Uuid,
    pub title:       String,
    pub description: Option<String>,
    pub status:      String,
    pub priority:    String,
    pub due_date:    Option<DateTime<Utc>>,
    pub start_date:  Option<DateTime<Utc>>,
    pub all_day:     bool,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct TaskAssignee {
    pub task_id:     Uuid,
    pub user_id:     Uuid,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct TaskLabel {
    pub task_id: Uuid,
    pub label:   String,
    pub color:   Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct TaskComment {
    pub id:         Uuid,
    pub task_id:    Uuid,
    pub user_id:    Uuid,
    pub content:    String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Request bodies ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTaskRequest {
    #[validate(length(min = 1, max = 500, message = "Title must be 1-500 characters"))]
    pub title: String,

    pub description: Option<String>,

    /// todo | in_progress | done | cancelled
    pub status: Option<String>,

    /// low | medium | high | urgent
    pub priority: Option<String>,

    pub due_date:   Option<DateTime<Utc>>,
    pub start_date: Option<DateTime<Utc>>,
    pub all_day:    Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTaskRequest {
    #[validate(length(min = 1, max = 500, message = "Title must be 1-500 characters"))]
    pub title: Option<String>,

    pub description: Option<String>,
    pub priority:    Option<String>,
    pub due_date:    Option<DateTime<Utc>>,
    pub start_date:  Option<DateTime<Utc>>,
    pub all_day:     Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskStatusRequest {
    /// todo | in_progress | done | cancelled
    pub status: String,
}

impl UpdateTaskStatusRequest {
    pub fn is_valid(&self) -> bool {
        matches!(
            self.status.as_str(),
            "todo" | "in_progress" | "done" | "cancelled"
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct AssignUserRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddLabelRequest {
    #[validate(length(min = 1, max = 100, message = "Label must be 1-100 characters"))]
    pub label: String,

    #[validate(length(max = 20, message = "Color must be at most 20 characters"))]
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddCommentRequest {
    #[validate(length(min = 1, message = "Comment cannot be empty"))]
    pub content: String,
}

// ─── Query params for listing tasks ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TaskQuery {
    pub date_from: Option<DateTime<Utc>>,
    pub date_to:   Option<DateTime<Utc>>,
    pub status:    Option<String>,
    pub priority:  Option<String>,
    pub assignee:  Option<Uuid>,
}
