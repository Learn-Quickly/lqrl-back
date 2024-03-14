use derive_more::From;
use lib_auth::pwd;
use lib_core::core::error::CoreError;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::dbx::error::DbxError;

pub type DbResult<T> = core::result::Result<T, DbError>;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, From)]
pub enum DbError {
	EntityNotFound {
		entity: String,
		id: i64,
	},
	UserCourseNotFound {
		entity: String,
		user_id: i64,
		course_id: i64,
	},
	ListLimitOverMax {
		max: i64,
		actual: i64,
	},

	// -- DB
	UserAlreadyExists {
		username: String,
	},
	CourseAlreadyExists {
		title: String,
	},
	UniqueViolation {
		table: String,
		constraint: String,
	},
	CourseStateMustBePublished {
		course_id: i64,
	},
	MissingFieldError {
		entity: String,
		field: String,
	},

	// -- DbManager
	CantCreateDbManagerProvider(String),

	// -- Modules
	#[from]
	Pwd(pwd::PwdError),

	Dbx(String),
}

impl core::fmt::Display for DbError {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for DbError {}

impl From<DbError> for CoreError {
	fn from(db_error: DbError) -> Self {
		let error_value = serde_json::to_value(db_error);
		match error_value {
			Ok(value) => CoreError::DbError(value),
			Err(err) => CoreError::SerdeJson(err),
		}
	}
}

impl From<DbxError> for DbError {
	fn from(dbx_error: DbxError) -> Self {
		match dbx_error {
			DbxError::DbError(db_error) => db_error,
			_ => Self::Dbx(dbx_error.to_string())
		}
	}
}

impl DbError {
	pub fn handle_option_field<T>(value: Option<T>, entity: &String, field: String) -> DbResult<T> {
		let result = value.unwrap_or(
			Err(DbError::MissingFieldError { 
				entity: entity.clone(),
				field,
			})?
		);

		Ok(result)
	}
}