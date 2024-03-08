use axum::{extract::{Multipart, Path, State}, routing::post, Json, Router};
use lib_core::model::{course::{CourseBmc, CourseForCreate}, ModelManager};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;

use crate::web::Result;

use super::{file_upload::upload_file, mw_auth::CtxW};

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/set_course_img/:i64", post(api_set_course_img_handler))
		.route("/create_course_draft", post(api_create_course_draft))
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
) -> Result<Json<String>> {
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

	let body = Json(serde_json::to_string(&created_course_draft)?);
	
	Ok(body)
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
            println!("field: {}      value: {}",field_name,data);
        }
    }

	let body = Json(json!({
		"result": {
			"error": "file error"
		}
	}));

    Ok(body)
}