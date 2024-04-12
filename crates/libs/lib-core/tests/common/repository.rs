use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use lib_core::{core::error::{CoreError, LessonError}, ctx::Ctx, interfaces::{course::{CourseResult, ICourseCommandRepository}, lesson::{ILessonCommandRepository, LessonResult}, user::{IUserCommandRepository, UserResult}}, model::{course::{Course, CourseForCreate, CourseForUpdateCommand, UserCourse, UserCourseRole}, lesson::{Lesson, LessonForChangeOreder, LessonForCreateCommand, LessonForUpdate}, user::{User, UserForCreate, UserForUpdate}}};

#[derive(Clone)]
pub struct CourseCommandRepositoryMock;

#[async_trait]
impl ICourseCommandRepository for CourseCommandRepositoryMock {
    async fn get_course(&self, _: &Ctx, _: i64) -> CourseResult<Course> { panic!() }
    async fn create_draft(&self, _: &Ctx, _: CourseForCreate) -> CourseResult<i64> { panic!() }
    async fn update_course(&self, _: &Ctx, _: CourseForUpdateCommand, _: i64) -> CourseResult<()> { panic!() }
    async fn create_user_course(&self, _: &Ctx, _: UserCourse) -> CourseResult<()> { panic!() }
    async fn get_user_course(&self, _: &Ctx, user_id: i64, course_id: i64) -> CourseResult<UserCourse> { 
        let res = UserCourse { 
            user_id, 
            course_id, 
            user_role: UserCourseRole::Creator
        };
        Ok(res)
    }
    async fn get_user_course_optional(&self, _: &Ctx, _: i64, _: i64) -> CourseResult<Option<UserCourse>> { panic!() }
    async fn delete_user_course(&self, _: &Ctx, _: i64, _: i64) -> CourseResult<()> { panic!() }
}

#[derive(Clone)]
pub struct UserCommandRepositoryMock;

#[async_trait]
impl IUserCommandRepository for UserCommandRepositoryMock {
    async fn get_user(&self, _: &Ctx, _: i64) -> UserResult<User> { panic!() }
    async fn create_user(&self, _: &Ctx, _: UserForCreate) -> UserResult<i64> { panic!() }
    async fn update_user(&self, _: &Ctx, _: UserForUpdate) -> UserResult<()> { panic!() }
    async fn update_pwd(&self, _: &Ctx, _: i64, _: String) -> UserResult<()> { panic!() }
}
    
#[derive(Clone)]
pub struct LessonCommandRepositoryMock {
    pub lessons: Arc<Mutex<Vec<Lesson>>>,
}

impl LessonCommandRepositoryMock {
    pub fn new(lessons: Vec<Lesson>) -> Self {
        Self {
            lessons: Arc::new(Mutex::new(lessons)),
        }
    }
}

#[async_trait]
impl ILessonCommandRepository for LessonCommandRepositoryMock {
    async fn get_lesson(&self, _: &Ctx, lesson_id: i64) -> LessonResult<Lesson> {
        let read_lesson = self.lessons
            .lock()
            .unwrap()
            .clone();

        let lesson = read_lesson
            .iter()
            .find(|lesson| lesson.id == lesson_id)
            .ok_or(
                CoreError::LessonError(LessonError::LessonNotFound { lesson_id })
            )?;
            
        Ok(lesson.clone())
    }

    async fn get_course_lessons_ordered(
        &self,
        _: &Ctx, 
        _: i64
    ) -> LessonResult<Vec<LessonForChangeOreder>> {
        let mut result = self.lessons.lock().unwrap().clone();

        result.sort_by(|a, b| a.lesson_order.cmp(&b.lesson_order));

        let result = result.iter().map(|lesson| LessonForChangeOreder { 
            id: lesson.id, 
            order:  lesson.lesson_order,
        }).collect();

        Ok(result)
    }

    async fn update_lesson_orders(
        &self, 
        _: &Ctx, 
        lessons_for_u_order: Vec<LessonForChangeOreder>
    ) -> LessonResult<()> {
        let mut result = self.lessons.lock().unwrap().clone();

        for lesson_order in &lessons_for_u_order {
            for lesson in &mut result {
                if lesson.id == lesson_order.id {
                    lesson.lesson_order = lesson_order.order;
                    break;
                }
            }
        }

        self.lessons.lock().unwrap().clear();
        self.lessons.lock().unwrap().extend(result.into_iter());

        Ok(())
    }

    async fn create_lesson(&self, _: &Ctx, _: LessonForCreateCommand) -> LessonResult<i64> {
        panic!()
    }

    async fn delete_lesson(&self, _: &Ctx, _: i64) -> LessonResult<()> {
        panic!()
    }

    async fn update_lesson(&self, _: &Ctx, _: LessonForUpdate) -> LessonResult<()> {
        panic!()
    }
}