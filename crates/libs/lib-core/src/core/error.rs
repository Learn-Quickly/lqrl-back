use derive_more::From;
use serde::Serialize;
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::Debug;


#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum CoreError {
	PermissionDenied,

	CourseError(CourseError),
	UserError(UserError),

	// File error
	#[from]
	IOError(#[serde_as(as = "DisplayFromStr")] std::io::Error),

	#[from]
	SerdeJson(#[serde_as(as = "DisplayFromStr")] serde_json::Error),

	#[from]
	DbError(Value),
}

impl core::fmt::Display for CoreError {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for CoreError {}

#[derive(Debug, Serialize)]
pub enum CourseError {
	// Course error
    CourseMustBePublishedError,
	CreatorCannotSubscribeToTheCourse,
	CannotRegisterForCourseTwice,
}

impl From<CourseError> for CoreError {
	fn from(course_error: CourseError) -> Self {
		CoreError::CourseError(course_error)
	}
}

#[derive(Debug, Serialize)]
pub enum UserError {
	WrongPasswordError { user_id: i64 },
	UserHasNoPwd { user_id: i64 },
}

impl From<UserError> for CoreError {
	fn from(user_error: UserError) -> Self {
		CoreError::UserError(user_error)
	}
}