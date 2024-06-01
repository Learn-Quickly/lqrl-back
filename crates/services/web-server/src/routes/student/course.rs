use axum::{extract::{Query, State}, routing::{get, put}, Json, Router};
use lib_core::interactors::student::course::StudentCourseInteractor;
use lib_db::query_repository::course::CourseQuery;
use serde_json::{json, Value};

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW, routes::models::course::{CourseFilterPayload, CourseId, CoursePayload, CoursesPayload}};

pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/register_for_course", put(api_register_for_course_handler))
		.route("/unsubscribe_from_course", put(api_unsubscribe_from_course_handler))
		.route("/get_user_courses_registered/", get(api_get_user_courses_registered_handler))
		.with_state(app_state)
}

#[utoipa::path(
	put,
	path = "/api/course/register_for_course",
	request_body = CourseId,
	responses(
		(status = 200),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_register_for_course_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Json(course_id): Json<CourseId>
) -> AppResult<Json<Value>> {
	let ctx = ctx.0;
	let course_id = course_id.course_id;

	let command_repository_manager = app_state.command_repository_manager;
	let course_interactor = StudentCourseInteractor::new(command_repository_manager);

	course_interactor.register_for_course(&ctx, course_id).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}

#[utoipa::path(
	put,
	path = "/api/course/unsubscribe_from_course",
	request_body = CourseId,
	responses(
		(status = 200),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_unsubscribe_from_course_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Json(course_id): Json<CourseId>
) -> AppResult<Json<Value>> {
	let ctx = ctx.0;
	let course_id = course_id.course_id;

	let command_repository_manager = app_state.command_repository_manager;
	let course_interactor = StudentCourseInteractor::new(command_repository_manager);

	course_interactor.unsubscribe_from_course(&ctx, course_id).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}

#[utoipa::path(
	get,
	path = "/api/course/get_user_courses_registered/",
	params(
		CourseFilterPayload
	),
	responses(
		(status = 200, body=CoursesPayload),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_get_user_courses_registered_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Query(filter_payload): Query<CourseFilterPayload>,
) -> AppResult<Json<CoursesPayload>> {
	let ctx = ctx.0;
	let user_id = ctx.user_id();

	let filters = if let Some(filters) = filter_payload.filters.clone() {
		serde_json::from_str(&filters)?
	} else {
		None
	};

	let list_options = if let Some(list_options) = filter_payload.list_options.clone() {
		serde_json::from_str(&list_options)?
	} else {
		None
	};

	let course_query_repository = app_state.query_repository_manager.get_course_repository();
	let courses: Vec<CourseQuery> = course_query_repository.get_user_courses_registered(&ctx, user_id, filters, list_options).await?;

	let mut courses_res: Vec<CoursePayload> = Vec::new();
	for course in courses {
		courses_res.push(course.try_into()?)
	}

	let filters = if let Some(filters) = filter_payload.filters.clone() {
		serde_json::from_str(&filters)?
	} else {
		None
	};

	let number_of_courses = course_query_repository.get_user_courses_registered_count(&ctx, user_id, filters).await?;

	let result = CoursesPayload { 
		courses: courses_res, 
		count: number_of_courses, 
	};

	Ok(Json(result))
}