use axum::{extract::State, routing::post, Json, Router};
use lib_core::interactors::student::exercise::StudentExerciseInteractor;
use serde_json::{json, Value};

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW, routes::models::exercise::{ExerciseCompletionForSaveChanges, ExerciseCompletionId, ExerciseEstimatePayload, ExerciseId}};

pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/start_exercise", post(api_start_exercise_handler))
		.route("/save_changes", post(api_save_changes_handler))
		.route("/complete_attempt", post(api_complete_attempt_handler))
		.with_state(app_state)
}

#[utoipa::path(
	post,
	path = "/api/course/lesson/exercise/start_exercise",
	request_body = ExerciseId,
	responses(
		(status = 200, body = ExerciseCompletionId, description = "Exercise started successfully"),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_start_exercise_handler(
    ctx: CtxW,
	State(app_state): State<AppState>,
	Json(payload): Json<ExerciseId>,
) -> AppResult<Json<ExerciseCompletionId>> {
    let ctx = ctx.0;

    let repository_manager = app_state.command_repository_manager;
    let exercise_interactor = StudentExerciseInteractor::new(repository_manager);

    let id = exercise_interactor.start_exercise(&ctx, payload.exercise_id).await?;
	
	let result = ExerciseCompletionId { 
		exercise_completion_id: id,
	};

	Ok(Json(result))
}

#[utoipa::path(
	post,
	path = "/api/course/lesson/exercise/save_changes",
	request_body = ExerciseCompletionForSaveChanges,
	responses(
		(status = 200, description = "Exercise changes saved successfully"),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_save_changes_handler(
    ctx: CtxW,
	State(app_state): State<AppState>,
	Json(payload): Json<ExerciseCompletionForSaveChanges>,
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;

    let repository_manager = app_state.command_repository_manager;
    let exercise_interactor = StudentExerciseInteractor::new(repository_manager);

    exercise_interactor.save_exercise_execution_changes(&ctx, payload.exercise_completion_id, payload.body).await?;
	
	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}

#[utoipa::path(
	post,
	path = "/api/course/lesson/exercise/complete_attempt",
	request_body = ExerciseCompletionId,
	responses(
		(status = 200, body = ExerciseEstimatePayload, description = "Exercise changes saved successfully"),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_complete_attempt_handler(
    ctx: CtxW,
	State(app_state): State<AppState>,
	Json(payload): Json<ExerciseCompletionId>,
) -> AppResult<Json<ExerciseEstimatePayload>> {
    let ctx = ctx.0;

    let repository_manager = app_state.command_repository_manager;
    let exercise_interactor = StudentExerciseInteractor::new(repository_manager);

    let result = exercise_interactor.complete_exercise(&ctx, payload.exercise_completion_id).await?;
	

	Ok(Json(result.into()))
}