use lib_core::interfaces::{command_repository_manager::ICommandRepositoryManager, course::ICourseCommandRepository, lesson::ILessonCommandRepository, user::IUserCommandRepository};

use crate::command_repository::{course::CourseCommandRepository, lesson::LessonCommandRepository, user::UserCommandRepository};

use super::{db_manager::DbManager, error::DbResult};

pub struct CommandRepositoryManager {
    user_repository: UserCommandRepository,
    course_repository: CourseCommandRepository,
    lesson_repository: LessonCommandRepository,
}

impl CommandRepositoryManager {
    pub async fn new() -> DbResult<Self> {
        let dbm = DbManager::new().await?;

        let user_repository = UserCommandRepository::new(dbm.clone());
        let course_repository = CourseCommandRepository::new(dbm.clone());
        let lesson_repository = LessonCommandRepository::new(dbm);

        let result = Self {
            user_repository,
            course_repository,
            lesson_repository,
        };

        Ok(result)
    }
}

impl<'a> ICommandRepositoryManager for CommandRepositoryManager {
    fn get_user_repository(&self) -> Box<dyn IUserCommandRepository + Send + Sync> {
        Box::new(self.user_repository.clone())
    }

    fn get_course_repository(&self) -> Box<dyn ICourseCommandRepository + Send + Sync> {
        Box::new(self.course_repository.clone())
    }

    fn get_lesson_repository(&self) -> Box<dyn ILessonCommandRepository + Send + Sync> {
        Box::new(self.lesson_repository.clone())
    }
}