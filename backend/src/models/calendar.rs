use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ─── Database rows ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Calendar {
    pub id:          Uuid,
    pub owner_id:    Uuid,
    pub name:        String,
    pub description: Option<String>,
    pub color:       String,
    pub is_public:   bool,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct CalendarMember {
    pub calendar_id: Uuid,
    pub user_id:     Uuid,
    pub role:        String,
    pub joined_at:   DateTime<Utc>,
}

// ─── Request bodies ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCalendarRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be 1-255 characters"))]
    pub name: String,

    pub description: Option<String>,

    #[validate(length(max = 20, message = "Color must be at most 20 characters"))]
    pub color: Option<String>,

    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCalendarRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be 1-255 characters"))]
    pub name: Option<String>,

    pub description: Option<String>,

    #[validate(length(max = 20, message = "Color must be at most 20 characters"))]
    pub color: Option<String>,

    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddMemberRequest {
    pub user_id: Uuid,

    /// Allowed values: "editor" | "viewer"
    pub role: String,
}

impl AddMemberRequest {
    pub fn is_valid_role(&self) -> bool {
        matches!(self.role.as_str(), "editor" | "viewer")
    }
}
