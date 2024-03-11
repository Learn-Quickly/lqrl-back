use crate::{ctx::Ctx, interfaces::course::{ICourseRepository, CourseResult}, model::course::{CourseForCreate, CourseForUpdate, UserCourse}};

use super::error::CoreError;

pub struct CourseController;

impl CourseController {
    pub async fn create_draft(
        ctx: &Ctx,
        repository: Box<dyn ICourseRepository>, 
        course_c: CourseForCreate
    ) -> CourseResult<i64> {
        repository.create_draft(&ctx, course_c).await
    }

    pub async fn update_course(
        ctx: &Ctx,
        repository: Box<dyn ICourseRepository>, 
        course_for_u: CourseForUpdate,
        course_id: i64,
    ) -> CourseResult<()> {
        repository.update_course(&ctx, course_for_u, course_id).await
    }

    pub async fn publish_course(
        ctx: &Ctx,
        repository: Box<dyn ICourseRepository>, 
        course_id: i64,
    ) -> CourseResult<()> {
        repository.publish_course(&ctx, course_id).await
    }

    pub async fn archive_course(
        ctx: &Ctx,
        repository: Box<dyn ICourseRepository>, 
        course_id: i64,
    ) -> CourseResult<()> {
        repository.archive_course(&ctx, course_id).await
    }

    pub async fn register_for_course(
        ctx: &Ctx,
        repository: Box<dyn ICourseRepository>, 
        course_for_register: UserCourse,
    ) -> CourseResult<()> {
        let course = repository.get_course(&ctx, course_for_register.course_id).await?;

        if !course.state.eq(&crate::model::course::CourseState::Published) {
            return Err(Box::new(CoreError::CourseMustBePublishedError))
        }

        repository.create_user_course(&ctx, course_for_register).await?;

        Ok(())
    }
}

