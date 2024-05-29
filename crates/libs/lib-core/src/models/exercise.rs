use derive_more::Display;
use serde::Deserialize;
use serde_json::Value;

use crate::interactors::error::{CoreError, ExerciseError};

#[derive(Clone)]
pub struct Exercise {
    pub lesson_id: i64,
    pub title: String,
    pub description: String,
    pub exercise_type: ExerciseType,
    pub body: Value,
    pub difficult: ExerciseDifficult,
    pub time_to_complete: Option<i64>,  
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
pub enum ExerciseDifficult {
    Read,
    Easy,
    Medium,
    Hard,
}

impl TryFrom<String> for ExerciseDifficult {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Read" => Ok(Self::Read),
            "Easy" => Ok(Self::Easy),
            "Medium" => Ok(Self::Medium),
            "Hard" => Ok(Self::Hard),
            _ => Err(ExerciseError::IncorrectExerciseDifficult {}.into())
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

pub struct ExerciseForCreateCommand {
    pub lesson_id: i64,
    pub title: String,
    pub description: String,
    pub exercise_type: ExerciseType,
    pub body: Value,
    pub exercise_order: i32,
    pub difficult: ExerciseDifficult,
    pub time_to_complete: Option<i64>,  
}

#[derive(Clone)]
pub struct ExerciseForUpdate {
    pub id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub exercise_type: Option<ExerciseType>,
    pub body: Option<Value>,
    pub difficult: Option<ExerciseDifficult>,
    pub time_to_complete: Option<i64>,  
}