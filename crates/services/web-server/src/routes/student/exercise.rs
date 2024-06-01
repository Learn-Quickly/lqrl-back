use axum::{extract::State, routing::post, Json, Router};
use lib_core::interactors::student::exercise::StudentExerciseInteractor;
use serde_json::{json, Value};

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW, routes::models::exercise::ExerciseId};

pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/start_exercise", post(api_start_exercise_handler))
		.with_state(app_state)
}

#[utoipa::path(
	post,
	path = "/api/course/lesson/exercise/start_exercise",
	request_body = ExerciseId,
	responses(
		(status = 200, description = "Exercise started successfully"),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_start_exercise_handler(
    ctx: CtxW,
	State(app_state): State<AppState>,
	Json(payload): Json<ExerciseId>,
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;

    let repository_manager = app_state.command_repository_manager;
    let exercise_interactor = StudentExerciseInteractor::new(repository_manager);

    exercise_interactor.start_exercise(&ctx, payload.exercise_id).await?;
    
	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}