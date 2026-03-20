use actix_web::{web, HttpResponse};
use uuid::Uuid;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::task::{
    AddCommentRequest, AddLabelRequest, AssignUserRequest, CreateTaskRequest,
    TaskQuery, UpdateTaskRequest, UpdateTaskStatusRequest,
};
use crate::repository::{calendar as cal_repo, task as task_repo};
use crate::state::AppState;

// ─── helper: verify user can access the task's calendar ──────────────────────

async fn verify_calendar_access(
    state: &AppState,
    calendar_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let calendar = cal_repo::find_calendar_by_id(&state.db, calendar_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Calendar {} not found", calendar_id)))?;

    if !calendar.is_public {
        let member = cal_repo::get_member(&state.db, calendar_id, user_id).await?;
        if member.is_none() && calendar.owner_id != user_id {
            return Err(AppError::Forbidden(
                "You do not have access to this calendar".to_string(),
            ));
        }
    }
    Ok(())
}

// ─── POST /api/v1/calendars/{calendar_id}/tasks ───────────────────────────────

pub async fn create_task(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<CreateTaskRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let calendar_id = path.into_inner();

    verify_calendar_access(&state, calendar_id, auth.user_id()).await?;

    let status = body.status.as_deref().unwrap_or("todo");
    let priority = body.priority.as_deref().unwrap_or("medium");

    let task = task_repo::create_task(
        &state.db,
        calendar_id,
        auth.user_id(),
        &body.title,
        body.description.as_deref(),
        status,
        priority,
        body.due_date,
        body.start_date,
        body.all_day.unwrap_or(false),
    )
    .await?;

    Ok(HttpResponse::Created().json(task))
}

// ─── GET /api/v1/calendars/{calendar_id}/tasks ────────────────────────────────

pub async fn list_tasks(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    query: web::Query<TaskQuery>,
) -> Result<HttpResponse, AppError> {
    let calendar_id = path.into_inner();

    verify_calendar_access(&state, calendar_id, auth.user_id()).await?;

    let tasks = task_repo::list_tasks(&state.db, calendar_id, &query).await?;

    Ok(HttpResponse::Ok().json(tasks))
}

// ─── GET /api/v1/tasks/{id} ───────────────────────────────────────────────────

pub async fn get_task(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let task_id = path.into_inner();

    let task = task_repo::find_task_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

    verify_calendar_access(&state, task.calendar_id, auth.user_id()).await?;

    Ok(HttpResponse::Ok().json(task))
}

// ─── PUT /api/v1/tasks/{id} ───────────────────────────────────────────────────

pub async fn update_task(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateTaskRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let task_id = path.into_inner();

    let task = task_repo::find_task_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

    verify_calendar_access(&state, task.calendar_id, auth.user_id()).await?;

    let updated = task_repo::update_task(
        &state.db,
        task_id,
        body.title.as_deref(),
        body.description.as_deref(),
        body.priority.as_deref(),
        body.due_date,
        body.start_date,
        body.all_day,
    )
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

    Ok(HttpResponse::Ok().json(updated))
}

// ─── DELETE /api/v1/tasks/{id} ────────────────────────────────────────────────

pub async fn delete_task(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let task_id = path.into_inner();

    let task = task_repo::find_task_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

    // Only creator or calendar owner can delete
    let calendar = cal_repo::find_calendar_by_id(&state.db, task.calendar_id)
        .await?
        .ok_or(AppError::InternalServerError)?;

    if task.creator_id != auth.user_id() && calendar.owner_id != auth.user_id() {
        return Err(AppError::Forbidden(
            "Only the task creator or calendar owner can delete this task".to_string(),
        ));
    }

    task_repo::delete_task(&state.db, task_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

// ─── PATCH /api/v1/tasks/{id}/status ─────────────────────────────────────────

pub async fn update_task_status(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateTaskStatusRequest>,
) -> Result<HttpResponse, AppError> {
    if !body.is_valid() {
        return Err(AppError::BadRequest(
            "Status must be: todo | in_progress | done | cancelled".to_string(),
        ));
    }

    let task_id = path.into_inner();

    let task = task_repo::find_task_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

    verify_calendar_access(&state, task.calendar_id, auth.user_id()).await?;

    let updated = task_repo::update_task_status(&state.db, task_id, &body.status)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

    Ok(HttpResponse::Ok().json(updated))
}

// ─── POST /api/v1/tasks/{id}/assignees ───────────────────────────────────────

pub async fn assign_user(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<AssignUserRequest>,
) -> Result<HttpResponse, AppError> {
    let task_id = path.into_inner();

    let task = task_repo::find_task_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

    verify_calendar_access(&state, task.calendar_id, auth.user_id()).await?;

    let assignee = task_repo::assign_user(&state.db, task_id, body.user_id).await?;

    Ok(HttpResponse::Created().json(assignee))
}

// ─── DELETE /api/v1/tasks/{id}/assignees/{user_id} ───────────────────────────

pub async fn unassign_user(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (task_id, target_user_id) = path.into_inner();

    let task = task_repo::find_task_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

    verify_calendar_access(&state, task.calendar_id, auth.user_id()).await?;

    task_repo::unassign_user(&state.db, task_id, target_user_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

// ─── POST /api/v1/tasks/{id}/labels ──────────────────────────────────────────

pub async fn add_label(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<AddLabelRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let task_id = path.into_inner();

    let task = task_repo::find_task_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

    verify_calendar_access(&state, task.calendar_id, auth.user_id()).await?;

    let color = body.color.as_deref().unwrap_or("#808080");
    let label = task_repo::add_label(&state.db, task_id, &body.label, color).await?;

    Ok(HttpResponse::Created().json(label))
}

// ─── DELETE /api/v1/tasks/{id}/labels/{label} ────────────────────────────────

pub async fn remove_label(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<(Uuid, String)>,
) -> Result<HttpResponse, AppError> {
    let (task_id, label_name) = path.into_inner();

    let task = task_repo::find_task_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

    verify_calendar_access(&state, task.calendar_id, auth.user_id()).await?;

    task_repo::remove_label(&state.db, task_id, &label_name).await?;

    Ok(HttpResponse::NoContent().finish())
}

// ─── POST /api/v1/tasks/{id}/comments ────────────────────────────────────────

pub async fn add_comment(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<AddCommentRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let task_id = path.into_inner();

    let task = task_repo::find_task_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

    verify_calendar_access(&state, task.calendar_id, auth.user_id()).await?;

    let comment = task_repo::add_comment(&state.db, task_id, auth.user_id(), &body.content).await?;

    Ok(HttpResponse::Created().json(comment))
}

// ─── GET /api/v1/tasks/{id}/comments ─────────────────────────────────────────

pub async fn list_comments(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let task_id = path.into_inner();

    let task = task_repo::find_task_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

    verify_calendar_access(&state, task.calendar_id, auth.user_id()).await?;

    let comments = task_repo::list_comments(&state.db, task_id).await?;

    Ok(HttpResponse::Ok().json(comments))
}
