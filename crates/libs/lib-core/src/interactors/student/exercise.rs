use std::sync::Arc;

use lib_utils::time::now_utc_sec;
use serde_json::Value;

use crate::{ctx::Ctx, interactors::{error::ExerciseError, exercise_checker::ExerciseChecker, exercise_validator::ExerciseValidator, permission_manager::PermissionManager}, interfaces::{command_repository_manager::ICommandRepositoryManager, exercise::ExerciseResult}, models::{exercise::ExerciseEstimate, exercise_completion::{ExerciseCompletionForCompleteCommand, ExerciseCompletionForCreate, ExerciseCompletionForUpdate, ExerciseCompletionState}, lesson_progress::LessonProgressState}};

pub struct StudentExerciseInteractor {
    permission_manager: PermissionManager,
    repository_manager: Arc<dyn ICommandRepositoryManager + Send + Sync>,
}

impl StudentExerciseInteractor {
    pub fn new(
        repository_manager: Arc<dyn ICommandRepositoryManager + Send + Sync>,
    ) -> Self {
        let permission_manager = PermissionManager::new(repository_manager.clone());

        Self {
            permission_manager,
            repository_manager,
        }
    }
}

impl StudentExerciseInteractor {
    pub async fn start_exercise(
        &self,
        ctx: &Ctx,
        exercise_id: i64
    ) -> ExerciseResult<i64> {
        self.permission_manager.check_exercise_student_permission(ctx, exercise_id).await?;
        
        self.check_lesson_state(ctx, exercise_id).await?;
        self.check_exercise_order(ctx, exercise_id).await?;

        let exercise_repository = self.repository_manager.get_exercise_repository();
        let pre_exercise_completions = exercise_repository.get_exercise_user_completions(ctx, ctx.user_id(), exercise_id).await?;
        let number_of_attempts = pre_exercise_completions.len();
        let date_started = now_utc_sec();

        let ex_comp_for_c = ExerciseCompletionForCreate {
            exercise_id,
            user_id: ctx.user_id(),
            number_of_attempts,
            date_started,
        };

        exercise_repository.create_exercise_completion(ctx, ex_comp_for_c).await       
    }

    async fn check_lesson_state(&self, ctx: &Ctx, exercise_id: i64) -> ExerciseResult<()> {
        let exercise_repository = self.repository_manager.get_exercise_repository();
        let exercise = exercise_repository.get_exercise(ctx, exercise_id).await?;

        let user_id = ctx.user_id();
        
        let lesson_repository = self.repository_manager.get_lesson_repository();
        let lesson = lesson_repository.get_lesson(ctx, exercise.lesson_id).await?;
        let lesson_progresses = lesson_repository.get_lessons_progresses(ctx, lesson.course_id, user_id).await?;

        match lesson_progresses.iter().find(|lesson_progress| lesson_progress.lesson_id == lesson.id) {
            Some(lesson_progress) => {
                if lesson_progress.state.ne(&LessonProgressState::InProgress) {
                    return Err(ExerciseError::LessonProgressMustBeInProgress {}.into());
                }
            },
            None => return Err(ExerciseError::LessonProgressMustBeInProgress {}.into()),
        }

        Ok(())
    }

    async fn check_exercise_order(&self, ctx: &Ctx, exercise_id: i64) -> ExerciseResult<()> {
        let exercise_repository = self.repository_manager.get_exercise_repository();
        let exercise = exercise_repository.get_exercise(ctx, exercise_id).await?;

        if exercise.exercise_order == 1 {
            return Ok(());
        }

        let exercises_ordered = exercise_repository
            .get_lesson_exercises_ordered(ctx, exercise.lesson_id)
            .await?;

        let previus_ex_id = match exercises_ordered.get((exercise.exercise_order - 2) as usize) {
            Some(prev_ex) => prev_ex.id,
            None => return Err(crate::interactors::error::ExerciseError::PreviousExerciseNotFound { exercise_id}.into()),
        };

        let pre_exercise_completions = exercise_repository.get_exercise_user_completions(ctx, ctx.user_id(), previus_ex_id).await?;
        
        match pre_exercise_completions.iter().find(|ex_comp| ex_comp.state == ExerciseCompletionState::Succeeded) {
            Some(_) => Ok(()),
            None => Err(crate::interactors::error::ExerciseError::PreviousExerciseNotCompleted { exercise_id: previus_ex_id }.into()),
        }
    }

    pub async fn save_exercise_execution_changes(
        &self, 
        ctx: &Ctx, 
        ex_comp_id: i64, 
        exercise_body_for_save: Value,
    ) -> ExerciseResult<()> {
        let exercise_repository = self.repository_manager.get_exercise_repository();
        let ex_comp = exercise_repository.get_exercise_completion(ctx, ex_comp_id).await?;

        let user_id = ctx.user_id();

        if ex_comp.user_id != user_id {
            return Err(ExerciseError::ExerciseCompletionAccessDenied { user_id, ex_comp_id }.into());
        }

        let exercise = exercise_repository.get_exercise(ctx, ex_comp.exercise_id).await?;
        
        let now = now_utc_sec();
        if let Some(time_to_complete) = exercise.time_to_complete {
            let deadline = time_to_complete as i64 + ex_comp.date_started;

            if now > deadline {
                return Err(ExerciseError::TimeToCompleteExerciseHasExpired {}.into());
            }
        }

        ExerciseValidator::validate_exercise(&exercise.exercise_type, exercise_body_for_save.clone())?;

        let ex_comp_for_u = ExerciseCompletionForUpdate {
            body: exercise_body_for_save,
            date_last_changes: now,
            id: ex_comp_id,
        };

        exercise_repository.update_exercise_completion(ctx, ex_comp_for_u).await?;

        Ok(())
    }

    pub async fn complete_exercise(&self, ctx: &Ctx, ex_comp_id: i64) -> ExerciseResult<ExerciseEstimate> {
        let exercise_repository = self.repository_manager.get_exercise_repository();
        let ex_comp = exercise_repository.get_exercise_completion(ctx, ex_comp_id).await?;

        let user_id = ctx.user_id();

        if ex_comp.user_id != user_id {
            return Err(ExerciseError::ExerciseCompletionAccessDenied { user_id, ex_comp_id }.into());
        }

        if ex_comp.state.ne(&ExerciseCompletionState::InProgress) {
            return Err(ExerciseError::AttemptHasAlreadyBeenCompleted {}.into());
        }

        let exercise = exercise_repository.get_exercise(ctx, ex_comp.exercise_id).await?;

        let exercise_estimate = ExerciseChecker::evaluate_exercise(&exercise, &ex_comp)?;

        let ex_comp_for_u = ExerciseCompletionForCompleteCommand {
            points_scored: exercise_estimate.points,
            max_points: exercise_estimate.max_points,
            state: exercise_estimate.state,
            id: ex_comp_id,
        };

        exercise_repository.complete_exercise_completion(ctx, ex_comp_for_u).await?;

        if self.is_lesson_state_complete(ctx, exercise.lesson_id, user_id).await? {
            self.complete_lesson(ctx, exercise.lesson_id, user_id).await?;
        }

        Ok(exercise_estimate)
    }

    async fn is_lesson_state_complete(&self, ctx: &Ctx, lesson_id: i64, user_id: i64) -> ExerciseResult<bool> {
        let exercise_repository = self.repository_manager.get_exercise_repository();

        let number_of_completed_exercises = exercise_repository.get_number_of_lesson_completed_exercises(ctx, lesson_id, user_id).await?;
        let exercises = exercise_repository.get_lesson_exercises_ordered(ctx, lesson_id).await?;

        if exercises.len() as i64 == number_of_completed_exercises {
            return Ok(true);
        }

        Ok(false)
    }

    async fn complete_lesson(&self, ctx: &Ctx, lesson_id: i64, user_id: i64) -> ExerciseResult<()> {
        let lesson_repository = self.repository_manager.get_lesson_repository();

        lesson_repository.update_lesson_progress_state(ctx, LessonProgressState::Done, lesson_id, user_id).await
    }
}