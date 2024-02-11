use crate::web::Result;
use axum::{extract::Multipart, routing::post, Router};
use lib_core::model::ModelManager;

use super::mw_auth::CtxW;

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/create_course", post(api_create_course_handler))
		.with_state(mm)
}

async fn api_create_course_handler(
	ctx: CtxW,
	mut multipart: Multipart,
) -> Result<()> {
	while let Some(field) = multipart.next_field().await.unwrap() {
		
	}

	Ok(())
}