use axum::{extract::{Path, Query, State}, routing::get, Json, Router};
use lib_db::query_repository::course::CourseQuery;

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW, routes::models::course::{CourseFilterPayload, CoursePayload}};

pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/get_course/:i64", get(api_get_course_handler))
		.route("/get_courses/", get(api_get_courses_handler))
		.with_state(app_state)
}

#[utoipa::path(
	get,
	path = "/api/course/get_course/{course_id}",
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
async fn api_get_course_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Path(course_id): Path<i64>,
) -> AppResult<Json<CoursePayload>> {
	let ctx = ctx.0;

	let course_query_repository = app_state.query_repository_manager.get_course_repository();
	let course = course_query_repository.get(&ctx, course_id).await?;

	Ok(Json(course.try_into()?))
}

#[utoipa::path(
	get,
	path = "/api/course/get_courses/",
	params(
		CourseFilterPayload
	),
	// params(
	// 	("filter_payload", description = 
	// 		"It contains two optional fields:\n
	// 			1) filters - list of filters\n
	// 			2) list_options - contains offset, limit, order_bys\n
	// 		Documentation: https://lib.rs/crates/modql"
	// 	),
	// ),
	responses(
		(status = 200, body = Vec<CoursePayload>),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_get_courses_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Query(filter_payload): Query<CourseFilterPayload>,
) -> AppResult<Json<Vec<CoursePayload>>> {
	let ctx = ctx.0;

	let filters = if let Some(filters) = filter_payload.filters {
		serde_json::from_value(filters)?
	} else {
		None
	};

	let list_options = if let Some(list_options) = filter_payload.list_options {
		serde_json::from_value(list_options)?
	} else {
		None
	};

	let course_query_repository = app_state.query_repository_manager.get_course_repository();
	let courses: Vec<CourseQuery> = course_query_repository.list(&ctx, filters, list_options).await?;

	let mut body: Vec<CoursePayload> = Vec::new();
	for course in courses {
		body.push(course.try_into()?)
	}

	Ok(Json(body))
}