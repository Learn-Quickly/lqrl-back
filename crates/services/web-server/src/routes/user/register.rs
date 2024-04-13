use axum::{extract::State, routing::post, Json, Router};
use lib_core::{core::user::UserInteractor, ctx::Ctx};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;

use crate::{app_state::AppState, error::AppResult};

pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/api/register", post(api_register_handler))
		.with_state(app_state)
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
	State(app_state): State<AppState>,
	Json(payload): Json<RegisterPayload>,
) -> AppResult<Json<Value>> {
    let ctx = Ctx::root_ctx();

	let command_repository_manager = app_state.command_repository_manager;
	let user_interactor = UserInteractor::new(command_repository_manager.as_ref());
    user_interactor.create_user(&ctx, payload.pwd, payload.username).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}