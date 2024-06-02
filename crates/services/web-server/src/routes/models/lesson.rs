use lib_db::query_repository::lesson::LessonData;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;


#[derive(Debug, Serialize, ToSchema)]
pub struct LessonCreatedPayload {
    pub lesson_id: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StartLessonPayload {
    pub lesson_id: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LessonCreatePayload {
    pub course_id: i64,
    pub title: String,
	pub description: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LessonDeletePayload {
	pub lesson_id: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LessonUpdatePayload {
    pub lesson_id: i64,
    pub title: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LessonChangeOrderPayload {
    pub lesson_id: i64,
    pub order: i32,
}

#[derive(Debug, Serialize, ToSchema)] 
pub struct LessonDataPayload {
	pub id: i64,
	pub course_id: i64,
    pub title: String,
    pub lesson_order: i32,
    pub description: String,
}

impl From<LessonData> for LessonDataPayload {
    fn from(lesson_data: LessonData) -> Self {
        Self {
            id: lesson_data.id,
            course_id: lesson_data.course_id,
            title: lesson_data.title.clone(),
            lesson_order: lesson_data.lesson_order,
            description: lesson_data.description,
        }
    }
}