use async_trait::async_trait;

use crate::{ctx::Ctx, interactors::error::CoreError, models::exercise::{ExerciseForChangeOreder, ExerciseForCreateCommand}};

pub type ExerciseResult<T> = core::result::Result<T, CoreError>;

#[async_trait]
pub trait IExerciseCommandRepository {
    async fn get_lesson_exercises_ordered(&self, ctx: &Ctx, lesson_id: i64) -> ExerciseResult<Vec<ExerciseForChangeOreder>>;
    async fn create(&self, ctx: &Ctx, exercise_c: ExerciseForCreateCommand) -> ExerciseResult<i64>;
}