use axum::{debug_handler, extract::{Multipart, Path, State}, routing::{get, post, put}, Json, Router};
use lib_core::{core::course::CourseInteractor, model::course::{CourseForCreate, CourseForUpdate}};
use lib_db::query_repository::course::{CourseQuery, CourseStateQuery};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_with::serde_as;
use utoipa::ToSchema;

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW};

pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/create_course_draft", post(api_create_course_draft_handler))
		.route("/update", put(api_update_course_handler))
		.route("/set_course_img/:i64", put(api_set_course_img_handler))
		.route("/publish_course", put(api_publish_course_handler))
		.route("/archive_course", put(api_archive_course_handler))
		.route("/register_for_course", put(api_register_for_course_handler))
		.route("/unsubscribe_from_course", put(api_unsubscribe_from_course_handler))
		.route("/get_course/:i64", get(api_get_course_handler))
		.route("/get_courses/", post(api_get_courses_handler))
		.route("/get_user_courses_registered/", get(api_get_user_courses_registered_handler))
		.route("/get_user_courses_created/", get(api_get_user_courses_created_handler))
		.with_state(app_state)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CourseCreateDraftPayload {
	title: String,
	description: String,
	course_type: String,
	price: f64,
	color: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreatedCourseDraft {
	course_id: i64,
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
	let course_interactor = CourseInteractor::new(command_repository_manager);

	let course_id = course_interactor.create_draft(&ctx, course_c).await?;

	let created_course_draft = CreatedCourseDraft {
    	course_id,
	};

	let body = Json(created_course_draft);
	
	Ok(body)
}

#[derive(Deserialize, ToSchema)]
pub struct CourseUpdatePayload {
	pub id: i64,
	pub title: Option<String>,
	pub description: Option<String>,
	pub course_type: Option<String>,
	pub price: Option<f64>,
	pub color: Option<String>,
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
	let course_interactor = CourseInteractor::new(command_repository_manager);

	course_interactor.update_course(&ctx, course_for_u, payload.id).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}

#[derive(ToSchema, Deserialize)]
pub struct CourseId {
	course_id: i64,
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
	let course_interactor = CourseInteractor::new(command_repository_manager);

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
	let course_interactor = CourseInteractor::new(command_repository_manager);

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
	let course_interactor = CourseInteractor::new(command_repository_manager);

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
	let course_interactor = CourseInteractor::new(command_repository_manager);

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
	let course_interactor = CourseInteractor::new(command_repository_manager);

	course_interactor.unsubscribe_from_course(&ctx, course_id).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}

#[serde_as]
#[derive(Serialize, ToSchema)]
pub struct CoursePayload {
	id: i64,
	title: String,
	description: String,
	course_type: String,
	price: f64,
	color: String,
	published_date: Option<i64>,
	img_url: Option<String>,
	state: CourseStatePayload,
}

#[serde_as]
#[derive(Serialize, ToSchema)]
pub enum CourseStatePayload {
    Draft,
    Published,
	Archived,
    None,
}

impl From<CourseStateQuery> for CourseStatePayload {
	fn from(value: CourseStateQuery) -> Self {
		match value {
			CourseStateQuery::Draft => Self::Draft,
			CourseStateQuery::Published => Self::Published,
			CourseStateQuery::Archived => Self::Archived,
			CourseStateQuery::None => Self::None,
		}
	}
}

impl From<CourseQuery> for CoursePayload {
	fn from(value: CourseQuery) -> Self {
		let published_date = value.published_date.and_then(|date| Some(date.unix_timestamp()));

		Self {
    		id: value.id,
    		title: value.title,
    		description: value.description,
    		course_type: value.course_type,
    		price: value.price,
    		color: value.color,
    		published_date,
    		img_url: value.img_url,
    		state: value.state.into(),
		}
	}
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

	Ok(Json(course.into()))
}

#[derive(Deserialize, ToSchema)]
pub struct CourseFilterPayload {
	#[schema(example = r#"{"price": {"$gte": 1000}}""#)]
	filters: Option<Value>,
	#[schema(example = r#"{"limit": 2, "offset": 0, "order_bys": "!title"}""#)]
	list_options: Option<Value>,
}

#[utoipa::path(
	post,
	path = "/api/course/get_courses/",
	request_body = CourseFilterPayload,
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
	Json(filter_payload): Json<CourseFilterPayload>,
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
	let body: Vec<CoursePayload> = courses.iter().map(|course| course.clone().into()).collect();  

	Ok(Json(body))
}

#[utoipa::path(
	get,
	path = "/api/course/get_user_courses_created/",
	responses(
		(status = 200, body=Vec<CoursePayload>),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_get_user_courses_created_handler(
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

#[utoipa::path(
	get,
	path = "/api/course/get_user_courses_registered/",
	responses(
		(status = 200, body=Vec<CoursePayload>),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_get_user_courses_registered_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
) -> AppResult<Json<Vec<CoursePayload>>> {
	let ctx = ctx.0;
	let user_id = ctx.user_id();

	let course_query_repository = app_state.query_repository_manager.get_course_repository();
	let courses: Vec<CourseQuery> = course_query_repository.get_user_courses_registered(&ctx, user_id).await?;
	let body: Vec<CoursePayload> = courses.iter().map(|course| course.clone().into()).collect();  

	Ok(Json(body))
}