use derive_more::From;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum CoreError {
	// Course error
    CourseMustBePublishedError,
	PermissionDenied,
	CreatorCannotSubscribeToTheCourse,
	CannotRegisterForCourseTwice,

	// File error
	#[from]
	IOError(#[serde_as(as = "DisplayFromStr")] std::io::Error),
}

impl core::fmt::Display for CoreError {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for CoreError { }