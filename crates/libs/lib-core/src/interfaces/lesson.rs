use async_trait::async_trait;

use crate::{core::error::CoreError, ctx::Ctx, model::lesson::{Lesson, LessonForChangeOreder, LessonForCreateCommand, LessonForUpdate}};


pub type LessonResult<T> = core::result::Result<T, CoreError>;

#[async_trait]
pub trait ILessonCommandRepository {
    async fn get_lesson(&self, ctx: &Ctx, lesson_id: i64) -> LessonResult<Lesson>;

    async fn get_course_lessons_ordered(&self, ctx: &Ctx, course_id: i64) -> LessonResult<Vec<LessonForChangeOreder>>;

    async fn create_lesson(&self, ctx: &Ctx, lesson_for_c: LessonForCreateCommand) -> LessonResult<i64>;

    async fn delete_lesson(&self, ctx: &Ctx, lesson_id: i64) -> LessonResult<()>;

    async fn update_lesson(&self, ctx: &Ctx, lesson_for_u: LessonForUpdate) -> LessonResult<()>;
    
    async fn update_lesson_orders(&self, ctx: &Ctx, lessons_for_u_order: Vec<LessonForChangeOreder>) -> LessonResult<()>;
}