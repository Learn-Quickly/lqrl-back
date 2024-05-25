use axum::{extract::{Path, State}, routing::get, Json, Router};

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW, routes::models::lesson::LessonDataPayload};


pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/get_lessons/:i64", get(api_get_lessons_handler))
		.with_state(app_state)
}

#[utoipa::path(
	get,
	path = "/api/course/lesson/get_lessons/{course_id}",
	params(
		("course_id", description = "ID of the course")
	),
	responses(
		(status = 200, body=CoursePayload),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_get_lessons_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Path(course_id): Path<i64>,
) -> AppResult<Json<Vec<LessonDataPayload>>> {
	let ctx = ctx.0;

	let lesson_query_repository = app_state.query_repository_manager.get_lesson_repository();
	let lessons = lesson_query_repository
        .get_course_lessons_ordered(&ctx, course_id)
        .await?
        .iter()
        .map(|lesson| lesson.clone().into()).collect();

	Ok(Json(lessons))
}