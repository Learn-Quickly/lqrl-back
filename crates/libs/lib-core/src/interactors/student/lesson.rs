use std::sync::Arc;

use crate::{ctx::Ctx, interactors::{error::CoreError, permission_manager::PermissionManager}, interfaces::{command_repository_manager::ICommandRepositoryManager, lesson::LessonResult}};


pub struct StudentLessonInteractor {
    permission_manager: PermissionManager,
    repository_manager: Arc<dyn ICommandRepositoryManager + Send + Sync>,
}

impl StudentLessonInteractor {
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

impl StudentLessonInteractor { 
    pub async fn start_lesson(
        &self,
        ctx: &Ctx,
        lesson_id: i64,
    ) -> LessonResult<()> {
        self.permission_manager.check_lesson_student_permission(ctx, lesson_id).await?;
        let lesson_repository = self.repository_manager.get_lesson_repository();
        let lesson = lesson_repository.get_lesson(ctx, lesson_id).await?;

        if lesson.lesson_order == 1 {
            lesson_repository.create_lesson_progress(ctx, lesson_id, ctx.user_id()).await?;

            return Ok(());
        }

        let lessons_ordered = lesson_repository.get_course_lessons_ordered(ctx, lesson.course_id).await?;
        let lesson_progresses = lesson_repository.get_lessons_progresses(ctx, lesson.course_id, ctx.user_id()).await?;

        let previus_lesson_id = match lessons_ordered.get((lesson.lesson_order - 2) as usize) {
            Some(prev_less) => prev_less.id,
            None => return Err(CoreError::LessonError(crate::interactors::error::LessonError::PreviousLessonNotFound { lesson_id })),
        };

        let prev_lesson_progress = match lesson_progresses.iter().find(|lesson_progress| lesson_progress.lesson_id == previus_lesson_id) {
            Some(prev_l_progress) => prev_l_progress,
            None => return Err(CoreError::LessonError(crate::interactors::error::LessonError::PreviousLessonNotFound { lesson_id })),
        };

        if prev_lesson_progress.state.ne(&crate::models::lesson_progress::LessonProgressState::Done) {
            return Err(CoreError::LessonError(crate::interactors::error::LessonError::PreviousLessonNotCompleted { lesson_id }));
        }

        lesson_repository.create_lesson_progress(ctx, lesson_id, ctx.user_id()).await?;
        
        Ok(())
    }
}