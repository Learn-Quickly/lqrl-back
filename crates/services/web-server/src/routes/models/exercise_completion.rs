use lib_db::query_repository::exercise_completion::ExerciseCompletionQuery;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ExerciseCompletionPayload {
    pub exercise_completion_id: i64,
    pub exercise_id: i64,
    pub user_id: i64,
    pub points_scored: Option<f32>,
    pub max_points: Option<f32>,
    pub number_of_attempts: i32,
    pub date_started: i64,
    pub date_last_changes: Option<i64>,
    pub state: String,   
    pub body: serde_json::Value,
}

impl From<ExerciseCompletionQuery> for ExerciseCompletionPayload {
    fn from(value: ExerciseCompletionQuery) -> Self {
        Self {
            exercise_completion_id: value.exercise_completion_id,
            exercise_id: value.exercise_id,
            user_id: value.user_id,
            points_scored: value.points_scored,
            max_points: value.max_points,
            number_of_attempts: value.number_of_attempts,
            date_started: value.date_started,
            date_last_changes: value.date_last_changes,
            state: value.state,
            body: value.body,
        }
    }
}