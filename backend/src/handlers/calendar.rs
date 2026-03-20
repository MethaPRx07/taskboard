use actix_web::{web, HttpResponse};
use uuid::Uuid;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::calendar::{AddMemberRequest, CreateCalendarRequest, UpdateCalendarRequest};
use crate::repository::calendar as cal_repo;
use crate::state::AppState;

// ─── POST /api/v1/calendars ──────────────────────────────────────────────────

pub async fn create_calendar(
    state: web::Data<AppState>,
    auth: AuthUser,
    body: web::Json<CreateCalendarRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;

    let color = body
        .color
        .as_deref()
        .unwrap_or("#4285F4");
    let is_public = body.is_public.unwrap_or(false);

    let calendar = cal_repo::create_calendar(
        &state.db,
        auth.user_id(),
        &body.name,
        body.description.as_deref(),
        color,
        is_public,
    )
    .await?;

    Ok(HttpResponse::Created().json(calendar))
}

// ─── GET /api/v1/calendars ───────────────────────────────────────────────────

pub async fn list_calendars(
    state: web::Data<AppState>,
    auth: AuthUser,
) -> Result<HttpResponse, AppError> {
    let calendars = cal_repo::list_calendars_for_user(&state.db, auth.user_id()).await?;
    Ok(HttpResponse::Ok().json(calendars))
}

// ─── GET /api/v1/calendars/{id} ──────────────────────────────────────────────

pub async fn get_calendar(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let calendar_id = path.into_inner();

    let calendar = cal_repo::find_calendar_by_id(&state.db, calendar_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Calendar {} not found", calendar_id)))?;

    // Allow access if public OR user is owner/member
    if !calendar.is_public {
        let member = cal_repo::get_member(&state.db, calendar_id, auth.user_id()).await?;
        if member.is_none() && calendar.owner_id != auth.user_id() {
            return Err(AppError::Forbidden(
                "You do not have access to this calendar".to_string(),
            ));
        }
    }

    Ok(HttpResponse::Ok().json(calendar))
}

// ─── PUT /api/v1/calendars/{id} ──────────────────────────────────────────────

pub async fn update_calendar(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateCalendarRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;
    let calendar_id = path.into_inner();

    let calendar = cal_repo::find_calendar_by_id(&state.db, calendar_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Calendar {} not found", calendar_id)))?;

    // Only owner or editors can update
    let member = cal_repo::get_member(&state.db, calendar_id, auth.user_id()).await?;
    let is_owner = calendar.owner_id == auth.user_id();
    let is_editor = member
        .as_ref()
        .map(|m| m.role == "editor" || m.role == "owner")
        .unwrap_or(false);

    if !is_owner && !is_editor {
        return Err(AppError::Forbidden(
            "Only the owner or editors can update this calendar".to_string(),
        ));
    }

    let updated = cal_repo::update_calendar(
        &state.db,
        calendar_id,
        body.name.as_deref(),
        body.description.as_deref(),
        body.color.as_deref(),
        body.is_public,
    )
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Calendar {} not found", calendar_id)))?;

    Ok(HttpResponse::Ok().json(updated))
}

// ─── DELETE /api/v1/calendars/{id} ───────────────────────────────────────────

pub async fn delete_calendar(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let calendar_id = path.into_inner();

    let calendar = cal_repo::find_calendar_by_id(&state.db, calendar_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Calendar {} not found", calendar_id)))?;

    if calendar.owner_id != auth.user_id() {
        return Err(AppError::Forbidden(
            "Only the owner can delete this calendar".to_string(),
        ));
    }

    cal_repo::delete_calendar(&state.db, calendar_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

// ─── POST /api/v1/calendars/{id}/members ─────────────────────────────────────

pub async fn add_member(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<AddMemberRequest>,
) -> Result<HttpResponse, AppError> {
    let calendar_id = path.into_inner();

    let calendar = cal_repo::find_calendar_by_id(&state.db, calendar_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Calendar {} not found", calendar_id)))?;

    if calendar.owner_id != auth.user_id() {
        return Err(AppError::Forbidden(
            "Only the owner can manage members".to_string(),
        ));
    }

    if !body.is_valid_role() {
        return Err(AppError::BadRequest(
            "Role must be 'editor' or 'viewer'".to_string(),
        ));
    }

    let member = cal_repo::add_member(&state.db, calendar_id, body.user_id, &body.role).await?;

    Ok(HttpResponse::Created().json(member))
}

// ─── DELETE /api/v1/calendars/{id}/members/{user_id} ─────────────────────────

pub async fn remove_member(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (calendar_id, target_user_id) = path.into_inner();

    let calendar = cal_repo::find_calendar_by_id(&state.db, calendar_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Calendar {} not found", calendar_id)))?;

    // Owner can remove anyone; users can remove themselves
    if calendar.owner_id != auth.user_id() && target_user_id != auth.user_id() {
        return Err(AppError::Forbidden(
            "Only the owner can remove other members".to_string(),
        ));
    }

    cal_repo::remove_member(&state.db, calendar_id, target_user_id).await?;

    Ok(HttpResponse::NoContent().finish())
}
