use async_trait::async_trait;

use crate::{ctx::Ctx, interactors::error::CoreError, models::{exercise::{Exercise, ExerciseForChangeOrder, ExerciseForCreateCommand, ExerciseForUpdate}, exercise_completion::{ExerciseCompletion, ExerciseCompletionForCompleteCommand, ExerciseCompletionForCreate, ExerciseCompletionForUpdate}}};

pub type ExerciseResult<T> = core::result::Result<T, CoreError>;

#[async_trait]
pub trait IExerciseCommandRepository {
    async fn get_lesson_exercises_ordered(&self, ctx: &Ctx, lesson_id: i64) -> ExerciseResult<Vec<ExerciseForChangeOrder>>;

    async fn get_exercise(&self, ctx: &Ctx, exercise_id: i64) -> ExerciseResult<Exercise>;

    async fn create(&self, ctx: &Ctx, exercise_c: ExerciseForCreateCommand) -> ExerciseResult<i64>;

    async fn update(&self, ctx: &Ctx, exercise_u: ExerciseForUpdate) -> ExerciseResult<()>;

    async fn update_exercise_orders(&self, ctx: &Ctx, lesson_exercises: Vec<ExerciseForChangeOrder>) -> ExerciseResult<()>;

    async fn get_exercise_user_completions(&self, ctx: &Ctx, user_id: i64, exercise_id: i64) -> ExerciseResult<Vec<ExerciseCompletion>>;

    async fn create_exercise_completion(&self, ctx: &Ctx, ex_comp_for_c: ExerciseCompletionForCreate) -> ExerciseResult<i64>;

    async fn get_exercise_completion(&self, ctx: &Ctx, ex_comp_id: i64) -> ExerciseResult<ExerciseCompletion>;

    async fn update_exercise_completion(&self, ctx: &Ctx, ex_comp_for_u: ExerciseCompletionForUpdate) -> ExerciseResult<()>;

    async fn complete_exercise_completion(&self, ctx: &Ctx, ex_comp_for_u: ExerciseCompletionForCompleteCommand) -> ExerciseResult<()>;

    async fn get_uncompleted_exercises(&self, ctx: &Ctx) -> ExerciseResult<Vec<ExerciseCompletion>>;

    async fn get_number_of_lesson_completed_exercises(&self, ctx: &Ctx, lesson_id: i64, user_id: i64) -> ExerciseResult<i64>;

    async fn remove_exercise_completions(&self, ctx: &Ctx, exercise_id: i64) -> ExerciseResult<()>;

}