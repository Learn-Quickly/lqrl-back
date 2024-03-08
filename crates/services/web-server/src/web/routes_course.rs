use axum::{extract::{Multipart, Path, State}, routing::{get, post}, Json, Router};
use lib_core::model::{course::{Course, CourseBmc, CourseForCreate, CourseState}, ModelManager};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_with::serde_as;
use time::OffsetDateTime;
use utoipa::ToSchema;
use lib_utils::time::Rfc3339;

use crate::web::Result;

use super::{file_upload::upload_file, mw_auth::CtxW};

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/set_course_img/:i64", post(api_set_course_img_handler))
		.route("/create_course_draft", post(api_create_course_draft))
		.route("/get_course/:i64", get(api_get_course))
		.route("/get_courses/", post(api_get_courses))
		.with_state(mm)
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
	path = "/api/create_course_draft",
	request_body = CourseCreateDraftPayload,
	responses(
		(status = 200, description = "Course draft created successfully", body = CreatedCourseDraft),
		(status = 500, description = "Course already exists. The title must be unique"),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_create_course_draft(
	ctx: CtxW,
	State(mm): State<ModelManager>,
	Json(paylod): Json<CourseCreateDraftPayload>
) -> Result<Json<CreatedCourseDraft>> {
	let ctx = ctx.0;

	let course_c = CourseForCreate {
    	title: paylod.title,
    	description: paylod.description,
    	course_type: paylod.course_type,
    	price: paylod.price,
    	color: paylod.color,
	};

	let course_id = CourseBmc::create_draft(&ctx, &mm, course_c).await?;

	let created_course_draft = CreatedCourseDraft {
    	course_id,
	};

	let body = Json(created_course_draft);
	
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
	published_date: Option<OffsetDateTime>,
	img_url: Option<String>,
	state: CourseState,
}

impl From<Course> for CoursePayload {
	fn from(value: Course) -> Self {
		Self {
    		id: value.id,
    		title: value.title,
    		description: value.description,
    		course_type: value.course_type,
    		price: value.price,
    		color: value.color,
    		published_date: value.published_date,
    		img_url: value.img_url,
    		state: value.state,
		}
	}
}

#[utoipa::path(
	get,
	path = "/api/get_course/{course_id}",
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
async fn api_get_course(
	ctx: CtxW,
	State(mm): State<ModelManager>,
	Path(course_id): Path<i64>,
) -> Result<Json<CoursePayload>> {
	let ctx = ctx.0;

	let course: Course = CourseBmc::get(&ctx, &mm, course_id).await?;

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
	path = "/api/get_courses/",
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
async fn api_get_courses(
	ctx: CtxW,
	State(mm): State<ModelManager>,
	Json(filter_payload): Json<CourseFilterPayload>,
) -> Result<Json<Vec<CoursePayload>>> {
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

	let courses: Vec<Course> = CourseBmc::list(&ctx, &mm, filters, list_options).await?;
	let body: Vec<CoursePayload> = courses.iter().map(|course| course.clone().into()).collect();  

	Ok(Json(body))
}

#[utoipa::path(
	post,
	path = "/api/set_course_img/{course_id}",
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
	Path(course_id): Path<i64>,
	mut multipart: Multipart,
) -> Result<Json<Value>> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap().to_string();
		
        if field_name == "image" {
            let data = field.bytes().await.unwrap();

			let img_url = upload_file(data).await.unwrap();

			let body = Json(json!({
				"result": {
					"img_url": img_url,
				}
			}));

			return Ok(body);
        } else {
            let data = field.text().await.unwrap();
            println!("field: {}, value: {}", field_name, data);
        }
    }

	let body = Json(json!({
		"result": {
			"error": "file error"
		}
	}));

    Ok(body)
}