#[derive(Debug)]
pub enum CoreError {
    CourseMustBePublishedError,
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