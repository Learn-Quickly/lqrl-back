use std::sync::Arc;

use lib_utils::time::now_utc_sec;

use crate::{ctx::Ctx, interactors::permission_manager::PermissionManager, interfaces::{command_repository_manager::ICommandRepositoryManager, exercise::ExerciseResult}, models::exercise_completion::{ExerciseCompletionForCreate, ExerciseCompletionState}};

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
    ) -> ExerciseResult<()> {
        self.permission_manager.check_exercise_student_permission(ctx, exercise_id).await?;
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
}