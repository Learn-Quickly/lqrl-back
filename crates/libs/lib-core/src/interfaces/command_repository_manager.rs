use super::{course::ICourseCommandRepository, lesson::ILessonCommandRepository, user::IUserCommandRepository};

pub trait ICommandRepositoryManager {
    fn get_user_repository(&self) -> Box<dyn IUserCommandRepository + Send + Sync>;

    fn get_course_repository(&self) -> Box<dyn ICourseCommandRepository + Send + Sync>;

    fn get_lesson_repository(&self) -> Box<dyn ILessonCommandRepository + Send + Sync>;
}