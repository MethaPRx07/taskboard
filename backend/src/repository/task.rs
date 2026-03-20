use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::task::{Task, TaskAssignee, TaskComment, TaskLabel, TaskQuery};

// ─── Task CRUD ────────────────────────────────────────────────────────────────

pub async fn create_task(
    pool: &PgPool,
    calendar_id: Uuid,
    creator_id: Uuid,
    title: &str,
    description: Option<&str>,
    status: &str,
    priority: &str,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    start_date: Option<chrono::DateTime<chrono::Utc>>,
    all_day: bool,
) -> Result<Task, AppError> {
    let task = sqlx::query_as::<_, Task>(
        r#"INSERT INTO tasks (calendar_id, creator_id, title, description, status, priority, due_date, start_date, all_day)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING id, calendar_id, creator_id, title, description, status, priority,
                     due_date, start_date, all_day, created_at, updated_at"#,
    )
    .bind(calendar_id)
    .bind(creator_id)
    .bind(title)
    .bind(description)
    .bind(status)
    .bind(priority)
    .bind(due_date)
    .bind(start_date)
    .bind(all_day)
    .fetch_one(pool)
    .await?;

    Ok(task)
}

pub async fn find_task_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Task>, AppError> {
    let task = sqlx::query_as::<_, Task>(
        r#"SELECT id, calendar_id, creator_id, title, description, status, priority,
                  due_date, start_date, all_day, created_at, updated_at
           FROM tasks
           WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(task)
}

pub async fn list_tasks(
    pool: &PgPool,
    calendar_id: Uuid,
    query: &TaskQuery,
) -> Result<Vec<Task>, AppError> {
    // Build a dynamic query with optional filters
    let tasks = sqlx::query_as::<_, Task>(
        r#"SELECT t.id, t.calendar_id, t.creator_id, t.title, t.description, t.status,
                  t.priority, t.due_date, t.start_date, t.all_day, t.created_at, t.updated_at
           FROM tasks t
           LEFT JOIN task_assignees ta ON ta.task_id = t.id
           WHERE t.calendar_id = $1
             AND ($2::timestamptz IS NULL OR t.due_date >= $2)
             AND ($3::timestamptz IS NULL OR t.due_date <= $3)
             AND ($4::text IS NULL OR t.status = $4)
             AND ($5::text IS NULL OR t.priority = $5)
             AND ($6::uuid IS NULL OR ta.user_id  = $6)
           ORDER BY t.created_at DESC"#,
    )
    .bind(calendar_id)
    .bind(query.date_from)
    .bind(query.date_to)
    .bind(&query.status)
    .bind(&query.priority)
    .bind(query.assignee)
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

pub async fn update_task(
    pool: &PgPool,
    id: Uuid,
    title: Option<&str>,
    description: Option<&str>,
    priority: Option<&str>,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    start_date: Option<chrono::DateTime<chrono::Utc>>,
    all_day: Option<bool>,
) -> Result<Option<Task>, AppError> {
    let task = sqlx::query_as::<_, Task>(
        r#"UPDATE tasks
           SET title       = COALESCE($2, title),
               description = COALESCE($3, description),
               priority    = COALESCE($4, priority),
               due_date    = COALESCE($5, due_date),
               start_date  = COALESCE($6, start_date),
               all_day     = COALESCE($7, all_day)
           WHERE id = $1
           RETURNING id, calendar_id, creator_id, title, description, status, priority,
                     due_date, start_date, all_day, created_at, updated_at"#,
    )
    .bind(id)
    .bind(title)
    .bind(description)
    .bind(priority)
    .bind(due_date)
    .bind(start_date)
    .bind(all_day)
    .fetch_optional(pool)
    .await?;

    Ok(task)
}

pub async fn update_task_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<Option<Task>, AppError> {
    let task = sqlx::query_as::<_, Task>(
        r#"UPDATE tasks SET status = $2
           WHERE id = $1
           RETURNING id, calendar_id, creator_id, title, description, status, priority,
                     due_date, start_date, all_day, created_at, updated_at"#,
    )
    .bind(id)
    .bind(status)
    .fetch_optional(pool)
    .await?;

    Ok(task)
}

pub async fn delete_task(pool: &PgPool, id: Uuid) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

// ─── Assignees ────────────────────────────────────────────────────────────────

pub async fn assign_user(
    pool: &PgPool,
    task_id: Uuid,
    user_id: Uuid,
) -> Result<TaskAssignee, AppError> {
    let assignee = sqlx::query_as::<_, TaskAssignee>(
        r#"INSERT INTO task_assignees (task_id, user_id)
           VALUES ($1, $2)
           ON CONFLICT DO NOTHING
           RETURNING task_id, user_id, assigned_at"#,
    )
    .bind(task_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(assignee)
}

pub async fn unassign_user(
    pool: &PgPool,
    task_id: Uuid,
    user_id: Uuid,
) -> Result<bool, AppError> {
    let result =
        sqlx::query("DELETE FROM task_assignees WHERE task_id = $1 AND user_id = $2")
            .bind(task_id)
            .bind(user_id)
            .execute(pool)
            .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_assignees(pool: &PgPool, task_id: Uuid) -> Result<Vec<TaskAssignee>, AppError> {
    let assignees = sqlx::query_as::<_, TaskAssignee>(
        "SELECT task_id, user_id, assigned_at FROM task_assignees WHERE task_id = $1",
    )
    .bind(task_id)
    .fetch_all(pool)
    .await?;

    Ok(assignees)
}

// ─── Labels ───────────────────────────────────────────────────────────────────

pub async fn add_label(
    pool: &PgPool,
    task_id: Uuid,
    label: &str,
    color: &str,
) -> Result<TaskLabel, AppError> {
    let lbl = sqlx::query_as::<_, TaskLabel>(
        r#"INSERT INTO task_labels (task_id, label, color)
           VALUES ($1, $2, $3)
           ON CONFLICT (task_id, label) DO UPDATE SET color = EXCLUDED.color
           RETURNING task_id, label, color"#,
    )
    .bind(task_id)
    .bind(label)
    .bind(color)
    .fetch_one(pool)
    .await?;

    Ok(lbl)
}

pub async fn remove_label(pool: &PgPool, task_id: Uuid, label: &str) -> Result<bool, AppError> {
    let result =
        sqlx::query("DELETE FROM task_labels WHERE task_id = $1 AND label = $2")
            .bind(task_id)
            .bind(label)
            .execute(pool)
            .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_labels(pool: &PgPool, task_id: Uuid) -> Result<Vec<TaskLabel>, AppError> {
    let labels = sqlx::query_as::<_, TaskLabel>(
        "SELECT task_id, label, color FROM task_labels WHERE task_id = $1",
    )
    .bind(task_id)
    .fetch_all(pool)
    .await?;

    Ok(labels)
}

// ─── Comments ─────────────────────────────────────────────────────────────────

pub async fn add_comment(
    pool: &PgPool,
    task_id: Uuid,
    user_id: Uuid,
    content: &str,
) -> Result<TaskComment, AppError> {
    let comment = sqlx::query_as::<_, TaskComment>(
        r#"INSERT INTO task_comments (task_id, user_id, content)
           VALUES ($1, $2, $3)
           RETURNING id, task_id, user_id, content, created_at, updated_at"#,
    )
    .bind(task_id)
    .bind(user_id)
    .bind(content)
    .fetch_one(pool)
    .await?;

    Ok(comment)
}

pub async fn list_comments(
    pool: &PgPool,
    task_id: Uuid,
) -> Result<Vec<TaskComment>, AppError> {
    let comments = sqlx::query_as::<_, TaskComment>(
        r#"SELECT id, task_id, user_id, content, created_at, updated_at
           FROM task_comments
           WHERE task_id = $1
           ORDER BY created_at ASC"#,
    )
    .bind(task_id)
    .fetch_all(pool)
    .await?;

    Ok(comments)
}
