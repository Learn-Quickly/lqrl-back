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
	pub id: i64,
	pub title: String,
	pub description: String,
	pub course_type: String,
	pub price: f64,
	pub color: String,
	#[serde_as(as = "Option<Rfc3339>")]
	pub published_date: Option<OffsetDateTime>,
	pub img_url: Option<String>,
	pub state: CourseState,
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