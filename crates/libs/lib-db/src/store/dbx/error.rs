use derive_more::From;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

pub type DbxResult<T> = core::result::Result<T, StoreError>;

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum StoreError {
	TxnCantCommitNoOpenTxn,
	CannotBeginTxnWithTxnFalse,
	CannotCommitTxnWithTxnFalse,

	// -- Externals
	#[from]
	Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
}

// region:    --- Error Boilerplate

impl core::fmt::Display for StoreError {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for StoreError {}

// endregion: --- Error Boilerplate
