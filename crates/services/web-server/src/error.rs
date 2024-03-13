use crate::web;
use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::From;
use lib_auth::pwd::PwdError;
use lib_auth::token::TokenError;
use lib_auth::{pwd, token};
use lib_core::core::error::CoreError;
use lib_db::store::error::DbError;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::error::Error;
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
	CtxExt(web::mw_auth::CtxExtError),

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

	// -- File
	CreateFileFail,
	RemoveFileFail,

	GenericError,
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		debug!("{:<12} - model::Error {self:?}", "INTO_RES");

		let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

		response.extensions_mut().insert(Arc::new(self));

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

macro_rules! impl_from_box {
    ($from:ty, $to:ident) => {
        impl From<Box<$from>> for AppError {
            fn from(value: Box<$from>) -> Self {
                AppError::$to(*value)
            }
        }
    };
}

impl_from_box!(CoreError, Core);
impl_from_box!(DbError, Db);
impl_from_box!(PwdError, Pwd);
impl_from_box!(TokenError, Token);
impl_from_box!(serde_json::Error, SerdeJson);
impl_from_box!(web::mw_auth::CtxExtError, CtxExt);
impl_from_box!(MultipartError, Multipart);

macro_rules! try_downcast_error {
    ($error:expr, $($type:ty),*) => {
        $( if let Ok(err) = $error.downcast::<$type>() {
            return err.into();
        } )*
    };
}

impl From<Box<dyn Error>> for AppError {
	fn from(error: Box<dyn Error>) -> Self {
		try_downcast_error!(
			error,
			CoreError,
			DbError,
			PwdError,
			TokenError,
			serde_json::Error,
			web::mw_auth::CtxExtError,
			MultipartError
		);

		AppError::GenericError
	}
}

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
			Db(DbError::EntityNotFound { entity, id }) => (
				StatusCode::BAD_REQUEST,
				ClientError::ENTITY_NOT_FOUND { entity, id: *id },
			),
			Db(DbError::UserCourseNotFound{ entity, course_id, user_id}) => (
				StatusCode::BAD_REQUEST,
				ClientError::USER_COURSE_NOT_FOUND { 
					entity, 
					course_id: *course_id, 
					user_id: *user_id 
				},
			),

			// -- Fallback.
			_ => (
				StatusCode::INTERNAL_SERVER_ERROR,
				ClientError::SERVICE_ERROR {
					description: self.to_string()
				},
			),
		}
	}
}

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
	LOGIN_FAIL,
	NO_AUTH,
	ENTITY_NOT_FOUND { entity: &'static str, id: i64 },
	USER_COURSE_NOT_FOUND { 
		entity: &'static str, 
		course_id: i64, 
		user_id: i64 
	},

	SERVICE_ERROR { description: String },
}
