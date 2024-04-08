use axum::{extract::State, routing::post, Json, Router};
use lib_core::{core::lesson::LessonInteractor, model::lesson::LessonForCreate};
use lib_db::{command_repository::lesson::LessonCommandRepository, store::DbManager};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{error::AppResult, middleware::mw_auth::CtxW};

pub fn routes(dbm: DbManager) -> Router {
	Router::new()
		.route("/create", post(api_create_lesson_handler))
		.with_state(dbm)
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreatedLessonPayload {
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
		(status = 200, description = "Lesson created successfully", body = CreatedLessonPayload),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_create_lesson_handler(
    ctx: CtxW,
	State(dbm): State<DbManager>,
	Json(paylod): Json<LessonCreatePayload>,
) -> AppResult<Json<CreatedLessonPayload>> {
    let ctx = ctx.0;

    let lesson_c = LessonForCreate {
        course_id: paylod.course_id,
        titile: paylod.titile,
    };

	let repository = LessonCommandRepository::new(dbm);
	let lesson_interactor = LessonInteractor::new(&ctx, &repository);

    let lesson_id = lesson_interactor.create_lesson(lesson_c).await?;

    let created_lesson = CreatedLessonPayload {
        lesson_id,
    };

    let body = Json(created_lesson);

    Ok(body)
}