use serde_json::Value;

use crate::{interfaces::exercise::ExerciseResult, models::exercise::{Conspect, Definition, Exercise, HeaderBody, Node}};

use super::error::ExerciseError;

pub struct ExerciseValidator;

impl ExerciseValidator {
    pub fn validate_exercise(exercise: &Exercise) -> ExerciseResult<()> {
        match exercise.exercise_type {
            crate::models::exercise::ExerciseType::Conspect | 
            crate::models::exercise::ExerciseType::InteractiveConspect => Self::validate_conspect_body(exercise.body.clone()),
        }
    }

    fn validate_conspect_body(body: Value) -> ExerciseResult<()> {
        let body: Conspect = serde_json::from_value(body)
            .map_err(|_| ExerciseError::IncorrectExerciseBodyFormat)?;

        Self::validate_nodes(&body.nodes)
    }

    fn validate_nodes(nodes: &Vec<Node>) -> ExerciseResult<()>  {
        let number_of_nodes = nodes.len();
        if number_of_nodes < 3 {
            return Err(ExerciseError::NotEnoughNodesError { number_of_nodes }.into());
        }

        for node in nodes {
            Self::validate_node_body(node)?;
        }

        Ok(())
    }

    fn validate_node_body(node: &Node) -> ExerciseResult<()> {
        match node.node_type {
            crate::models::exercise::NodeType::Header => Self::validate_node_header(&node.body),
            crate::models::exercise::NodeType::Definition => Self::validate_node_definition(&node.body),
            crate::models::exercise::NodeType::ProcessStages => Self::validate_node_process_stages(&node.body),
        }
    }

    fn validate_node_header(body: &str) -> ExerciseResult<()> {
        serde_json::from_str::<HeaderBody>(body)
            .map_err(|_| ExerciseError::IncorrectHeaderFormat)?;

        Ok(())
    }

    fn validate_node_definition(body: &str) -> ExerciseResult<()> {
        serde_json::from_str::<Definition>(body)
            .map_err(|_| ExerciseError::IncorrectDefinitionFormat)?;

        Ok(())
    }

    fn validate_node_process_stages(body: &str) -> ExerciseResult<()> {
        serde_json::from_str::<Definition>(body)
            .map_err(|_| ExerciseError::IncorrectProcessStagesFormat)?;

        Ok(())
    }
}