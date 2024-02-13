use crate::web::Result;
use axum::{extract::State, routing::post, Json, Router};
use lib_core::{ctx::Ctx, model::{user::{UserBmc, UserForCreate}, ModelManager}};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/api/register", post(api_register_handler))
		.with_state(mm)
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
	State(mm): State<ModelManager>,
	Json(payload): Json<RegisterPayload>,
) -> Result<Json<Value>> {
    let ctx = Ctx::root_ctx();

    let user_c = UserForCreate {
        username: payload.username,
        pwd_clear: payload.pwd,
    };
    UserBmc::create(&ctx, &mm, user_c).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}