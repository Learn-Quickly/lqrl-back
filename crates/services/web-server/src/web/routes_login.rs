use crate::web::{self, remove_token_cookie, Error, Result};
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use lib_auth::pwd::{self, ContentToHash, SchemeStatus};
use lib_core::ctx::Ctx;
use lib_core::model::user::{UserBmc, UserForLogin};
use lib_core::model::ModelManager;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::Cookies;
use tracing::debug;
use utoipa::ToSchema;

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/api/login", post(api_login_handler))
		.route("/api/logoff", post(api_logoff_handler))
		.with_state(mm)
}

// region:    --- Login
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginPayload {
	#[schema(example = "username_test")]
	username: String,
	#[schema(example = "test_pwd")]
	pwd: String,
}

#[utoipa::path(
	post,
	path = "/api/login",
	request_body = LoginPayload,
	responses(
		(status = 200, description = "Login successful"),
		(status = 403, description = "Login failed")
	)
)]
async fn api_login_handler(
	State(mm): State<ModelManager>,
	cookies: Cookies,
	Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_login_handler", "HANDLER");

	let LoginPayload {
		username,
		pwd: pwd_clear,
	} = payload;
	let root_ctx = Ctx::root_ctx();

	// -- Get the user.
	let user: UserForLogin = UserBmc::first_by_username(&root_ctx, &mm, &username)
		.await?
		.ok_or(Error::LoginFailUsernameNotFound)?;
	let user_id = user.id;

	// -- Validate the password.
	let Some(pwd) = user.pwd else {
		return Err(Error::LoginFailUserHasNoPwd { user_id });
	};

	let scheme_status = pwd::validate_pwd(
		ContentToHash {
			salt: user.pwd_salt,
			content: pwd_clear.clone(),
		},
		pwd,
	)
	.await
	.map_err(|_| Error::LoginFailPwdNotMatching { user_id })?;

	// -- Update password scheme if needed
	if let SchemeStatus::Outdated = scheme_status {
		debug!("pwd encrypt scheme outdated, upgrading.");
		UserBmc::update_pwd(&root_ctx, &mm, user.id, &pwd_clear).await?;
	}

	// -- Set web token.
	web::set_token_cookie(&cookies, &user.username, user.token_salt)?;

	// Create the success body.
	let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)
}
// endregion: --- Login

// region:    --- Logoff
#[utoipa::path(
	post,
	path = "/api/logoff",
	responses(
		(status = 200, description = "Logout successful"),
	),
	security(
		("auth_token" = [])
	)
)]
async fn api_logoff_handler(
	cookies: Cookies,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_logoff_handler", "HANDLER");

	remove_token_cookie(&cookies)?;

	// Create the success body.
	let body = Json(json!({
		"result": {
			"logged_off": "successful",
		}
	}));

	Ok(body)
}
// endregion: --- Logoff
