use async_trait::async_trait;

use crate::{ctx::Ctx, interactors::error::CoreError, models::exercise::{Exercise, ExerciseForChangeOreder, ExerciseForCreateCommand, ExerciseForUpdate}};

pub type ExerciseResult<T> = core::result::Result<T, CoreError>;

#[async_trait]
pub trait IExerciseCommandRepository {
    async fn get_lesson_exercises_ordered(&self, ctx: &Ctx, lesson_id: i64) -> ExerciseResult<Vec<ExerciseForChangeOreder>>;
    async fn get_exercise(&self, ctx: &Ctx, exercise_id: i64) -> ExerciseResult<Exercise>;
    async fn create(&self, ctx: &Ctx, exercise_c: ExerciseForCreateCommand) -> ExerciseResult<i64>;
    async fn update(&self, ctx: &Ctx, exercise_u: ExerciseForUpdate) -> ExerciseResult<()>;
}