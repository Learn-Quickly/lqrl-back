use crate::{ctx::Ctx, interfaces::course::{ICourseRepository, CourseResult}, model::course::{CourseForCreate, CourseForUpdate, UserCourse}};

use super::{error::CoreError, img_file::{remove_file, upload_file}};

pub struct CourseController {
    ctx: Ctx,
    repository: Box<dyn ICourseRepository + Send + Sync>,
}

impl CourseController {
    pub fn new(ctx: Ctx, repository: Box<dyn ICourseRepository + Send + Sync>) -> Self {
        Self {
            ctx,
            repository,
        }
    }
}

impl CourseController {
    pub async fn create_draft(
        &self,
        course_c: CourseForCreate,
    ) -> CourseResult<i64> {
        self.repository.create_draft(&self.ctx, course_c).await
    }

    pub async fn update_course(
        &self,
        course_for_u: CourseForUpdate,
        course_id: i64,
    ) -> CourseResult<()> {
        self.repository.update_course(&self.ctx, course_for_u, course_id).await
    }

    pub async fn set_course_img(
        &self,
        course_id: i64,
        file_data: &[u8],
        file_path: &str,
    ) -> CourseResult<String> {
        let course = self.repository.get_course(&self.ctx, course_id).await?;
        let user_course = self.repository.get_user_course(&self.ctx, self.ctx.user_id(), course_id).await?;
        if !user_course.user_role.eq(&crate::model::course::UserCourseRole::Creator) {
            return Err(Box::new(CoreError::PermissionDenied));
        }

        let new_img_url = upload_file(file_path, file_data).await?;

        let course_for_u = CourseForUpdate::builder()
            .img_url(new_img_url.clone())
            .build();

        self.repository.update_course(&self.ctx, course_for_u, course_id).await?;

        let outdated_img_url = course.img_url;

        if let Some(img_ulr) = outdated_img_url {
            remove_file(img_ulr).await?;
        }

        Ok(new_img_url)
    }

    pub async fn publish_course(
        &self, 
        course_id: i64,
    ) -> CourseResult<()> {
        self.repository.publish_course(&self.ctx, course_id).await
    }

    pub async fn archive_course(
        &self,
        course_id: i64,
    ) -> CourseResult<()> {
        self.repository.archive_course(&self.ctx, course_id).await
    }

    pub async fn register_for_course(
        &self,
        course_id: i64,
    ) -> CourseResult<()> {
        let course = self.repository.get_course(&self.ctx, course_id).await?;

        if !course.state.eq(&crate::model::course::CourseState::Published) {
            return Err(Box::new(CoreError::CourseMustBePublishedError))
        }

        let user_course = self.repository.get_user_course_optional(&self.ctx, self.ctx.user_id(), course_id).await?;
        if let Some(user_course) = user_course {
            if user_course.user_role.eq(&crate::model::course::UserCourseRole::Creator) {
                return Err(Box::new(CoreError::CreatorCannotSubscribeToTheCourse));
            }
            
            return Err(Box::new(CoreError::CannotRegisterForCourseTwice));
        }

        let course_for_register = UserCourse {
            user_id: self.ctx.user_id(),
            course_id,
            user_role: crate::model::course::UserCourseRole::Student,
        };

        self.repository.create_user_course(&self.ctx, course_for_register).await?;

        Ok(())
    }

    pub async fn unsubscribe_from_course(
        &self,
        course_id: i64
    ) -> CourseResult<()> {
        self.repository.delete_user_course(&self.ctx, self.ctx.user_id(), course_id).await
    }
}
