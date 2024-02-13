use std::collections::HashMap;

use axum::{extract::{multipart, Multipart, Path}, routing::post, Json, Router};
use lib_core::model::ModelManager;
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::{IntoParams, ToSchema};

use crate::web::Result;

use super::file_upload::upload_file;

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/set_course_img/:i64", post(api_set_course_img_handler))
		.with_state(mm)
}

#[derive(ToSchema)]
struct ImgFile {
	img: Vec<u8>
}

#[derive(Deserialize, IntoParams)]
struct CourseId {
	course_id: i64
}

#[utoipa::path(
	post,
	path = "/api/set_course_img",
	params(CourseId),
	request_body(content_type = "multipart/formdata", content = Vec<u8>),
	responses(
		(status = 200, description = "Course draft created successfully"),
	)
)]
async fn api_set_course_img_handler(
	Path(course_id): Path<CourseId>,
	mut multipart: Multipart,
) -> Result<Json<Value>> {
	let course_id = course_id.course_id;

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