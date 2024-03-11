use axum::{extract::State, routing::post, Json, Router};
use lib_core::ctx::Ctx;
use lib_db::repository::{user::{UserBmc, UserForCreate}, DbManager};
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

    let user_c = UserForCreate {
        username: payload.username,
        pwd_clear: payload.pwd,
    };
    UserBmc::create(&ctx, &dbm, user_c).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}