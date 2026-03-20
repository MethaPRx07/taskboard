use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::calendar::{Calendar, CalendarMember};

// ─── Calendar CRUD ────────────────────────────────────────────────────────────

pub async fn create_calendar(
    pool: &PgPool,
    owner_id: Uuid,
    name: &str,
    description: Option<&str>,
    color: &str,
    is_public: bool,
) -> Result<Calendar, AppError> {
    let calendar = sqlx::query_as::<_, Calendar>(
        r#"INSERT INTO calendars (owner_id, name, description, color, is_public)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, owner_id, name, description, color, is_public, created_at, updated_at"#,
    )
    .bind(owner_id)
    .bind(name)
    .bind(description)
    .bind(color)
    .bind(is_public)
    .fetch_one(pool)
    .await?;

    // Also add the creator as owner member
    sqlx::query(
        r#"INSERT INTO calendar_members (calendar_id, user_id, role)
           VALUES ($1, $2, 'owner')
           ON CONFLICT DO NOTHING"#,
    )
    .bind(calendar.id)
    .bind(owner_id)
    .execute(pool)
    .await?;

    Ok(calendar)
}

pub async fn find_calendar_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Calendar>, AppError> {
    let calendar = sqlx::query_as::<_, Calendar>(
        r#"SELECT id, owner_id, name, description, color, is_public, created_at, updated_at
           FROM calendars
           WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(calendar)
}

pub async fn list_calendars_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Calendar>, AppError> {
    // Returns calendars the user owns OR is a member of
    let calendars = sqlx::query_as::<_, Calendar>(
        r#"SELECT DISTINCT c.id, c.owner_id, c.name, c.description, c.color, c.is_public,
                  c.created_at, c.updated_at
           FROM calendars c
           LEFT JOIN calendar_members cm ON cm.calendar_id = c.id
           WHERE c.owner_id = $1
              OR cm.user_id  = $1
           ORDER BY c.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(calendars)
}

pub async fn update_calendar(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    color: Option<&str>,
    is_public: Option<bool>,
) -> Result<Option<Calendar>, AppError> {
    let calendar = sqlx::query_as::<_, Calendar>(
        r#"UPDATE calendars
           SET name        = COALESCE($2, name),
               description = COALESCE($3, description),
               color       = COALESCE($4, color),
               is_public   = COALESCE($5, is_public)
           WHERE id = $1
           RETURNING id, owner_id, name, description, color, is_public, created_at, updated_at"#,
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(color)
    .bind(is_public)
    .fetch_optional(pool)
    .await?;

    Ok(calendar)
}

pub async fn delete_calendar(pool: &PgPool, id: Uuid) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM calendars WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

// ─── Calendar members ─────────────────────────────────────────────────────────

pub async fn get_member(
    pool: &PgPool,
    calendar_id: Uuid,
    user_id: Uuid,
) -> Result<Option<CalendarMember>, AppError> {
    let member = sqlx::query_as::<_, CalendarMember>(
        r#"SELECT calendar_id, user_id, role, joined_at
           FROM calendar_members
           WHERE calendar_id = $1 AND user_id = $2"#,
    )
    .bind(calendar_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(member)
}

pub async fn add_member(
    pool: &PgPool,
    calendar_id: Uuid,
    user_id: Uuid,
    role: &str,
) -> Result<CalendarMember, AppError> {
    let member = sqlx::query_as::<_, CalendarMember>(
        r#"INSERT INTO calendar_members (calendar_id, user_id, role)
           VALUES ($1, $2, $3)
           ON CONFLICT (calendar_id, user_id)
           DO UPDATE SET role = EXCLUDED.role
           RETURNING calendar_id, user_id, role, joined_at"#,
    )
    .bind(calendar_id)
    .bind(user_id)
    .bind(role)
    .fetch_one(pool)
    .await?;

    Ok(member)
}

pub async fn remove_member(
    pool: &PgPool,
    calendar_id: Uuid,
    user_id: Uuid,
) -> Result<bool, AppError> {
    let result =
        sqlx::query("DELETE FROM calendar_members WHERE calendar_id = $1 AND user_id = $2")
            .bind(calendar_id)
            .bind(user_id)
            .execute(pool)
            .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_members(
    pool: &PgPool,
    calendar_id: Uuid,
) -> Result<Vec<CalendarMember>, AppError> {
    let members = sqlx::query_as::<_, CalendarMember>(
        r#"SELECT calendar_id, user_id, role, joined_at
           FROM calendar_members
           WHERE calendar_id = $1
           ORDER BY joined_at ASC"#,
    )
    .bind(calendar_id)
    .fetch_all(pool)
    .await?;

    Ok(members)
}
