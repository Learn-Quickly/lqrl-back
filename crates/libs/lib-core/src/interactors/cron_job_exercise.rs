use std::sync::Arc;

use lib_utils::time::now_utc_sec;

use crate::{ctx::Ctx, interactors::exercise_checker::ExerciseChecker, interfaces::{command_repository_manager::ICommandRepositoryManager, exercise::ExerciseResult}, models::exercise_completion::{ExerciseCompletion, ExerciseCompletionForCompleteCommand}};

pub struct CronJobExercise {
    repository_manager: Arc<dyn ICommandRepositoryManager + Send + Sync>,
}

impl CronJobExercise {
    pub fn new(
        repository_manager: Arc<dyn ICommandRepositoryManager + Send + Sync>,
    ) -> Self {
        Self {
            repository_manager,
        }
    }
}

impl CronJobExercise {
    /// return number of completed exercises 
    pub async fn complete_overdue_exercises(&self) -> ExerciseResult<i32> {
        let ctx = Ctx::root_ctx();

        let exercise_repository = self.repository_manager.get_exercise_repository();
        let uncompleted_exercises = exercise_repository.get_uncompleted_exercises(&ctx).await?;

        let mut result = 0;

        for exercise_completion in uncompleted_exercises {
            result += self.complete_exercise(&ctx, exercise_completion).await?;
        }

        Ok(result)
    }

    async fn complete_exercise(&self, ctx: &Ctx, ex_comp: ExerciseCompletion) -> ExerciseResult<i32> {
        let exercise_repository = self.repository_manager.get_exercise_repository();

        let exercise = exercise_repository.get_exercise(ctx, ex_comp.exercise_id).await?;

        let now = now_utc_sec();
        if let Some(time_to_complete) = exercise.time_to_complete {
            let deadline = time_to_complete as i64 + ex_comp.date_started;

            if now < deadline {
                return Ok(0);
            }
        }

        let exercise_estimate = ExerciseChecker::evaluate_exercise(&exercise, &ex_comp)?;

        let ex_comp_for_u = ExerciseCompletionForCompleteCommand {
            points_scored: exercise_estimate.points,
            max_points: exercise_estimate.max_points,
            state: exercise_estimate.state,
            id: ex_comp.id,
        };


        exercise_repository.complete_exercise_completion(ctx, ex_comp_for_u).await?;

        Ok(1)
    }
}
