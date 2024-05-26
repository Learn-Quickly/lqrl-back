use crate::interactors::error::{CoreError, LessonProgressError};

pub struct LessonProgress {
  pub user_id: i64,
  pub lesson_id: i64,

  pub date_started: i64,
  pub date_complete: Option<i64>,  

  pub state: LessonProgressState, 
}

#[derive(PartialEq, Eq)]
pub enum LessonProgressState {
    InProgress,
    Done,
}

impl TryFrom<String> for LessonProgressState {
	type Error = CoreError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		match value.as_str() {
			"InProgress" => Ok(Self::InProgress),
			"Done" => Ok(Self::Done),
			state => Err(LessonProgressError::LessonProgressStateDoesNotExist { state: state.to_string() }.into())
		}
	}
}