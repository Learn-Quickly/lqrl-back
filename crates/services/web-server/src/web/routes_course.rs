use axum::{extract::Multipart, routing::post, Json, Router};
use lib_core::model::ModelManager;
use serde_json::{json, Value};

use crate::web::Result;

use super::file_upload::upload_file;

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/create_course", post(api_create_course_handler))
		.with_state(mm)
}

async fn api_create_course_handler(
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