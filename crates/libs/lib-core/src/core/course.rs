use std::sync::Arc;

use crate::{ctx::Ctx, interfaces::{command_repository_manager::ICommandRepositoryManager, course::CourseResult}, model::course::{CourseForCreate, CourseForUpdate, CourseForUpdateCommand, UserCourse}};

use super::{error::CourseError, img_file::{remove_file, upload_file}, permission_manager::PermissionManager};

pub struct CourseInteractor {
    permission_manager: PermissionManager,
    repository_manager: Arc<dyn ICommandRepositoryManager + Send + Sync>,
}

impl CourseInteractor {
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

impl CourseInteractor {
    pub async fn create_draft(
        &self,
        ctx: &Ctx,
        course_c: CourseForCreate,
    ) -> CourseResult<i64> {
        let course_repository = self.repository_manager.get_course_repository();
        course_repository.create_draft(ctx, course_c).await
    }

    pub async fn update_course(
        &self,
        ctx: &Ctx,
        course_for_u: CourseForUpdate,
        course_id: i64,
    ) -> CourseResult<()> {
        self.permission_manager.check_course_creator_permission(ctx, course_id).await?;

        let command = CourseForUpdateCommand {
            title: course_for_u.title,
            description: course_for_u.description,
            course_type: course_for_u.course_type,
            price: course_for_u.price,
            color: course_for_u.color,
            img_url: None,
            published_date: None,
            state: None,
        };

        let course_repository = self.repository_manager.get_course_repository();
        course_repository.update_course(ctx, command, course_id).await
    }

    pub async fn set_course_img(
        &self,
        ctx: &Ctx,
        course_id: i64,
        file_data: &[u8],
        file_path: &str,
    ) -> CourseResult<String> {
        self.permission_manager.check_course_creator_permission(ctx, course_id).await?;

        let course_repository = self.repository_manager.get_course_repository();

        let course = course_repository.get_course(ctx, course_id).await?;

        let new_img_url = upload_file(file_path, file_data).await?;

        let course_for_u = CourseForUpdateCommand::builder()
            .img_url(new_img_url.clone())
            .build();

        course_repository.update_course(ctx, course_for_u, course_id).await?;

        let outdated_img_url = course.img_url;

        if let Some(img_ulr) = outdated_img_url {
            remove_file(img_ulr).await?;
        }

        Ok(new_img_url)
    }

    pub async fn publish_course(
        &self, 
        ctx: &Ctx,
        course_id: i64,
    ) -> CourseResult<()> {
        self.permission_manager.check_course_creator_permission(ctx, course_id).await?;

        let command = CourseForUpdateCommand::builder()
            .state(crate::model::course::CourseState::Published)
            .build();

        let course_repository = self.repository_manager.get_course_repository();
        course_repository.update_course(ctx, command, course_id).await
    }

    pub async fn archive_course(
        &self,
        ctx: &Ctx,
        course_id: i64,
    ) -> CourseResult<()> {
        self.permission_manager.check_course_creator_permission(ctx, course_id).await?;

        let command = CourseForUpdateCommand::builder()
            .state(crate::model::course::CourseState::Archived)
            .build();

        let course_repository = self.repository_manager.get_course_repository();
        course_repository.update_course(ctx, command, course_id).await
    }

    pub async fn register_for_course(
        &self,
        ctx: &Ctx,
        course_id: i64,
    ) -> CourseResult<()> {
        let course_repository = self.repository_manager.get_course_repository();
        let course = course_repository.get_course(ctx, course_id).await?;

        if !course.state.eq(&crate::model::course::CourseState::Published) {
            return Err(CourseError::CourseMustBePublishedError.into())
        }

        let user_course = course_repository
            .get_user_course_optional(ctx, ctx.user_id(), course_id)
            .await?;

        if let Some(user_course) = user_course {
            if user_course.user_role.eq(&crate::model::course::UserCourseRole::Creator) {
                return Err(CourseError::CreatorCannotSubscribeToTheCourse.into());
            }
            
            return Err(CourseError::CannotRegisterForCourseTwice.into());
        }

        let course_for_register = UserCourse {
            user_id: ctx.user_id(),
            course_id,
            user_role: crate::model::course::UserCourseRole::Student,
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
