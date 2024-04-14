use crate::query_repository::{course::CourseQueryRepository, user::UserQueryRepository};

use super::{db_manager::DbManager, error::DbResult};

#[derive(Clone)]
pub struct QueryRepositoryManager {
    user_repository: UserQueryRepository,
    course_repository: CourseQueryRepository,
}

impl QueryRepositoryManager {
    pub async fn new() -> DbResult<Self> {
        let dbm = DbManager::new().await?;

        let user_repository = UserQueryRepository::new(dbm.clone());
        let course_repository = CourseQueryRepository::new(dbm.clone());

        let result = Self {
            user_repository,
            course_repository,
        };

        Ok(result)
    }

    pub fn get_user_repository(&self) -> UserQueryRepository {
        self.user_repository.clone()
    }

    pub fn get_course_repository(&self) -> CourseQueryRepository {
        self.course_repository.clone()
    }
}