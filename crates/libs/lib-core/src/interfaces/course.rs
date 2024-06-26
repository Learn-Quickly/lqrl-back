use async_trait::async_trait;

use crate::{interactors::error::CoreError, ctx::Ctx, models::course::{Course, CourseForCreate, CourseForUpdateCommand, UserCourse}};

pub type CourseResult<T> = core::result::Result<T, CoreError>;

#[async_trait]
pub trait ICourseCommandRepository {
    async fn get_course(&self, ctx: &Ctx, course_id: i64) -> CourseResult<Course>;

    async fn get_user_course(&self, ctx: &Ctx, user_id: i64, course_id: i64) -> CourseResult<UserCourse>;

    async fn get_user_course_optional(&self, ctx: &Ctx, user_id: i64, course_id: i64) -> CourseResult<Option<UserCourse>>;

    async fn create_draft(&self, ctx: &Ctx, course_c: CourseForCreate) -> CourseResult<i64>;

    async fn update_course(&self, ctx: &Ctx, course_for_u: CourseForUpdateCommand, course_id: i64) -> CourseResult<()>;

    async fn create_user_course(&self, ctx: &Ctx, course_for_r: UserCourse) -> CourseResult<()>;

    async fn delete_user_course(&self, ctx: &Ctx, user_id: i64, course_id: i64) -> CourseResult<()>;
}
