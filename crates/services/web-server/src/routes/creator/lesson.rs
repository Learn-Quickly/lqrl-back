use axum::{extract::State, routing::{delete, post, put}, Json, Router};
use lib_core::{interactors::creator::lesson::CreatorLessonInteractor, models::lesson::{LessonForChangeOreder, LessonForCreate, LessonForUpdate}};
use serde_json::{json, Value};

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW, routes::models::lesson::{LessonChangeOrderPayload, LessonCreatePayload, LessonCreatedPayload, LessonDeletePayload, LessonUpdatePayload}};

pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/create", post(api_create_lesson_handler))
		.route("/delete", delete(api_delete_lesson_handler))
		.route("/update", put(api_update_lesson_handler))
		.route("/change_order", put(api_lesson_change_order_handler))
		.with_state(app_state)
}

#[utoipa::path(
	post,
	path = "/api/course/lesson/create",
	request_body = LessonCreatePayload,
	responses(
		(status = 200, description = "Lesson created successfully", body = LessonCreatedPayload),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_create_lesson_handler(
    ctx: CtxW,
	State(app_state): State<AppState>,
	Json(paylod): Json<LessonCreatePayload>,
) -> AppResult<Json<LessonCreatedPayload>> {
    let ctx = ctx.0;

    let lesson_c = LessonForCreate {
        course_id: paylod.course_id,
        title: paylod.title,
		description: paylod.description,
    };

	let command_repository_manager = app_state.command_repository_manager;
	let lesson_interactor = CreatorLessonInteractor::new(command_repository_manager);

    let lesson_id = lesson_interactor.create_lesson(&ctx, lesson_c).await?;

    let created_lesson = LessonCreatedPayload {
        lesson_id,
    };

    let body = Json(created_lesson);

    Ok(body)
}

#[utoipa::path(
	delete,
	path = "/api/course/lesson/delete",
	request_body = LessonDeletePayload,
	responses(
		(status = 200, description = "Lesson deleted successfully"),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_delete_lesson_handler(
    ctx: CtxW,
	State(app_state): State<AppState>,
	Json(paylod): Json<LessonDeletePayload>,
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;

	let command_repository_manager = app_state.command_repository_manager;
	let lesson_interactor = CreatorLessonInteractor::new(command_repository_manager);

    lesson_interactor.delete_lesson(&ctx, paylod.lesson_id).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

    Ok(body)
}

#[utoipa::path(
	put,
	path = "/api/course/lesson/update",
	request_body = LessonUpdatePayload,
	responses(
		(status = 200, description = "Lesson updated successfully"),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_update_lesson_handler(
    ctx: CtxW,
	State(app_state): State<AppState>,
	Json(paylod): Json<LessonUpdatePayload>,
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;

    let lesson_u = LessonForUpdate {
        id: paylod.lesson_id,
        title: paylod.title,
		description: paylod.description,
    };

	let command_repository_manager = app_state.command_repository_manager;
	let lesson_interactor = CreatorLessonInteractor::new(command_repository_manager);

    lesson_interactor.update_lesson(&ctx, lesson_u).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

    Ok(body)
}

#[utoipa::path(
	put,
	path = "/api/course/lesson/change_order",
	request_body = LessonChangeOrderPayload,
	responses(
		(status = 200, description = "Lesson order updated successfully"),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_lesson_change_order_handler(
    ctx: CtxW,
	State(app_state): State<AppState>,
	Json(paylod): Json<LessonChangeOrderPayload>,
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;

    let lesson_c_o = LessonForChangeOreder {
        id: paylod.lesson_id,
		order: paylod.order,
    };

	let command_repository_manager = app_state.command_repository_manager;
	let lesson_interactor = CreatorLessonInteractor::new(command_repository_manager);

    lesson_interactor.change_order(&ctx, lesson_c_o).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

    Ok(body)
}