use crate::{ctx::Ctx, interfaces::lesson::{ILessonCommandRepository, LessonResult}, model::lesson::{LessonForChangeOreder, LessonForCreate, LessonForCreateCommand, LessonForUpdate}};

use super::error::LessonError;


pub struct LessonInteractor<'a> {
    ctx: &'a Ctx,
    repository: &'a (dyn ILessonCommandRepository + Send + Sync),
}

impl<'a> LessonInteractor<'a> {
    pub fn new(ctx: &'a Ctx, repository: &'a (impl ILessonCommandRepository + Send + Sync)) -> Self {
        Self {
            ctx,
            repository,
        }
    }
}

impl<'a> LessonInteractor<'a> {
    pub async fn create_lesson(&self, lesson: LessonForCreate) -> LessonResult<i64> {
        let course_lessons = self.repository.get_course_lessons_ordered(&self.ctx, lesson.course_id).await?;

        let order = course_lessons.len() + 1;

        let lesson_for_c = LessonForCreateCommand {
            course_id: lesson.course_id,
            titile: lesson.titile,
            order: order as i32,
        };

        self.repository.create_lesson(&self.ctx, lesson_for_c).await
    }

    pub async fn update_lesson(&self, lesson_for_u: LessonForUpdate) -> LessonResult<()> {
        self.repository.update_lesson(&self.ctx, lesson_for_u).await
    }

    pub async fn change_order(&self, lesson_for_u_order: LessonForChangeOreder) -> LessonResult<()> {
        let course_lesson = self.repository.get_lesson(&self.ctx, lesson_for_u_order.id).await?;
        let course_lessons = self.repository.get_course_lessons_ordered(&self.ctx, course_lesson.course_id).await?;

        let course_lessons = self.compute_orders(&course_lessons, &lesson_for_u_order)?;
        self.repository.update_lesson_orders(&self.ctx, course_lessons).await?;

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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use crate::{
        core::{error::{CoreError, LessonError}, lesson::LessonInteractor}, ctx::Ctx, interfaces::lesson::{
            ILessonCommandRepository, LessonResult
        }, model::lesson::{
            Lesson, LessonForChangeOreder, LessonForCreateCommand, LessonForUpdate
        }};

    struct LessonCommandRepositoryMock {
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

        async fn update_lesson(&self, _: &Ctx, _: LessonForUpdate) -> LessonResult<()> {
            panic!()
        }
    }

    #[tokio::test]
    async fn test_change_order() {
        struct TestData {
            pub lessons: Vec<Lesson>,
            pub result: Vec<LessonForChangeOreder>,
            pub lesson_for_u_order: LessonForChangeOreder,
        }

        let course_id = 1;
        let user_id = 1;

        let data = vec![
            TestData {
                lessons: vec![
                    Lesson { id: 3, course_id, titile: "Lesson 3".to_string(), lesson_order: 1 },
                    Lesson { id: 5, course_id, titile: "Lesson 5".to_string(), lesson_order: 2 },
                    Lesson { id: 1, course_id, titile: "Lesson 1".to_string(), lesson_order: 3 },
                    Lesson { id: 10, course_id, titile: "Lesson 10".to_string(), lesson_order: 4 },
                    Lesson { id: 16, course_id, titile: "Lesson 16".to_string(), lesson_order: 5 },
                    Lesson { id: 9, course_id, titile: "Lesson 9".to_string(), lesson_order: 6 },
                ], 
                result: vec![
                    LessonForChangeOreder { id: 3, order: 1 },
                    LessonForChangeOreder { id: 5, order: 2 },
                    LessonForChangeOreder { id: 10, order: 3 },
                    LessonForChangeOreder { id: 16, order: 4 },
                    LessonForChangeOreder { id: 1, order: 5 },
                    LessonForChangeOreder { id: 9, order: 6 },
                ],
                lesson_for_u_order: LessonForChangeOreder { 
                    id: 1, 
                    order: 5,
                },
            },
            TestData {
                lessons: vec![
                    Lesson { id: 3, course_id, titile: "Lesson 3".to_string(), lesson_order: 1 },
                ], 
                result: vec![
                    LessonForChangeOreder { id: 3, order: 1 },
                ],
                lesson_for_u_order: LessonForChangeOreder { 
                    id: 3, 
                    order: 1,
                },
            },
            TestData {
                lessons: vec![
                    Lesson { id: 3, course_id, titile: "Lesson 3".to_string(), lesson_order: 1 },
                    Lesson { id: 5, course_id, titile: "Lesson 5".to_string(), lesson_order: 2 },
                    Lesson { id: 1, course_id, titile: "Lesson 1".to_string(), lesson_order: 3 },
                    Lesson { id: 10, course_id, titile: "Lesson 10".to_string(), lesson_order: 4 },
                    Lesson { id: 16, course_id, titile: "Lesson 16".to_string(), lesson_order: 5 },
                    Lesson { id: 9, course_id, titile: "Lesson 9".to_string(), lesson_order: 6 },
                ], 
                result: vec![
                    LessonForChangeOreder { id: 5, order: 1 },
                    LessonForChangeOreder { id: 1, order: 2 },
                    LessonForChangeOreder { id: 10, order: 3 },
                    LessonForChangeOreder { id: 16, order: 4 },
                    LessonForChangeOreder { id: 9, order: 5 },
                    LessonForChangeOreder { id: 3, order: 6 },
                ],
                lesson_for_u_order: LessonForChangeOreder { 
                    id: 3, 
                    order: 6,
                },
            },
            TestData {
                lessons: vec![
                    Lesson { id: 3, course_id, titile: "Lesson 3".to_string(), lesson_order: 1 },
                    Lesson { id: 5, course_id, titile: "Lesson 5".to_string(), lesson_order: 2 },
                    Lesson { id: 1, course_id, titile: "Lesson 1".to_string(), lesson_order: 3 },
                    Lesson { id: 10, course_id, titile: "Lesson 10".to_string(), lesson_order: 4 },
                    Lesson { id: 16, course_id, titile: "Lesson 16".to_string(), lesson_order: 5 },
                    Lesson { id: 9, course_id, titile: "Lesson 9".to_string(), lesson_order: 6 },
                ], 
                result: vec![
                    LessonForChangeOreder { id: 3, order: 1 },
                    LessonForChangeOreder { id: 16, order: 2 },
                    LessonForChangeOreder { id: 5, order: 3 },
                    LessonForChangeOreder { id: 1, order: 4 },
                    LessonForChangeOreder { id: 10, order: 5 },
                    LessonForChangeOreder { id: 9, order: 6 },
                ],
                lesson_for_u_order: LessonForChangeOreder { 
                    id: 16, 
                    order: 2,
                },
            },
        ];

        for test_data in data {
            let repository = LessonCommandRepositoryMock::new(test_data.lessons);
            let ctx = Ctx::new(user_id).unwrap();

            let lesson_interactor = LessonInteractor::new(&ctx, &repository);

            lesson_interactor.change_order(test_data.lesson_for_u_order).await.unwrap();

            let result = repository.get_course_lessons_ordered(&ctx, course_id).await.unwrap();

            assert_eq!(result, test_data.result);
        }
    }
}