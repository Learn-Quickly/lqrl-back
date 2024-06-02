use axum::{extract::State, routing::{post, put}, Json, Router};
use lib_core::{interactors::creator::exercise::CreatorExerciseInteractor, models::exercise::{ExerciseForChangeOrder, ExerciseForCreate, ExerciseForUpdate}};
use serde_json::{json, Value};

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW, routes::models::exercise::{ExerciseChangeOrderPayload, ExerciseCreatePayload, ExerciseCreatedPayload, ExerciseForUpdatePayload}};

pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/create", post(api_create_exercise_handler))
		.route("/update", put(api_update_exercise_handler))
		.route("/change_order", put(api_exercise_change_order_handler))
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
	Json(payload): Json<ExerciseCreatePayload>,
) -> AppResult<Json<ExerciseCreatedPayload>> {
    let ctx = ctx.0;

    let exercise_c = ExerciseForCreate { 
        lesson_id: payload.lesson_id, 
        title: payload.title.clone(), 
        description: payload.description.clone(), 
        exercise_type: payload.exercise_type.try_into()?, 
        difficult: payload.difficult.try_into()?, 
        time_to_complete: payload.time_to_complete,
		answer_body: payload.answer_body.clone(),
		exercise_body: payload.exercise_body.clone(),
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

#[utoipa::path(
	put,
	path = "/api/course/lesson/exercise/update",
	request_body = LessonUpdatePayload,
	responses(
		(status = 200, description = "Exercise updated successfully"),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_update_exercise_handler(
    ctx: CtxW,
	State(app_state): State<AppState>,
	Json(payload): Json<ExerciseForUpdatePayload>,
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;
    let exercise_type = if let Some(t) = payload.exercise_type {
        Some(t.try_into()?)
    } else {
        None
    };

    let difficult = if let Some(d) = payload.difficult {
        Some(d.try_into()?)
    } else {
        None
    };

    let lesson_u = ExerciseForUpdate { 
        id: payload.exercise_id, 
        title: payload.title.clone(), 
        description: payload.description.clone(), 
        exercise_type, 
        exercise_body: payload.exercise_body.clone(), 
        answer_body: payload.answer_body.clone(), 
        difficult, 
        time_to_complete: payload.time_to_complete,
    };

	let command_repository_manager = app_state.command_repository_manager;
	let exercise_interactor = CreatorExerciseInteractor::new(command_repository_manager);

    exercise_interactor.update_exercise(&ctx, lesson_u).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

    Ok(body)
}

#[utoipa::path(
	put,
	path = "/api/course/lesson/exercise/change_order",
	request_body = ExerciseChangeOrderPayload,
	responses(
		(status = 200, description = "Exercise order updated successfully"),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_exercise_change_order_handler(
    ctx: CtxW,
	State(app_state): State<AppState>,
	Json(paylod): Json<ExerciseChangeOrderPayload>,
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;

    let exercise_c_o = ExerciseForChangeOrder {
        id: paylod.exercise_id,
		order: paylod.order,
    };

	let command_repository_manager = app_state.command_repository_manager;
	let exercise_interactor = CreatorExerciseInteractor::new(command_repository_manager);

    exercise_interactor.change_order(&ctx, exercise_c_o).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

    Ok(body)
}