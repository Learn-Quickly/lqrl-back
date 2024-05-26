use axum::{extract::State, routing::put, Json, Router};
use lib_core::interactors::student::lesson::StudentLessonInteractor;
use serde_json::{json, Value};

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW, routes::models::lesson::StartLessonPayload};

pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/start_lesson", put(api_start_lesson_handler))
		.with_state(app_state)
}

#[utoipa::path(
	put,
	path = "/api/course/lesson/start_lesson",
	request_body = LessonCreatePayload,
	responses(
		(status = 200, description = "Lesson started successfully", body = LessonCreatedPayload),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_start_lesson_handler(
    ctx: CtxW,
	State(app_state): State<AppState>,
	Json(paylod): Json<StartLessonPayload>,
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;

	let command_repository_manager = app_state.command_repository_manager;
	let lesson_interactor = StudentLessonInteractor::new(command_repository_manager);

    lesson_interactor.start_lesson(&ctx, paylod.lesson_id).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}
