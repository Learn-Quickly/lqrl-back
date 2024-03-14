use serde::{Deserialize, Serialize};

pub type Result<T> = core::result::Result<T, PwdSchemeError>;

#[derive(Debug, Serialize, Deserialize)]
pub enum PwdSchemeError {
	Key,
	Salt,
	Hash,
	PwdValidate,
	SchemeNotFound(String),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for PwdSchemeError {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for PwdSchemeError {}
// endregion: --- Error Boilerplate
