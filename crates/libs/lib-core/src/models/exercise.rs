use derive_more::Display;
use serde::Deserialize;
use serde_json::Value;

use crate::interactors::error::{CoreError, ExerciseError};

use super::exercise_completion::ExerciseCompletionState;

#[derive(Clone)]
pub struct Exercise {
    pub lesson_id: i64,
    pub title: String,
    pub description: String,
    pub exercise_type: ExerciseType,
    pub answer_body: Value,
    pub exercise_body: Value,
    pub difficult: ExerciseDifficulty,
    pub time_to_complete: Option<i32>,  
    pub exercise_order: i32,
}

#[derive(Clone, Display)]
pub enum ExerciseType {
    Conspect,
    InteractiveConspect,
}

impl TryFrom<String> for ExerciseType {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Conspect" => Ok(Self::Conspect),
            "InteractiveConspect" => Ok(Self::InteractiveConspect),
            _ => Err(ExerciseError::IncorrectExerciseType {}.into())
        }
    }
}

#[derive(Clone, Display)]
pub enum ExerciseDifficulty {
    Read,
    Easy,
    Medium,
    Hard,
}

impl From<ExerciseDifficulty> for f32 {
    fn from(value: ExerciseDifficulty) -> Self {
        match value {
            ExerciseDifficulty::Read => 0.0,
            ExerciseDifficulty::Easy => 0.35,
            ExerciseDifficulty::Medium => 0.7,
            ExerciseDifficulty::Hard => 1.0,
        }
    }
}

impl TryFrom<String> for ExerciseDifficulty {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Read" => Ok(Self::Read),
            "Easy" => Ok(Self::Easy),
            "Medium" => Ok(Self::Medium),
            "Hard" => Ok(Self::Hard),
            _ => Err(ExerciseError::IncorrectExerciseDifficulty {}.into())
        }
    }
}

#[derive(Deserialize)]
pub struct Conspect {
    pub connections: Vec<Connection>,
    pub nodes: Vec<Node>,
}

#[derive(Deserialize)]
pub struct Connection {
    pub from: String, // Node id
    pub to: String,
}

#[derive(Deserialize)]
pub struct Node {
    pub id: String,
    pub x: i64,
    pub y: i64,
    pub node_type: NodeType,
    pub body: String,
}

#[derive(Deserialize)]
pub enum NodeType {
    Header, 
    Definition,
    ProcessStages,
}

#[derive(Deserialize)]
pub struct HeaderBody {
    pub header: String,
}

#[derive(Deserialize)]
pub struct Definition {
    pub header: String,
    pub definition: String,
}

#[derive(Deserialize)]
pub struct ProcessStages {
    pub header: String,
    pub stages: Vec<Stage>,
}

#[derive(Deserialize)]
pub struct Stage {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct ExerciseForChangeOrder {
    pub id: i64,
    pub order: i32,
}

#[derive(Clone)]
pub struct ExerciseForCreate {
    pub lesson_id: i64,
    pub title: String,
    pub description: String,
    pub exercise_type: ExerciseType,
    pub answer_body: Value,
    pub exercise_body: Value,
    pub difficult: ExerciseDifficulty,
    pub time_to_complete: Option<i32>,  
}

pub struct ExerciseForCreateCommand {
    pub lesson_id: i64,
    pub title: String,
    pub description: String,
    pub exercise_type: ExerciseType,
    pub answer_body: Value,
    pub exercise_body: Value,
    pub exercise_order: i32,
    pub difficult: ExerciseDifficulty,
    pub time_to_complete: Option<i32>,  
}

#[derive(Clone)]
pub struct ExerciseForUpdate {
    pub id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub exercise_type: Option<ExerciseType>,
    pub answer_body: Option<Value>,
    pub exercise_body: Option<Value>,
    pub difficult: Option<ExerciseDifficulty>,
    pub time_to_complete: Option<i64>,  
}

pub struct ExerciseEstimate {
    pub points: f32,
    pub max_points: f32,
    pub difficulty: ExerciseDifficulty,
    pub state: ExerciseCompletionState,
}