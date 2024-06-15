use std::sync::Arc;

use serde_json::Value;

use crate::{ctx::Ctx, interactors::{error::ExerciseError, exercise_validator::ExerciseValidator, permission_manager::PermissionManager}, interfaces::{command_repository_manager::ICommandRepositoryManager, exercise::ExerciseResult}, models::exercise::{ExerciseForChangeOrder, ExerciseForCreate, ExerciseForCreateCommand, ExerciseForUpdate, ExerciseType}};


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
        exercise: ExerciseForCreate, 
    ) -> ExerciseResult<i64> {
        self.permission_manager
            .check_lesson_creator_permission(ctx, exercise.lesson_id)
            .await?;

        ExerciseValidator::validate_exercise(&exercise.exercise_type, exercise.answer_body.clone())?;
        ExerciseValidator::validate_exercise(&exercise.exercise_type, exercise.exercise_body.clone())?;

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
            exercise_order: order as i32,
            difficult: exercise.difficult,
            time_to_complete: exercise.time_to_complete,
            answer_body: exercise.answer_body,
            exercise_body: exercise.exercise_body,
        };

        exercise_repository.create(ctx, exercise_for_c).await
    }

    pub async fn update_exercise(
        &self, 
        ctx: &Ctx,
        exercise_for_u: ExerciseForUpdate,
        is_retake_exercise: bool,
    ) -> ExerciseResult<()> {
        self.permission_manager
            .check_exercise_creator_permission(ctx, exercise_for_u.id)
            .await?;

        self.validate_exercise(exercise_for_u.exercise_type.clone(), exercise_for_u.answer_body.clone())?;
        self.validate_exercise(exercise_for_u.exercise_type.clone(), exercise_for_u.exercise_body.clone())?;

        let exercise_repository = self.repository_manager.get_exercise_repository();

        if is_retake_exercise {
            self.change_lessons_state(ctx, exercise_for_u.id).await?;
            exercise_repository.remove_exercise_completions(ctx, exercise_for_u.id).await?;
        }

        exercise_repository.update(ctx, exercise_for_u).await?;


        Ok(())
    }

    async fn change_lessons_state(&self, ctx: &Ctx, exercise_id: i64) -> ExerciseResult<()> {
        let exercise_repository = self.repository_manager.get_exercise_repository();
        let exercise = exercise_repository.get_exercise(ctx, exercise_id).await?;

        let lesson_repository = self.repository_manager.get_lesson_repository();
        let lesson = lesson_repository.get_lesson(ctx, exercise.lesson_id).await?;

        lesson_repository.change_lesson_progress_states_for_update_exercise(ctx, lesson.course_id, lesson.lesson_order).await?;

        Ok(())
    }

    fn validate_exercise(
        &self, 
        exercise_type: Option<ExerciseType>, 
        body: Option<Value>
    ) -> ExerciseResult<()> {
        if let Some(body) = body {
            if let Some(exercise_type) = exercise_type.clone() {
                ExerciseValidator::validate_exercise(&exercise_type, body)?;
            } else {
                return Err(ExerciseError::CannotUpdateExerciseBodyWithoutType {}.into())
            }
        } else if let Some(_) = exercise_type {
            return Err(ExerciseError::CannotUpdateExercisetypeWithoutBody {}.into())
        }

        Ok(())
    }

    pub async fn change_order(
        &self, 
        ctx: &Ctx,
        exercise_for_u_order: ExerciseForChangeOrder, 
    ) -> ExerciseResult<()> {
        self.permission_manager
            .check_exercise_creator_permission(ctx, exercise_for_u_order.id)
            .await?;

        let exercise_repository = self.repository_manager.get_exercise_repository();
        let exercise = exercise_repository.get_exercise(ctx, exercise_for_u_order.id).await?;

        let lesson_exercises = exercise_repository
            .get_lesson_exercises_ordered(ctx, exercise.lesson_id)
            .await?;

        let lesson_exercises = self.compute_orders(&lesson_exercises, &exercise_for_u_order)?;
        exercise_repository.update_exercise_orders(ctx, lesson_exercises).await?;

        Ok(())
    }

    fn compute_orders(
        &self,
        exercises: &Vec<ExerciseForChangeOrder>, 
        exercise_for_u_order: &ExerciseForChangeOrder
    ) -> ExerciseResult<Vec<ExerciseForChangeOrder>> {
        let number_of_exercises = exercises.len() as i32;
        if number_of_exercises < exercise_for_u_order.order {
            return Err(
                ExerciseError::IncorrectExerciseOreder { 
                    exercise_id: exercise_for_u_order.id,
                    order: exercise_for_u_order.order 
                }.into()
            );
        }

        let mut result = Vec::new();
        let mut d_order = 0;

        for exercise in exercises {
            let order = if exercise.id == exercise_for_u_order.id {
                d_order = if d_order != 0 {
                    0
                } else {
                    -1
                };

                exercise_for_u_order.order
            } else if exercise.order == exercise_for_u_order.order {
                let order = if d_order == 0 {
                    exercise.order + 1
                } else {
                    exercise.order - 1
                };
                d_order += 1;

                order 
            } else {
                let order = exercise.order + d_order;

                if exercise.order == exercise_for_u_order.order {
                    d_order = 0;
                }

                order
            };

            let new_order = ExerciseForChangeOrder {
                id: exercise.id,
                order,
            };

            result.push(new_order);
        }

        Ok(result)
    }
}
