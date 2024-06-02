use lib_core::models::exercise::ExerciseEstimate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExerciseCreatePayload {
    pub lesson_id: i64,
    pub title: String,
    pub description: String,
    pub exercise_type: String,
    pub exercise_body: Value,
    pub answer_body: Value,
    pub exercise_order: i32,
    pub difficult: String,
    pub time_to_complete: Option<i64>,  
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