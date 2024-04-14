use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

pub use time::format_description::well_known::Rfc3339;

pub fn now_utc() -> OffsetDateTime {
	OffsetDateTime::now_utc()
}

pub fn from_unix_timestamp(secs: i64) -> Result<OffsetDateTime> {
	OffsetDateTime::from_unix_timestamp(secs)
		.map_err(|err| DateError::FailToParseUnixTimestamp(err.to_string()))
}

pub fn format_time(time: OffsetDateTime) -> String {
	time.format(&Rfc3339).unwrap() // TODO: need to check if safe.
}

pub fn now_utc_plus_sec_str(sec: f64) -> String {
	let new_time = now_utc() + Duration::seconds_f64(sec);
	format_time(new_time)
}

pub fn parse_utc(moment: &str) -> Result<OffsetDateTime> {
	OffsetDateTime::parse(moment, &Rfc3339)
		.map_err(|_| DateError::FailToDateParse(moment.to_string()))
}

// region:    --- Error

pub type Result<T> = core::result::Result<T, DateError>;

#[derive(Debug, Serialize, Deserialize)]
pub enum DateError {
	FailToDateParse(String),
	FailToParseUnixTimestamp(String),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for DateError {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for DateError {}
// endregion: --- Error Boilerplate

// endregion: --- Error
