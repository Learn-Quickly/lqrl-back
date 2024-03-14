use crate::pwd::scheme;
use derive_more::From;
use serde::{Deserialize, Serialize};

pub type Result<T> = core::result::Result<T, PwdError>;

#[derive(Debug, Serialize, Deserialize, From)]
pub enum PwdError {
	PwdWithSchemeFailedParse,

	FailSpawnBlockForValidate,
	FailSpawnBlockForHash,

	// -- Modules
	#[from]
	Scheme(scheme::PwdSchemeError),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for PwdError {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for PwdError {}
// endregion: --- Error Boilerplate
