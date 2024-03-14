use std::borrow::Cow;

use derive_more::From;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use sqlx::error::DatabaseError;

use crate::store::error::DbError;

pub type DbxResult<T> = core::result::Result<T, DbxError>;

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum DbxError {
	TxnCantCommitNoOpenTxn,
	CannotBeginTxnWithTxnFalse,
	CannotCommitTxnWithTxnFalse,

	DbError(DbError),

	// -- Externals
	#[from]
	Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
	#[from]
	SeaQuery(#[serde_as(as = "DisplayFromStr")] sea_query::error::Error),
	#[from]
	ModqlIntoSea(#[serde_as(as = "DisplayFromStr")] modql::filter::IntoSeaError),
}

impl std::error::Error for DbxError {}

impl core::fmt::Display for DbxError {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl DbxError {
	/// This function will transform the error into a more precise variant if it is an SQLX or PGError Unique Violation.
	/// The resolver can contain a function (table_name: &str, constraint: &str) that may return a specific Error if desired.
	/// If the resolver is None, or if the resolver function returns None, it will default to Error::UniqueViolation {table, constraint}.
	pub fn resolve_unique_violation<F>(dbx_error: DbxError, resolver: Option<F>) -> DbError
	where
		F: FnOnce(&str, &str) -> Option<DbError>,
	{
		match dbx_error.as_database_error().map(|db_error| {
			(db_error.code(), db_error.table(), db_error.constraint())
		}) {
			// "23505" => postgresql "unique violation"
			Some((Some(Cow::Borrowed("23505")), Some(table), Some(constraint))) => {
				resolver
					.and_then(|fun| fun(table, constraint))
					.and_then(|err| err.into())
					.unwrap_or_else(|| DbError::UniqueViolation {
						table: table.to_string(),
						constraint: constraint.to_string(),
					})
			}
			_ => dbx_error.into(),
		}
	}

	/// A convenient function to return the eventual database error (Postgres)
	/// if this Error is an SQLX Error that contains a database error.
	pub fn as_database_error(&self) -> Option<&(dyn DatabaseError + 'static)> {
		match self {
			DbxError::Sqlx(sqlx_error) => {
				sqlx_error.as_database_error()
			}
			_ => None,
		}
	}

}