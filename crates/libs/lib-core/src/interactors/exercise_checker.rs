use serde_json::Value;

use crate::{interfaces::exercise::ExerciseResult, models::{exercise::{Connection, Conspect, Exercise, ExerciseEstimate, Node, ProcessStages}, exercise_completion::{ExerciseCompletion, ExerciseCompletionState}}};

use super::error::ExerciseError;

pub struct Estimate {
    pub points: i64,
    pub max_points: i64,
}

pub struct ExerciseChecker;

impl ExerciseChecker {
    pub fn evaluate_exercise(exercise: &Exercise, ex_comp: &ExerciseCompletion) -> ExerciseResult<ExerciseEstimate> {
        if exercise.difficult.eq(&crate::models::exercise::ExerciseDifficulty::Read) {
            return Ok(ExerciseEstimate {
                points: 0.0,
                max_points: 0.0,
                difficulty: crate::models::exercise::ExerciseDifficulty::Read,
                state: ExerciseCompletionState::Succeeded,
            });
        }

        let conspect_estimate = match exercise.exercise_type {
            crate::models::exercise::ExerciseType::Conspect | 
            crate::models::exercise::ExerciseType::InteractiveConspect => Self::evaluate_conspects_bodies(exercise.answer_body.clone(), ex_comp.body.clone())?,
        };

        let max_points: f32 = f32::from(exercise.difficult.clone()) * 100.0;
        let points = max_points * conspect_estimate.points as f32 / conspect_estimate.max_points as f32;        

        let state = if points / max_points < 0.6 {
            ExerciseCompletionState::Failed
        } else {
            ExerciseCompletionState::Succeeded
        };

        Ok(ExerciseEstimate {
            points: points,
            max_points,
            difficulty: exercise.difficult.clone(),
            state,
        })
    }

    fn evaluate_conspects_bodies(answer_conspect: Value, solution_conspect: Value) -> ExerciseResult<Estimate> {
        let answer_conspect: Conspect = serde_json::from_value(answer_conspect)
            .map_err(|_| ExerciseError::IncorrectExerciseBodyFormat)?;

        let solution_conspect: Conspect = serde_json::from_value(solution_conspect)
            .map_err(|_| ExerciseError::IncorrectExerciseBodyFormat)?;

        let connections_estimate = Self::evaluate_connections(&answer_conspect.connections, &solution_conspect.connections)?;
        let nodes_estimate = Self::evaluate_nodes(&answer_conspect.nodes, &solution_conspect.nodes)?;

        let points = nodes_estimate.points + connections_estimate.points;
        let max_points = nodes_estimate.max_points + connections_estimate.max_points;

        Ok(Estimate { points, max_points })
    }

    fn evaluate_connections(answer_connections: &Vec<Connection>, solution_connections: &Vec<Connection>) -> ExerciseResult<Estimate> {
        let mut result = Estimate {
            points: 0, 
            max_points: answer_connections.len() as i64, 
        };
        
        for answer_connection in answer_connections {
            let connection_exist = solution_connections
                .iter()
                .find(|connection| connection.from == answer_connection.from && connection.to == answer_connection.to)
                .is_some();

            if connection_exist {
                result.points += 1;
            }
        }

        let dconn = solution_connections.len() as i64 - answer_connections.len() as i64;
        if dconn > 0 {
            result.points -= dconn;
        }

        Ok(result)
    }

    fn evaluate_nodes(answer_nodes: &Vec<Node>, solution_nodes: &Vec<Node>) -> ExerciseResult<Estimate>  {
        let mut result = Estimate {
            points: 0,
            max_points: 0,
        };

        for node in answer_nodes {
            let estimate = Self::validate_node_body(node, solution_nodes)?;
            result.max_points += estimate.max_points;
            result.points += estimate.points;
        }

        Ok(result)
    }

    fn validate_node_body(node: &Node, solution_nodes: &Vec<Node>) -> ExerciseResult<Estimate> {
        match node.node_type {
            crate::models::exercise::NodeType::Header => Ok(Estimate { points: 0, max_points: 0 }),
            crate::models::exercise::NodeType::Definition => Ok(Estimate { points: 0, max_points: 0 }),
            crate::models::exercise::NodeType::ProcessStages => Self::evaluate_node_process_stages(&node, solution_nodes),
        }
    }

    fn evaluate_node_process_stages(node: &Node, solution_nodes: &Vec<Node>) -> ExerciseResult<Estimate> {
        let solution_node = match solution_nodes.iter().find(|solution_node| solution_node.id == node.id) {
            Some(solution_node) => solution_node,
            None => return Ok(Estimate { points: 0, max_points: 0 }),
        };

        let stages = serde_json::from_str::<ProcessStages>(&node.body)
            .map_err(|_| ExerciseError::IncorrectProcessStagesFormat)?.stages;

        let solution_stages = serde_json::from_str::<ProcessStages>(&solution_node.body)
            .map_err(|_| ExerciseError::IncorrectProcessStagesFormat)?.stages;

        let mut points = 0;

        for i in 0..stages.len() {
            let stage = match stages.get(i) {
                Some(stage) => stage,
                None => break,
            };

            let solution_stage = match solution_stages.get(i) {
                Some(stage) => stage,
                None => break,
            };

            if stage.id == solution_stage.id {
                points += 1;
            }
        }
        
        Ok(Estimate { 
            points, 
            max_points: stages.len() as i64,
        })
    }
}