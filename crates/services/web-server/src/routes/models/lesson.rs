use serde::{Deserialize, Serialize};
use utoipa::ToSchema;


#[derive(Debug, Serialize, ToSchema)]
pub struct LessonCreatedPayload {
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