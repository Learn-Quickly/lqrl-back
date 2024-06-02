use lib_core::models::exercise::ExerciseEstimate;
use lib_db::query_repository::exercise::ExerciseQuery;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ExercisePayload {
    pub exercise_id: i64,
    pub lesson_id: i64,
    pub title: String,
    pub description: String,
    pub exercise_type: String,
    pub exercise_body: Value,
    pub answer_body: Value,
    pub exercise_order: i32,
    pub difficult: String,
    pub time_to_complete: Option<i32>,  
}

impl From<ExerciseQuery> for ExercisePayload {
    fn from(value: ExerciseQuery) -> Self {
        Self {
            exercise_id: value.id,
            lesson_id: value.lesson_id,
            title: value.title,
            description: value.description.clone(),
            exercise_type: value.exercise_type.to_string(),
            exercise_body: value.exercise_body.clone(),
            answer_body: value.answer_body,
            exercise_order: value.exercise_order,
            difficult: value.difficult,
            time_to_complete: value.time_to_complete,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExerciseCreatePayload {
    pub lesson_id: i64,
    pub title: String,
    pub description: String,
    pub exercise_type: String,
    pub exercise_body: Value,
    pub answer_body: Value,
    pub difficult: String,
    pub time_to_complete: Option<i32>,  
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExerciseCreatedPayload {
    pub exercise_id: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExerciseForUpdatePayload {
    pub exercise_id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub exercise_type: Option<String>,
    pub exercise_body: Option<Value>,
    pub answer_body: Option<Value>,
    pub difficult: Option<String>,
    pub time_to_complete: Option<i64>,  
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExerciseChangeOrderPayload {
    pub exercise_id: i64,
    pub order: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExerciseId {
    pub exercise_id: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExerciseCompletionId {
    pub exercise_completion_id: i64
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExerciseCompletionForSaveChanges {
    pub exercise_completion_id: i64,
    pub body: Value,
}

#[derive(Serialize, ToSchema)] 
pub struct ExerciseEstimatePayload {
    pub points: f32,
    pub max_points: f32,
    pub difficulty: String,
    pub state: String,   
}

impl From<ExerciseEstimate> for ExerciseEstimatePayload {
    fn from(value: ExerciseEstimate) -> Self {
        Self {
            points: value.points,
            max_points: value.max_points,
            difficulty: value.difficulty.to_string(),
            state: value.state.to_string(),
        }
    }
}