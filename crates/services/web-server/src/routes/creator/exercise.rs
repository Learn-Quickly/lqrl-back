use axum::{extract::State, routing::post, Json, Router};
use lib_core::{interactors::creator::exercise::CreatorExerciseInteractor, models::exercise::Exercise};

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW, routes::models::exercise::{ExerciseCreatePayload, ExerciseCreatedPayload}};

pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/create", post(api_create_exercise_handler))
		.with_state(app_state)
}

#[utoipa::path(
	post,
	path = "/api/course/lesson/exercise/create",
	request_body = ExerciseCreatePayload,
	responses(
		(status = 200, description = "Exercise created successfully", body = ExerciseCreatedPayload),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_create_exercise_handler(
    ctx: CtxW,
	State(app_state): State<AppState>,
	Json(paylod): Json<ExerciseCreatePayload>,
) -> AppResult<Json<ExerciseCreatedPayload>> {
    let ctx = ctx.0;

    let exercise_c = Exercise { 
        lesson_id: paylod.lesson_id, 
        title: paylod.title.clone(), 
        description: paylod.description.clone(), 
        exercise_type: paylod.exercise_type.try_into()?, 
        body: paylod.body.clone(), 
        difficult: paylod.difficult.try_into()?, 
        time_to_complete: paylod.time_to_complete,
    };

	let command_repository_manager = app_state.command_repository_manager;
	let exercise_interactor = CreatorExerciseInteractor::new(command_repository_manager);

    let exercise_id = exercise_interactor.create_exercise(&ctx, exercise_c).await?;

    let created_lesson = ExerciseCreatedPayload {
        exercise_id
    };

    let body = Json(created_lesson);

    Ok(body)
}