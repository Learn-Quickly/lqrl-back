use crate::web;
use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::From;
use lib_auth::{pwd, token};
use lib_core::core::error::CoreError;
use lib_db::store::error::DbError;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::sync::Arc;
use tracing::debug;

pub type AppResult<T> = core::result::Result<T, AppError>;

#[serde_as]
#[derive(Debug, Serialize, From, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum AppError {
	// -- Login
	LoginFailUsernameNotFound,
	LoginFailUserHasNoPwd {
		user_id: i64,
	},
	LoginFailPwdNotMatching {
		user_id: i64,
	},

	// -- Extractors
	ReqStampNotInReqExt,

	// -- CtxExtError
	#[from]
	CtxExt(web::middleware::mw_auth::CtxExtError),

	// -- Modules

	#[from]
	Core(CoreError),
	#[from]
	Db(DbError),
	#[from]
	Pwd(pwd::PwdError),
	#[from]
	Token(token::TokenError),

	// -- External Modules
	#[from]
	SerdeJson(#[serde_as(as = "DisplayFromStr")] serde_json::Error),

	#[from]
	Multipart(#[serde_as(as = "DisplayFromStr")] MultipartError),
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		debug!("{:<12} - model::Error {self:?}", "INTO_RES");

		let (status_code, client_error) = self.client_status_and_error();
		let mut response = status_code.into_response();

		response.extensions_mut().insert(Arc::new(client_error));

		response
	}
}

impl core::fmt::Display for AppError {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for AppError {}

impl AppError {
	pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
		use AppError::*;

		match self {
			// -- Login
			LoginFailUsernameNotFound
			| LoginFailUserHasNoPwd { .. }
			| LoginFailPwdNotMatching { .. } => {
				(StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL)
			}

			// -- Auth
			CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

			// -- Db
			Db(db_error) => map_db_error(db_error),
			Core(CoreError::DbError(value)) => {
				let db_error: Result<DbError, serde_json::Error> = serde_json::from_value(value.clone());
				match db_error {
					Ok(db_error) => map_db_error(&db_error),
					Err(json_error) => internal_server_error(json_error.to_string()),
				}
			},
			// -- Fallback.
			_ => internal_server_error(self.to_string()),
		}
	}
}

fn map_db_error(db_error: &DbError) -> (StatusCode, ClientError) {
	match db_error {
		DbError::EntityNotFound { entity, id } => (
			StatusCode::BAD_REQUEST,
			ClientError::ENTITY_NOT_FOUND { entity: entity.to_string(), id: *id },
		),
		DbError::UserCourseNotFound { entity, user_id, course_id } => (
			StatusCode::BAD_REQUEST,
			ClientError::USER_COURSE_NOT_FOUND { 
				entity: entity.to_string(), 
				course_id: *course_id, 
				user_id: *user_id 
			},
		),
		DbError::ListLimitOverMax { .. } |
		DbError::UserAlreadyExists { .. } |
		DbError::CourseAlreadyExists { .. } |
		DbError::UniqueViolation { .. } |
		DbError::CourseStateMustBePublished { .. } | 
		DbError::MissingFieldError { .. } => bad_request(db_error.to_string()),
		_ => internal_server_error(db_error.to_string()),
	}
}

fn bad_request(description: String) -> (StatusCode, ClientError) {
	(
		StatusCode::BAD_REQUEST,
		ClientError::BAD_REQUEST {
			description,
		},
	)
}

fn internal_server_error(description: String) -> (StatusCode, ClientError) {
	(
		StatusCode::INTERNAL_SERVER_ERROR,
		ClientError::SERVICE_ERROR {
			description,
		},
	)
}

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
	LOGIN_FAIL,
	NO_AUTH,
	ENTITY_NOT_FOUND { entity: String, id: i64 },
	USER_COURSE_NOT_FOUND { 
		entity: String, 
		course_id: i64, 
		user_id: i64 
	},

	BAD_REQUEST { description: String },

	SERVICE_ERROR { description: String },
}
