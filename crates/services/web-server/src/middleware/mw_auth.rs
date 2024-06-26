use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum_auth::AuthBearer;
use derive_more::Display;
use lib_auth::token::{validate_web_token, Token};
use lib_core::ctx::Ctx;
use lib_db::query_repository::user::UserData;
use serde::Serialize;
use tracing::debug;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};

pub async fn mw_ctx_require(
	ctx: AppResult<CtxW>,
	req: Request<Body>,
	next: Next,
) -> AppResult<Response> {
	debug!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");

	ctx?;

	Ok(next.run(req).await)
}

pub async fn mw_ctx_resolver(
	State(app_state): State<AppState>,
	AuthBearer(token): AuthBearer,
	mut req: Request<Body>,
	next: Next,
) -> Response {
	debug!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");

	let ctx_ext_result = ctx_resolve(app_state, &token).await;

	// Store the ctx_ext_result in the request extension
	// (for Ctx extractor).
	req.extensions_mut().insert(ctx_ext_result);

	next.run(req).await
}

async fn ctx_resolve(app_state: AppState, token: &str) -> CtxExtResult {
	// -- Parse Token
	let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

	// -- Get UserForAuth
	let user_query_repository = app_state.query_repository_manager.get_user_repository();
	let user: UserData =
		user_query_repository.first_by_username(&Ctx::root_ctx(), &token.ident)
			.await
			.map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
			.ok_or(CtxExtError::UserNotFound)?;

	// -- Validate Token
	validate_web_token(&token, user.token_salt)
		.map_err(|_| CtxExtError::FailValidate)?;

	// -- Create CtxExtResult
	Ctx::new(user.id)
		.map(CtxW)
		.map_err(|ex| CtxExtError::CtxCreateFail(ex.to_string()))
}

// region:    --- Ctx Extractor
#[derive(Debug, Clone)]
pub struct CtxW(pub Ctx);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for CtxW {
	type Rejection = AppError;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> AppResult<Self> {
		debug!("{:<12} - Ctx", "EXTRACTOR");

		parts
			.extensions
			.get::<CtxExtResult>()
			.ok_or(AppError::CtxExt(CtxExtError::CtxNotInRequestExt))?
			.clone()
			.map_err(AppError::CtxExt)
	}
}
// endregion: --- Ctx Extractor

// region:    --- Ctx Extractor Result/Error
type CtxExtResult = core::result::Result<CtxW, CtxExtError>;

#[derive(Clone, Serialize, Debug, Display)]
pub enum CtxExtError {
	TokenWrongFormat,

	UserNotFound,
	ModelAccessError(String),
	FailValidate,

	CtxNotInRequestExt,
	CtxCreateFail(String),
}

impl std::error::Error for CtxExtError {}

// endregion: --- Ctx Extractor Result/Error
