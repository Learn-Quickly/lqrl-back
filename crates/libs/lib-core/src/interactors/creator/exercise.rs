use std::sync::Arc;

use crate::{ctx::Ctx, interactors::{exercise_validator::ExerciseValidator, permission_manager::PermissionManager}, interfaces::{command_repository_manager::ICommandRepositoryManager, exercise::ExerciseResult}, models::exercise::{Exercise, ExerciseForCreateCommand}};


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
        let lesson_repository = self.repository_manager.get_lesson_repository();
        let lesson = lesson_repository.get_lesson(ctx, exercise.lesson_id).await?;

        self.permission_manager
            .check_course_creator_permission(ctx, lesson.course_id)
            .await?;

        ExerciseValidator::validate_exercise(&exercise)?;

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

        let exercise_id = exercise_repository.create(ctx, exercise_for_c).await?;

        Ok(exercise_id)
    }
}
