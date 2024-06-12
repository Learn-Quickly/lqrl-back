use async_trait::async_trait;

use crate::{
    ctx::Ctx, interactors::error::CoreError, 
    models::{
        lesson::{
            Lesson, LessonForChangeOreder, LessonForCreateCommand, LessonForUpdate}, 
            lesson_progress::{LessonProgress, LessonProgressState}
    }
};

pub type LessonResult<T> = core::result::Result<T, CoreError>;

#[async_trait]
pub trait ILessonCommandRepository {
    async fn get_lesson(&self, ctx: &Ctx, lesson_id: i64) -> LessonResult<Lesson>;

    async fn change_lesson_progress_states_for_update_exercise(&self, ctx: &Ctx, lesson_id: i64, order: i32) -> LessonResult<()>;

    async fn get_course_lessons_ordered(&self, ctx: &Ctx, course_id: i64) -> LessonResult<Vec<LessonForChangeOreder>>;

    async fn create_lesson_progress(&self, ctx: &Ctx, lesson_id: i64, user_id: i64) -> LessonResult<()>;
    
    async fn get_lessons_progresses(&self, ctx: &Ctx, course_id: i64, user_id: i64) -> LessonResult<Vec<LessonProgress>>;

    async fn create_lesson(&self, ctx: &Ctx, lesson_for_c: LessonForCreateCommand) -> LessonResult<i64>;

    async fn delete_lesson(&self, ctx: &Ctx, lesson_id: i64) -> LessonResult<()>;

    async fn update_lesson(&self, ctx: &Ctx, lesson_for_u: LessonForUpdate) -> LessonResult<()>;

    async fn update_lesson_progress_state(&self, ctx: &Ctx, lesson_for_u: LessonProgressState, lesson_id: i64, user_id: i64) -> LessonResult<()>;
    
    async fn update_lesson_orders(&self, ctx: &Ctx, lessons_for_u_order: Vec<LessonForChangeOreder>) -> LessonResult<()>;
}