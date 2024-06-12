use axum::{extract::{Query, State}, routing::get, Json, Router};

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW, routes::models::lesson_progress::{GetLessonProgressesPayload, LessonProgressPayload}};

pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/get_lesson_progresses", get(api_get_lesson_progresses_handler))
		.with_state(app_state)
}

#[utoipa::path(
	get,
	path = "/api/course/lesson/get_lesson_progresses",
	params(
		GetLessonProgressesPayload
	),
	responses(
		(status = 200, body = Vec<LessonProgressPayload>),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_get_lesson_progresses_handler(
    ctx: CtxW,
	State(app_state): State<AppState>,
	Query(paylod): Query<GetLessonProgressesPayload>,
) -> AppResult<Json<Vec<LessonProgressPayload>>> {
    let ctx = ctx.0;
	let user_id = ctx.user_id();

	let lesson_progress_query_repository = app_state.query_repository_manager.get_lesson_progress_repository();
	let lesson_progresses_data = lesson_progress_query_repository
		.get_lessons_progresses(&ctx, paylod.course_id, user_id)
		.await?
		.iter()
		.map(|lesson_progrerss_data| lesson_progrerss_data.into())
		.collect();

	Ok(Json(lesson_progresses_data))
}
