use serde::Serialize;

pub type Result<T> = core::result::Result<T, TokenError>;

#[derive(Debug, Serialize)]
pub enum TokenError {
	HmacFailNewFromSlice,

	InvalidFormat,
	CannotDecodeIdent,
	CannotDecodeExp,
	SignatureNotMatching,
	ExpNotIso,
	Expired,
}

// region:    --- Error Boilerplate
impl core::fmt::Display for TokenError {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for TokenError {}
// endregion: --- Error Boilerplate
