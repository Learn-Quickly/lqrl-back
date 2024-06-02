use axum::{extract::{Path, State}, routing::get, Json, Router};

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW, routes::models::exercise::ExercisePayload};


pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/get_lesson_exercises/:i64", get(api_get_lesson_exercises_handler))
		.route("/get_exercise/:i64", get(api_get_exercise_handler))
		.with_state(app_state)
}

#[utoipa::path(
	get,
	path = "/api/course/lesson/exercise/get_lesson_exercises",
	params(
		("lesson_id", description = "ID of the lesson")
	),
	responses(
		(status = 200, body=Vec<ExercisePayload>),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_get_lesson_exercises_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Path(lesson_id): Path<i64>,
) -> AppResult<Json<Vec<ExercisePayload>>> {
	let ctx = ctx.0;

	let exercise_query_repository = app_state.query_repository_manager.get_exercise_repository();

	let exercises = exercise_query_repository
        .get_lesson_exercises(&ctx, lesson_id)
        .await?
        .iter()
        .map(|exercise| exercise.clone().into()).collect();

	Ok(Json(exercises))
}

#[utoipa::path(
	get,
	path = "/api/course/lesson/exercise/get_exercise/{exercise_id}",
	params(
		("exercise_id", description = "ID of the exercise")
	),
	responses(
		(status = 200, body=ExercisePayload),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_get_exercise_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Path(exercise_id): Path<i64>,
) -> AppResult<Json<ExercisePayload>> {
	let ctx = ctx.0;

	let exercise_query_repository = app_state.query_repository_manager.get_exercise_repository();

	let exercise = exercise_query_repository
        .get_exercise(&ctx, exercise_id)
        .await?;

	Ok(Json(exercise.into()))
}