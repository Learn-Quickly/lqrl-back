use lib_db::query_repository::lesson_progress::LessonProgressData;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GetLessonProgressesPayload {
    pub course_id: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LessonProgressPayload {
    pub user_id: i64,
    pub lesson_id: i64,

    pub date_started: i64,
    pub date_complete: Option<i64>,  

    pub state: String,
}

impl From<&LessonProgressData> for LessonProgressPayload {
    fn from(value: &LessonProgressData) -> Self {
        Self {
            user_id: value.user_id,
            lesson_id: value.lesson_id,
            date_started: value.date_started.unix_timestamp(),
            date_complete: value.date_complete.and_then(|date| Some(date.unix_timestamp())),
            state: value.state.clone(),
        }
    }
}