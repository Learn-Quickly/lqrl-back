use crate::web::{Error, Result};
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum_auth::AuthBearer;
use lib_auth::token::{validate_web_token, Token};
use lib_core::ctx::Ctx;
use lib_db::model::user::{UserBmc, UserForAuth};
use lib_db::model::ModelManager;
use serde::Serialize;
use tracing::debug;

pub async fn mw_ctx_require(
	ctx: Result<CtxW>,
	req: Request<Body>,
	next: Next,
) -> Result<Response> {
	debug!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");

	ctx?;

	Ok(next.run(req).await)
}

pub async fn mw_ctx_resolver(
	State(mm): State<ModelManager>,
	AuthBearer(token): AuthBearer,
	mut req: Request<Body>,
	next: Next,
) -> Response {
	debug!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");

	let ctx_ext_result = ctx_resolve(mm, &token).await;

	// Store the ctx_ext_result in the request extension
	// (for Ctx extractor).
	req.extensions_mut().insert(ctx_ext_result);

	next.run(req).await
}

async fn ctx_resolve(mm: ModelManager, token: &str) -> CtxExtResult {
	// -- Parse Token
	let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

	// -- Get UserForAuth
	let user: UserForAuth =
		UserBmc::first_by_username(&Ctx::root_ctx(), &mm, &token.ident)
			.await
			.map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
			.ok_or(CtxExtError::UserNotFound)?;

	// -- Validate Token
	validate_web_token(&token, user.token_salt)
		.map_err(|_| CtxExtError::FailValidate)?;

	// // -- Update Token
	// set_token_cookie(cookies, &user.username, user.token_salt)
	// 	.map_err(|_| CtxExtError::CannotSetTokenCookie)?;

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
	type Rejection = Error;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
		debug!("{:<12} - Ctx", "EXTRACTOR");

		parts
			.extensions
			.get::<CtxExtResult>()
			.ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
			.clone()
			.map_err(Error::CtxExt)
	}
}
// endregion: --- Ctx Extractor

// region:    --- Ctx Extractor Result/Error
type CtxExtResult = core::result::Result<CtxW, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
	TokenWrongFormat,

	UserNotFound,
	ModelAccessError(String),
	FailValidate,

	CtxNotInRequestExt,
	CtxCreateFail(String),
}
// endregion: --- Ctx Extractor Result/Error
