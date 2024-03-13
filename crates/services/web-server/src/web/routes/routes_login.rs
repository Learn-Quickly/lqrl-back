use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use axum_auth::AuthBasic;
use lib_auth::pwd::{self, ContentToHash, SchemeStatus};
use lib_auth::token::{generate_web_token, validate_web_token, Token};
use lib_core::ctx::Ctx;
use lib_db::command_repository::user::{UserBmc, UserForAuth, UserForLogin};
use lib_db::store::DbManager;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::debug;
use utoipa::ToSchema;

use crate::error::{AppError, AppResult};
use crate::web::middleware::mw_auth::CtxExtError;

pub fn routes(dbm: DbManager) -> Router {
	Router::new()
		.route("/api/login", post(api_login_handler))
		.route("/api/refresh_token", post(api_refresh_access_token_handler))
		.with_state(dbm)
}

// region:    --- Login
#[utoipa::path(
	post,
	path = "/api/login",
	responses(
		(status = 200, description = "Login successful"),
		(status = 403, description = "Login failed")
	),
	security(
		("basicAuth" = [])
	)
)]
async fn api_login_handler(
	State(dbm): State<DbManager>,
	AuthBasic((username, pwd_clear)): AuthBasic,
) -> AppResult<Json<Value>> {
	debug!("{:<12} - api_login_handler", "HANDLER");

	let root_ctx = Ctx::root_ctx();

	// -- Get the user.
	let user: UserForLogin = UserBmc::first_by_username(&root_ctx, &dbm, &username)
		.await?
		.ok_or(AppError::LoginFailUsernameNotFound)?;

	let user_id = user.id;

	let pwd_clear = match pwd_clear {
    	Some(pwd) => pwd,
    	None => return Err(AppError::LoginFailUserHasNoPwd { user_id }),
	};

	// -- Validate the password.
	let Some(pwd) = user.pwd else {
		return Err(AppError::LoginFailUserHasNoPwd { user_id });
	};

	let scheme_status = pwd::validate_pwd(
		ContentToHash {
			salt: user.pwd_salt,
			content: pwd_clear.clone(),
		},
		pwd,
	)
	.await
	.map_err(|_| AppError::LoginFailPwdNotMatching { user_id })?;

	// -- Update password scheme if needed
	if let SchemeStatus::Outdated = scheme_status {
		debug!("pwd encrypt scheme outdated, upgrading.");
		UserBmc::update_pwd(&root_ctx, &dbm, user.id, &pwd_clear).await?;
	}

	let access_token = generate_web_token(&user.username, user.token_salt, lib_auth::token::TokenType::Access)?;
	let refresh_token = generate_web_token(&user.username, user.token_salt, lib_auth::token::TokenType::Refresh)?;

	// Create the success body.
	let body = Json(json!({
		"result": {
			"access_token": access_token.to_string(),
			"refresh_token": refresh_token.to_string()
		}
	}));

	Ok(body)
}
// endregion: --- Login

// region:    --- Refresh token

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenPayload {
	#[schema(example = "dGVzdHVzZXI.MjAyNC0wOC0xN1QwNDo0ODowOS4xNjq3XjU0NTha.MjcedibXB_UadS2vIG2lPfwlukqw5Ir-DIO_zwwmn9dQqd0oeozAi3Aa99f4UlC8ETrJjRiZNMHjyIsyEaqgDA")]
	refresh_token: String,
}

#[utoipa::path(
	post,
	path = "/api/refresh_token",
	request_body = RefreshTokenPayload,
	responses(
		(status = 200, description = "Token refreshed successfully"),
		(status = 403, description = "Token refresh failed")
	)
)]
async fn api_refresh_access_token_handler(
	State(dbm): State<DbManager>,
	Json(payload): Json<RefreshTokenPayload>
) -> AppResult<Json<Value>> {
	let refresh_token: Token = payload.refresh_token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

	// -- Get UserForAuth
	let user: UserForAuth =
		UserBmc::first_by_username(&Ctx::root_ctx(), &dbm, &refresh_token.ident)
			.await
			.map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
			.ok_or(CtxExtError::UserNotFound)?;

	// -- Validate Token
	validate_web_token(&refresh_token, user.token_salt)
		.map_err(|_| CtxExtError::FailValidate)?;

	let access_token = generate_web_token(&user.username, user.token_salt, lib_auth::token::TokenType::Access)?;
	let refresh_token = generate_web_token(&user.username, user.token_salt, lib_auth::token::TokenType::Refresh)?;

	let body = Json(json!({
		"result": {
			"access_token": access_token.to_string(),
			"refresh_token": refresh_token.to_string()
		}
	}));

	Ok(body)
}
// endregion  --- Refresh token