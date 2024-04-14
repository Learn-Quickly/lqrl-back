use derive_more::From;
use lib_auth::pwd::PwdError;
use serde::Serialize;
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::Debug;


#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum CoreError {
	PermissionDenied,

	#[from]
	CourseError(CourseError),
	#[from]
	LessonError(LessonError),
	#[from]
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
	CourseStateDoesNotExist { state: String },
}

#[derive(Debug, Serialize)]
pub enum LessonError {
	IncorrectLessonOreder { lesson_id: i64, order: i32 },
	LessonNotFound { lesson_id: i64 },
}

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum UserError {
	WrongPasswordError { user_id: i64 },
	UserHasNoPwd { user_id: i64 },
	RoleDoesNotExist { role: String },
	
	#[from]
	PwdError(PwdError),
}