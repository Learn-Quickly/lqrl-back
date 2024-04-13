use axum::{debug_handler, extract::{Multipart, Path, State}, routing::{get, post, put}, Json, Router};
use lib_core::{interactors::creator::course::CreatorCourseInteractor, model::course::{CourseForCreate, CourseForUpdate}};
use lib_db::query_repository::course::CourseQuery;
use serde_json::{json, Value};

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW, routes::models::course::{CourseCreateDraftPayload, CourseId, CoursePayload, CourseUpdatePayload, CreatedCourseDraft}};

pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/create_course_draft", post(api_create_course_draft_handler))
		.route("/update", put(api_update_course_handler))
		.route("/set_course_img/:i64", put(api_set_course_img_handler))
		.route("/publish_course", put(api_publish_course_handler))
		.route("/archive_course", put(api_archive_course_handler))
		.route("/get_created_courses", get(api_get_created_courses_handler))
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
#[debug_handler]
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
			let img_url = course_interactor.set_course_img(&ctx, course_id, &data, "/public/uploads").await?;

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
	path = "/api/course/get_created_courses/",
	responses(
		(status = 200, body=Vec<CoursePayload>),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_get_created_courses_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
) -> AppResult<Json<Vec<CoursePayload>>> {
	let ctx = ctx.0;
	let user_id = ctx.user_id();

	let course_query_repository = app_state.query_repository_manager.get_course_repository();
	let courses: Vec<CourseQuery> = course_query_repository.get_user_courses_created(&ctx, user_id).await?;
	let body: Vec<CoursePayload> = courses.iter().map(|course| course.clone().into()).collect();  

	Ok(Json(body))
}
