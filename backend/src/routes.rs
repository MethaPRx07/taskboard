use actix_web::web;

use crate::handlers::{auth, calendar, task};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // ── Health check ─────────────────────────────────────────────────────
        .route("/health", web::get().to(health_check))

        // ── API v1 ───────────────────────────────────────────────────────────
        .service(
            web::scope("/api/v1")

                // ── Auth routes (no JWT required for register/login/refresh)
                .service(
                    web::scope("/auth")
                        .route("/register", web::post().to(auth::register))
                        .route("/login",    web::post().to(auth::login))
                        .route("/refresh",  web::post().to(auth::refresh))
                        .route("/logout",   web::post().to(auth::logout))   // JWT required
                        .route("/me",       web::get().to(auth::me)),       // JWT required
                )

                // ── Calendar routes (all require JWT)
                .service(
                    web::scope("/calendars")
                        .route("",             web::get().to(calendar::list_calendars))
                        .route("",             web::post().to(calendar::create_calendar))
                        .route("/{id}",        web::get().to(calendar::get_calendar))
                        .route("/{id}",        web::put().to(calendar::update_calendar))
                        .route("/{id}",        web::delete().to(calendar::delete_calendar))
                        // Members
                        .route("/{id}/members",              web::post().to(calendar::add_member))
                        .route("/{id}/members/{user_id}",    web::delete().to(calendar::remove_member))
                        // Tasks nested under calendar
                        .route("/{id}/tasks",  web::get().to(task::list_tasks))
                        .route("/{id}/tasks",  web::post().to(task::create_task)),
                )

                // ── Task routes (all require JWT)
                .service(
                    web::scope("/tasks")
                        .route("/{id}",                       web::get().to(task::get_task))
                        .route("/{id}",                       web::put().to(task::update_task))
                        .route("/{id}",                       web::delete().to(task::delete_task))
                        .route("/{id}/status",                web::patch().to(task::update_task_status))
                        // Assignees
                        .route("/{id}/assignees",             web::post().to(task::assign_user))
                        .route("/{id}/assignees/{user_id}",   web::delete().to(task::unassign_user))
                        // Labels
                        .route("/{id}/labels",                web::post().to(task::add_label))
                        .route("/{id}/labels/{label}",        web::delete().to(task::remove_label))
                        // Comments
                        .route("/{id}/comments",              web::get().to(task::list_comments))
                        .route("/{id}/comments",              web::post().to(task::add_comment)),
                ),
        );
}

async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "calendar-task-api",
        "version": "1.0.0"
    }))
}
