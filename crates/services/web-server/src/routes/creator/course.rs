use axum::{extract::{Multipart, Path, Query, State}, routing::{get, post, put}, Json, Router};
use lib_core::{interactors::creator::course::CreatorCourseInteractor, models::course::{CourseForCreate, CourseForUpdate}};
use lib_db::query_repository::course::CourseQuery;
use lib_utils::time::now_utc_sec;
use serde_json::{json, Value};
use tracing::info;

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW, routes::models::{course::{CourseCreateDraftPayload, CourseFilterPayload, CourseId, CoursePayload, CoursePointStatisticsPayload, CourseUpdatePayload, CoursesPayload, CreatedCourseDraft}, user::{GetAttendatsPayload, UserPayload, UsersPayload}}};

pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/create_course_draft", post(api_create_course_draft_handler))
		.route("/update", put(api_update_course_handler))
		.route("/set_course_img/:i64", put(api_set_course_img_handler))
		.route("/publish_course", put(api_publish_course_handler))
		.route("/archive_course", put(api_archive_course_handler))
		.route("/get_created_courses", get(api_get_created_courses_handler))
		.route("/get_attendants", get(api_get_attendants))
		.route("/get_point_statistics/:i64", get(api_get_point_statistics_handler))
		.with_state(app_state)
}

#[utoipa::path(
	post,
	path = "/api/course/create_course_draft",
	request_body = CourseCreateDraftPayload,
	responses(
		(status = 200, description = "Course draft created successfully", body = CreatedCourseDraft),
		(status = 500, description = "Course already exists. The title must be unique"),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_create_course_draft_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Json(paylod): Json<CourseCreateDraftPayload>
) -> AppResult<Json<CreatedCourseDraft>> {
	let ctx = ctx.0;

	let course_c = CourseForCreate {
		title: paylod.title, 
		description: paylod.description,
		course_type: paylod.course_type,
		price: paylod.price, 
		color: paylod.color, 
		date_created: now_utc_sec(), 
	};

	let command_repository_manager = app_state.command_repository_manager;
	let course_interactor = CreatorCourseInteractor::new(command_repository_manager);

	let course_id = course_interactor.create_draft(&ctx, course_c).await?;

	let created_course_draft = CreatedCourseDraft {
    	course_id,
	};

	let body = Json(created_course_draft);
	
	Ok(body)
}

#[utoipa::path(
	put,
	path = "/api/course/update",
	request_body = CourseUpdatePayload,
	responses(
		(status = 200),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_update_course_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Json(payload): Json<CourseUpdatePayload>
) -> AppResult<Json<Value>> {
	let ctx = ctx.0;

	let course_for_u = CourseForUpdate {
    	title: payload.title,
    	description: payload.description,
    	course_type: payload.course_type,
    	price: payload.price,
    	color: payload.color,
    	img_url: None,
	};

	let command_repository_manager = app_state.command_repository_manager;
	let course_interactor = CreatorCourseInteractor::new(command_repository_manager);

	course_interactor.update_course(&ctx, course_for_u, payload.id).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}

#[utoipa::path(
	put,
	path = "/api/course/publish_course",
	request_body = CourseId,
	responses(
		(status = 200),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_publish_course_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Json(course_id): Json<CourseId>,
) -> AppResult<Json<Value>> {
	let ctx = ctx.0;

	let command_repository_manager = app_state.command_repository_manager;
	let course_interactor = CreatorCourseInteractor::new(command_repository_manager);

	course_interactor.publish_course(&ctx, course_id.course_id).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}

#[utoipa::path(
	put,
	path = "/api/course/archive_course",
	request_body = CourseId,
	responses(
		(status = 200),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_archive_course_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Json(course_id): Json<CourseId>,
) -> AppResult<Json<Value>> {
	let ctx = ctx.0;

	let command_repository_manager = app_state.command_repository_manager;
	let course_interactor = CreatorCourseInteractor::new(command_repository_manager);

	course_interactor.archive_course(&ctx, course_id.course_id).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}

#[utoipa::path(
	put,
	path = "/api/course/set_course_img/{course_id}",
	params(
		("course_id", description = "ID of the course for which we set an avatar")
	),
	request_body(content_type = "multipart/formdata", content = Vec<u8>),
	responses(
		(status = 200, description = "Course image successfully set"),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_set_course_img_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Path(course_id): Path<i64>,
	mut multipart: Multipart,
) -> AppResult<Json<Value>> {
	let ctx = ctx.0;

	let command_repository_manager = app_state.command_repository_manager;
	let course_interactor = CreatorCourseInteractor::new(command_repository_manager);

    while let Some(field) = multipart.next_field().await? {
        let field_name = if let Some(field_name) = field.name() {
			field_name.to_string()
		} else {
			continue;
		};
		
        if field_name == "image" {
            let data = field.bytes().await?;
			let img_url = course_interactor.set_course_img(&ctx, course_id, &data).await?;

			let body = Json(json!({
				"result": {
					"img_url": img_url,
				}
			}));

			return Ok(body);
        }
    }

	let body = Json(json!({
		"result": {
			"error": "file error"
		}
	}));

    Ok(body)
}

#[utoipa::path(
	get,
	path = "/api/course/get_created_courses",
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

async fn api_get_created_courses_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Query(filter_payload): Query<CourseFilterPayload>,
) -> AppResult<Json<CoursesPayload>> {
	let ctx = ctx.0;
	let user_id = ctx.user_id();

	let filters = if let Some(filters) = filter_payload.filters.clone() {
		info!(filters);
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
	let courses: Vec<CourseQuery> = course_query_repository.get_user_courses_created(&ctx, user_id, filters, list_options).await?;
	let mut courses_res: Vec<CoursePayload> = Vec::new();
	for course in courses {
		courses_res.push(course.try_into()?)
	}

	let filters = if let Some(filters) = filter_payload.filters.clone() {
		info!(filters);
		serde_json::from_str(&filters)?
	} else {
		None
	};

	let number_of_courses = course_query_repository.count_created(&ctx, user_id, filters).await?;

	let res = CoursesPayload {
		courses: courses_res, 
		count: number_of_courses 
	};

	Ok(Json(res))
}

#[utoipa::path(
	get,
	path = "/api/course/get_attendants",
	params(
		GetAttendatsPayload,	
	),
	responses(
		(status = 200, body = UsersPayload),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_get_attendants(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Query(payload): Query<GetAttendatsPayload>,
) -> AppResult<Json<UsersPayload>> {
	let ctx = ctx.0;
	let course_id = payload.course_id;

	app_state.permission_manager.check_course_creator_permission(&ctx, course_id).await?;

	let list_options = if let Some(list_options) = payload.list_options {
		serde_json::from_str(&list_options)?
	} else {
		None
	};

	let user_query_repo = app_state.query_repository_manager.get_user_repository();
	let users = user_query_repo.get_attendants(&ctx, course_id, list_options).await?;
	let count = user_query_repo.get_number_of_attendance(course_id).await?;


	let users_result = users
		.iter()
		.map(|user| UserPayload { 
			id: user.id,
			username: user.username.clone(), 
			number_of_completed_lessons: user.number_of_completed_lessons,
			date_registered: user.date_registered,
		})
		.collect();

	let result = UsersPayload { users: users_result, count  };

	Ok(Json(result))
}


#[utoipa::path(
	get,
	path = "/api/course/get_point_statistics/{course_id}",
	params(
		("course_id", description = "ID of the course")
	),
	responses(
		(status = 200, body = CoursePointStatisticsPayload),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_get_point_statistics_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Path(course_id): Path<i64>,
) -> AppResult<Json<CoursePointStatisticsPayload>> {
	let ctx = ctx.0;

	let exercise_repository = app_state.query_repository_manager.get_exercise_repository();

	let course_point_statistics = exercise_repository.get_course_point_statistics(&ctx, course_id).await?;

	Ok(Json(course_point_statistics.into()))
}