use std::sync::Arc;

use crate::{
    ctx::Ctx,
    interactors::{
        error::CourseError, 
        permission_manager::PermissionManager
    }, 
    interfaces::{command_repository_manager::ICommandRepositoryManager, course::CourseResult}, 
    models::course::UserCourse
};

pub struct StudentCourseInteractor {
    permission_manager: PermissionManager,
    repository_manager: Arc<dyn ICommandRepositoryManager + Send + Sync>,
}

impl StudentCourseInteractor {
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

impl StudentCourseInteractor {
    pub async fn register_for_course(
        &self,
        ctx: &Ctx,
        course_id: i64,
    ) -> CourseResult<()> {
        let course_repository = self.repository_manager.get_course_repository();
        let course = course_repository.get_course(ctx, course_id).await?;

        if !course.state.eq(&crate::models::course::CourseState::Published) {
            return Err(CourseError::CourseMustBePublishedError.into())
        }

        let user_course = course_repository
            .get_user_course_optional(ctx, ctx.user_id(), course_id)
            .await?;

        if let Some(user_course) = user_course {
            if user_course.user_role.eq(&crate::models::course::UserCourseRole::Creator) {
                return Err(CourseError::CreatorCannotSubscribeToTheCourse.into());
            }
            
            return Err(CourseError::CannotRegisterForCourseTwice.into());
        }

        let course_for_register = UserCourse {
            user_id: ctx.user_id(),
            course_id,
            user_role: crate::models::course::UserCourseRole::Student,
        };

        course_repository.create_user_course(ctx, course_for_register).await?;

        Ok(())
    }

    pub async fn unsubscribe_from_course(
        &self,
        ctx: &Ctx,
        course_id: i64
    ) -> CourseResult<()> {
        let course_repository = self.repository_manager.get_course_repository();
        course_repository.delete_user_course(ctx, ctx.user_id(), course_id).await
    }
}
