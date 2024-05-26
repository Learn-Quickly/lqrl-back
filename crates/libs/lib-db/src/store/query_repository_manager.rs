use crate::query_repository::{course::CourseQueryRepository, lesson::LessonQueryRepository, lesson_progress::LessonProgressQueryRepository, user::UserQueryRepository};

use super::{db_manager::DbManager, error::DbResult};

#[derive(Clone)]
pub struct QueryRepositoryManager {
    user_repository: UserQueryRepository,
    course_repository: CourseQueryRepository,
    lesson_repository: LessonQueryRepository,
    lesson_progress_repository: LessonProgressQueryRepository,
}

impl QueryRepositoryManager {
    pub async fn new() -> DbResult<Self> {
        let dbm = DbManager::new().await?;

        let user_repository = UserQueryRepository::new(dbm.clone());
        let course_repository = CourseQueryRepository::new(dbm.clone());
        let lesson_repository = LessonQueryRepository::new(dbm.clone());
        let lesson_progress_repository = LessonProgressQueryRepository::new(dbm.clone());

        let result = Self {
            user_repository,
            course_repository,
            lesson_repository,
            lesson_progress_repository,
        };

        Ok(result)
    }

    pub fn get_user_repository(&self) -> UserQueryRepository {
        self.user_repository.clone()
    }

    pub fn get_course_repository(&self) -> CourseQueryRepository {
        self.course_repository.clone()
    }

    pub fn get_lesson_repository(&self) -> LessonQueryRepository {
        self.lesson_repository.clone()
    }

    pub fn get_lesson_progress_repository(&self) -> LessonProgressQueryRepository {
        self.lesson_progress_repository.clone()
    }
}