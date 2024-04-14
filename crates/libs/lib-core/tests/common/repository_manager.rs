use lib_core::{interfaces::{command_repository_manager::ICommandRepositoryManager, course::ICourseCommandRepository, lesson::ILessonCommandRepository, user::IUserCommandRepository}, models::lesson::Lesson};

use super::repository::{CourseCommandRepositoryMock, LessonCommandRepositoryMock, UserCommandRepositoryMock};

pub struct CommandRepositoryManagerMock {
    user_repository: UserCommandRepositoryMock,
    course_repository: CourseCommandRepositoryMock,
    lesson_repository: LessonCommandRepositoryMock,
}

impl CommandRepositoryManagerMock {
    pub fn new(lessons: Vec<Lesson>) -> Self {
        let lesson_repository = LessonCommandRepositoryMock::new(lessons);
        let user_repository = UserCommandRepositoryMock;
        let course_repository = CourseCommandRepositoryMock;

        Self {
            user_repository,
            course_repository,
            lesson_repository,
        }
    }
}

impl ICommandRepositoryManager for CommandRepositoryManagerMock {
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