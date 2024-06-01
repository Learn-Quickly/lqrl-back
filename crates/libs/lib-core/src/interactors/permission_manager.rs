use std::sync::Arc;

use crate::{ctx::Ctx, interfaces::command_repository_manager::ICommandRepositoryManager};

use super::error::CoreError;

pub type PermissionResult<T> = core::result::Result<T, CoreError>;

pub struct PermissionManager {
    repository_manager: Arc<dyn ICommandRepositoryManager + Send + Sync>,
}

impl PermissionManager {
    pub fn new(repository_manager: Arc<dyn ICommandRepositoryManager + Send + Sync>) -> Self {
        Self {
            repository_manager,
        }
    }
}

impl PermissionManager {
    pub async fn check_course_creator_permission(
        &self, 
        ctx: &Ctx,
        course_id: i64,
    ) -> PermissionResult<()> {
        let user_id = ctx.user_id();

        // admin
        if user_id == 1000 {
            return Ok(());
        }

        let course_repository = self.repository_manager.get_course_repository();
        let user_course = course_repository.get_user_course(ctx, ctx.user_id(), course_id).await?;
        if !user_course.user_role.eq(&crate::models::course::UserCourseRole::Creator) {
            return Err(CoreError::PermissionDenied);
        }

        Ok(())
    }

    pub async fn check_course_student_permission(
        &self, 
        ctx: &Ctx,
        course_id: i64,
    ) -> PermissionResult<()> {
        let user_id = ctx.user_id();

        // admin
        if user_id == 1000 {
            return Ok(());
        }

        let course_repository = self.repository_manager.get_course_repository();
        course_repository
            .get_user_course(ctx, ctx.user_id(), course_id)
            .await
            .map_err(|_| CoreError::PermissionDenied)?;

        Ok(())
    }

    pub async fn check_lesson_creator_permission(
        &self, 
        ctx: &Ctx,
        lesson_id: i64,
    ) -> PermissionResult<()> {
        let lesson_repository = self.repository_manager.get_lesson_repository();
        let lesson = lesson_repository.get_lesson(ctx, lesson_id).await?;

        self.check_course_creator_permission(ctx, lesson.course_id).await
    }

    pub async fn check_lesson_student_permission(
        &self, 
        ctx: &Ctx,
        lesson_id: i64,
    ) -> PermissionResult<()> {
        let lesson_repository = self.repository_manager.get_lesson_repository();
        let lesson = lesson_repository.get_lesson(ctx, lesson_id).await?;

        self.check_course_student_permission(ctx, lesson.course_id).await
    }

    pub async fn check_exercise_creator_permission(
        &self,
        ctx: &Ctx,
        exercise_id: i64,
    ) -> PermissionResult<()> {
        let exercise_repository = self.repository_manager.get_exercise_repository();
        let exercise = exercise_repository.get_exercise(ctx, exercise_id).await?;

        let lesson_repository = self.repository_manager.get_lesson_repository();
        let lesson = lesson_repository.get_lesson(ctx, exercise.lesson_id).await?;

        self.check_lesson_creator_permission(ctx, lesson.id).await
    }

    pub async fn check_exercise_student_permission(
        &self,
        ctx: &Ctx,
        exercise_id: i64,
    ) -> PermissionResult<()> {
        let exercise_repository = self.repository_manager.get_exercise_repository();
        let exercise = exercise_repository.get_exercise(ctx, exercise_id).await?;

        let lesson_repository = self.repository_manager.get_lesson_repository();
        let lesson = lesson_repository.get_lesson(ctx, exercise.lesson_id).await?;

        self.check_lesson_student_permission(ctx, lesson.id).await
    }
} 
