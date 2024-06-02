use derive_more::Display;
use serde_json::Value;

use crate::interactors::error::ExerciseError;

pub struct ExerciseCompletion {
    pub id: i64,
    pub exercise_id: i64,
    pub user_id: i64,
    pub points_scored: Option<f32>,
    pub max_points: Option<f32>,
    pub number_of_attempts: i32,
    pub date_started: i64,
    pub date_last_changes: Option<i64>,
    pub state: ExerciseCompletionState,   
    pub body: Value,
}

#[derive(PartialEq, Display, Clone, Copy)]
pub enum ExerciseCompletionState {
    InProgress,
    Succeeded,
    Failed,      
}

impl TryFrom<String> for ExerciseCompletionState {
    type Error = ExerciseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "InProgress" => Ok(Self::InProgress),
            "Succeeded" => Ok(Self::Succeeded),
            "Failed" => Ok(Self::Failed),
            state => Err(ExerciseError::ExerciseCompletionStateDoesNotExist { state: state.to_string() }.into()),
        }
    }
}

pub struct ExerciseCompletionForCreate {
    pub exercise_id: i64,
    pub user_id: i64,
    pub number_of_attempts: usize,
    pub date_started: i64,
}

pub struct ExerciseCompletionForUpdate {
    pub body: Value,
    pub date_last_changes: i64,
}

pub struct ExerciseCompletionForCompleteCommand {
    pub points_scored: f32,
    pub max_points: f32,
    pub state: ExerciseCompletionState,
}