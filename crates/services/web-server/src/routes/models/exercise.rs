use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExerciseCreatePayload {
    pub lesson_id: i64,
    pub title: String,
    pub description: String,
    pub exercise_type: String,
    pub body: Value,
    pub exercise_order: i32,
    pub difficult: String,
    pub time_to_complete: Option<i64>,  
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExerciseCreatedPayload {
    pub exercise_id: i64,
}