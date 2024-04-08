use axum::{extract::State, routing::{post, put}, Json, Router};
use lib_core::{core::lesson::LessonInteractor, model::lesson::{LessonForChangeOreder, LessonForCreate, LessonForUpdate}};
use lib_db::{command_repository::lesson::LessonCommandRepository, store::DbManager};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;

use crate::{error::AppResult, middleware::mw_auth::CtxW};

pub fn routes(dbm: DbManager) -> Router {
	Router::new()
		.route("/create", post(api_create_lesson_handler))
		.route("/update", put(api_update_lesson_handler))
		.route("/change_order", put(api_lesson_change_order_handler))
		.with_state(dbm)
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LessonCreatedPayload {
    lesson_id: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LessonCreatePayload {
    pub course_id: i64,
    pub titile: String,
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
	State(dbm): State<DbManager>,
	Json(paylod): Json<LessonCreatePayload>,
) -> AppResult<Json<LessonCreatedPayload>> {
    let ctx = ctx.0;

    let lesson_c = LessonForCreate {
        course_id: paylod.course_id,
        titile: paylod.titile,
    };

	let repository = LessonCommandRepository::new(dbm);
	let lesson_interactor = LessonInteractor::new(&ctx, &repository);

    let lesson_id = lesson_interactor.create_lesson(lesson_c).await?;

    let created_lesson = LessonCreatedPayload {
        lesson_id,
    };

    let body = Json(created_lesson);

    Ok(body)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LessonUpdatePayload {
    pub lesson_id: i64,
    pub titile: String,
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
	State(dbm): State<DbManager>,
	Json(paylod): Json<LessonUpdatePayload>,
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;

    let lesson_u = LessonForUpdate {
        id: paylod.lesson_id,
        titile: paylod.titile,
    };

	let repository = LessonCommandRepository::new(dbm);
	let lesson_interactor = LessonInteractor::new(&ctx, &repository);

    lesson_interactor.update_lesson(lesson_u).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

    Ok(body)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LessonChangeOrderPayload {
    pub lesson_id: i64,
    pub order: i32,
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
	State(dbm): State<DbManager>,
	Json(paylod): Json<LessonChangeOrderPayload>,
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;

    let lesson_c_o = LessonForChangeOreder {
        id: paylod.lesson_id,
		order: paylod.order,
    };

	let repository = LessonCommandRepository::new(dbm);
	let lesson_interactor = LessonInteractor::new(&ctx, &repository);

    lesson_interactor.change_order(lesson_c_o).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

    Ok(body)
}