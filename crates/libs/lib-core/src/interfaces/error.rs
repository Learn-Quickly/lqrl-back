use std::error::Error;
use std::fmt::{Debug, Display};

pub trait IAppError: Error + Display + Debug + Sync + Send {
	fn error_type(&self) -> String;
	fn description(&self) -> String;
}
