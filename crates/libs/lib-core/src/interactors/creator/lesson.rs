use std::sync::Arc;

use crate::{ctx::Ctx, interactors::{error::LessonError, permission_manager::PermissionManager}, interfaces::{command_repository_manager::ICommandRepositoryManager, lesson::LessonResult}, model::lesson::{LessonForChangeOreder, LessonForCreate, LessonForCreateCommand, LessonForUpdate}};


pub struct CreatorLessonInteractor {
    permission_manager: PermissionManager,
    repository_manager: Arc<dyn ICommandRepositoryManager + Send + Sync>,
}

impl CreatorLessonInteractor {
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

impl CreatorLessonInteractor {
    pub async fn create_lesson(
        &self, 
        ctx: &Ctx,
        lesson: LessonForCreate
    ) -> LessonResult<i64> {
        self.permission_manager
            .check_course_creator_permission(ctx, lesson.course_id)
            .await?;

        let lesson_repository = self.repository_manager.get_lesson_repository();

        let course_lessons = lesson_repository
            .get_course_lessons_ordered(ctx, lesson.course_id)
            .await?;

        let order = course_lessons.len() + 1;

        let lesson_for_c = LessonForCreateCommand {
            course_id: lesson.course_id,
            title: lesson.title,
            description: lesson.description,
            order: order as i32,
        };

        lesson_repository.create_lesson(ctx, lesson_for_c).await
    }

    pub async fn delete_lesson(
        &self, 
        ctx: &Ctx,
        lesson_id: i64
    ) -> LessonResult<()> {
        self.permission_manager
            .check_lesson_creator_permission(ctx, lesson_id)
            .await?;

        let lesson_repository = self.repository_manager.get_lesson_repository();
        lesson_repository.delete_lesson(ctx, lesson_id).await
    }

    pub async fn update_lesson(
        &self,
        ctx: &Ctx, 
        lesson_for_u: LessonForUpdate
    ) -> LessonResult<()> {
        self.permission_manager
            .check_lesson_creator_permission(ctx, lesson_for_u.id)
            .await?;

        let lesson_repository = self.repository_manager.get_lesson_repository();
        lesson_repository.update_lesson(ctx, lesson_for_u).await
    }

    pub async fn change_order(
        &self, 
        ctx: &Ctx,
        lesson_for_u_order: LessonForChangeOreder
    ) -> LessonResult<()> {
        self.permission_manager
            .check_lesson_creator_permission(ctx, lesson_for_u_order.id)
            .await?;

        let lesson_repository = self.repository_manager.get_lesson_repository();

        let course_lesson = lesson_repository.get_lesson(ctx, lesson_for_u_order.id).await?;
        let course_lessons = lesson_repository
            .get_course_lessons_ordered(ctx, course_lesson.course_id)
            .await?;

        let course_lessons = self.compute_orders(&course_lessons, &lesson_for_u_order)?;
        lesson_repository.update_lesson_orders(ctx, course_lessons).await?;

        Ok(())
    }

    fn compute_orders(
        &self,
        lessons: &Vec<LessonForChangeOreder>, 
        lesson_for_u_order: &LessonForChangeOreder
    ) -> LessonResult<Vec<LessonForChangeOreder>> {
        let number_of_lessons = lessons.len() as i32;
        if number_of_lessons < lesson_for_u_order.order {
            return Err(
                LessonError::IncorrectLessonOreder { 
                    lesson_id: lesson_for_u_order.id,
                    order: lesson_for_u_order.order 
                }.into()
            );
        }

        let mut result = Vec::new();
        let mut d_order = 0;

        for lesson in lessons {
            let order = if lesson.id == lesson_for_u_order.id {
                d_order = if d_order != 0 {
                    0
                } else {
                    -1
                };

                lesson_for_u_order.order
            } else if lesson.order == lesson_for_u_order.order {
                let order = if d_order == 0 {
                    lesson.order + 1
                } else {
                    lesson.order - 1
                };
                d_order += 1;

                order 
            } else {
                let order = lesson.order + d_order;

                if lesson.order == lesson_for_u_order.order {
                    d_order = 0;
                }

                order
            };

            let new_order = LessonForChangeOreder {
                id: lesson.id,
                order,
            };

            result.push(new_order);
        }

        Ok(result)
    }
}
