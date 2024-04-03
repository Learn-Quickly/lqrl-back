use axum::{extract::State, routing::post, Json, Router};
use lib_core::{core::user::UserController, ctx::Ctx};
use lib_db::{command_repository::user::UserCommandRepository, store::DbManager};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;

use crate::error::AppResult;

pub fn routes(dbm: DbManager) -> Router {
	Router::new()
		.route("/api/register", post(api_register_handler))
		.with_state(dbm)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterPayload {
	#[schema(example = "username_test")]
	username: String,
	#[schema(example = "test_pwd")]
	pwd: String,
}

#[utoipa::path(
	post,
	path = "/api/register",
	request_body = RegisterPayload,
	responses(
		(status = 200, description = "Register successful"),
	)
)]
async fn api_register_handler(
	State(dbm): State<DbManager>,
	Json(payload): Json<RegisterPayload>,
) -> AppResult<Json<Value>> {
    let ctx = Ctx::root_ctx();

	let repository = UserCommandRepository::new(dbm);
	let user_controller = UserController::new(&ctx, &repository);
    user_controller.create_user(payload.pwd, payload.username).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}