use std::sync::Arc;

use crate::{ctx::Ctx, interactors::{error::ExerciseError, exercise_validator::ExerciseValidator, permission_manager::PermissionManager}, interfaces::{command_repository_manager::ICommandRepositoryManager, exercise::ExerciseResult}, models::exercise::{Exercise, ExerciseForCreateCommand, ExerciseForUpdate}};


pub struct CreatorExerciseInteractor {
    permission_manager: PermissionManager,
    repository_manager: Arc<dyn ICommandRepositoryManager + Send + Sync>,
}

impl CreatorExerciseInteractor {
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

impl CreatorExerciseInteractor {
    pub async fn create_exercise(
        &self, 
        ctx: &Ctx,
        exercise: Exercise, 
    ) -> ExerciseResult<i64> {
        self.permission_manager
            .check_lesson_creator_permission(ctx, exercise.lesson_id)
            .await?;

        ExerciseValidator::validate_exercise(&exercise.exercise_type, exercise.body.clone())?;

        let exercise_repository = self.repository_manager.get_exercise_repository();
        let lesson_exercises = exercise_repository
            .get_lesson_exercises_ordered(ctx, exercise.lesson_id)
            .await?;

        let order = lesson_exercises.len() + 1;
        
        let exercise_for_c = ExerciseForCreateCommand {
            lesson_id: exercise.lesson_id,
            title: exercise.title.clone(),
            description: exercise.description.clone(),
            exercise_type: exercise.exercise_type.clone(),
            body: exercise.body,
            exercise_order: order as i32,
            difficult: exercise.difficult,
            time_to_complete: exercise.time_to_complete,
        };

        exercise_repository.create(ctx, exercise_for_c).await
    }

    pub async fn update_exercise(
        &self, 
        ctx: &Ctx,
        exercise_for_u: ExerciseForUpdate
    ) -> ExerciseResult<()> {
        self.permission_manager
            .check_exercise_creator_permission(ctx, exercise_for_u.id)
            .await?;

        if let Some(body) = exercise_for_u.body.clone() {
            if let Some(exercise_type) = exercise_for_u.exercise_type.clone() {
                ExerciseValidator::validate_exercise(&exercise_type, body)?;
            } else {
                return Err(ExerciseError::CannotUpdateExerciseBodyWithoutType {}.into())
            }
        } else if let Some(_) = exercise_for_u.exercise_type {
            return Err(ExerciseError::CannotUpdateExercisetypeWithoutBody {}.into())
        }

        let exercise_repository = self.repository_manager.get_exercise_repository();

        exercise_repository.update(ctx, exercise_for_u).await
    }
}
