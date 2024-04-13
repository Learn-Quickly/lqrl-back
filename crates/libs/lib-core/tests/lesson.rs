use std::sync::Arc;

use lib_core::{interactors::creator::lesson::CreatorLessonInteractor, ctx::Ctx, interfaces::command_repository_manager::ICommandRepositoryManager, models::lesson::{Lesson, LessonForChangeOreder}};

use crate::common::repository_manager::CommandRepositoryManagerMock;

mod common;


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
                Lesson { id: 3, course_id, title: "Lesson 3".to_string(), lesson_order: 1 },
                Lesson { id: 5, course_id, title: "Lesson 5".to_string(), lesson_order: 2 },
                Lesson { id: 1, course_id, title: "Lesson 1".to_string(), lesson_order: 3 },
                Lesson { id: 10, course_id, title: "Lesson 10".to_string(), lesson_order: 4 },
                Lesson { id: 16, course_id, title: "Lesson 16".to_string(), lesson_order: 5 },
                Lesson { id: 9, course_id, title: "Lesson 9".to_string(), lesson_order: 6 },
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
                Lesson { id: 3, course_id, title: "Lesson 3".to_string(), lesson_order: 1 },
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
                Lesson { id: 3, course_id, title: "Lesson 3".to_string(), lesson_order: 1 },
                Lesson { id: 5, course_id, title: "Lesson 5".to_string(), lesson_order: 2 },
                Lesson { id: 1, course_id, title: "Lesson 1".to_string(), lesson_order: 3 },
                Lesson { id: 10, course_id, title: "Lesson 10".to_string(), lesson_order: 4 },
                Lesson { id: 16, course_id, title: "Lesson 16".to_string(), lesson_order: 5 },
                Lesson { id: 9, course_id, title: "Lesson 9".to_string(), lesson_order: 6 },
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
                Lesson { id: 3, course_id, title: "Lesson 3".to_string(), lesson_order: 1 },
                Lesson { id: 5, course_id, title: "Lesson 5".to_string(), lesson_order: 2 },
                Lesson { id: 1, course_id, title: "Lesson 1".to_string(), lesson_order: 3 },
                Lesson { id: 10, course_id, title: "Lesson 10".to_string(), lesson_order: 4 },
                Lesson { id: 16, course_id, title: "Lesson 16".to_string(), lesson_order: 5 },
                Lesson { id: 9, course_id, title: "Lesson 9".to_string(), lesson_order: 6 },
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
        TestData {
            lessons: vec![
                Lesson { id: 3, course_id, title: "Lesson 3".to_string(), lesson_order: 1 },
                Lesson { id: 5, course_id, title: "Lesson 5".to_string(), lesson_order: 2 },
                Lesson { id: 1, course_id, title: "Lesson 1".to_string(), lesson_order: 3 },
                Lesson { id: 10, course_id, title: "Lesson 10".to_string(), lesson_order: 4 },
                Lesson { id: 16, course_id, title: "Lesson 16".to_string(), lesson_order: 5 },
                Lesson { id: 9, course_id, title: "Lesson 9".to_string(), lesson_order: 6 },
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
                Lesson { id: 3, course_id, title: "Lesson 3".to_string(), lesson_order: 1 },
                Lesson { id: 5, course_id, title: "Lesson 5".to_string(), lesson_order: 2 },
                Lesson { id: 1, course_id, title: "Lesson 1".to_string(), lesson_order: 3 },
                Lesson { id: 10, course_id, title: "Lesson 10".to_string(), lesson_order: 4 },
                Lesson { id: 16, course_id, title: "Lesson 16".to_string(), lesson_order: 5 },
                Lesson { id: 9, course_id, title: "Lesson 9".to_string(), lesson_order: 6 },
            ], 
            result: vec![
                LessonForChangeOreder { id: 9, order: 1 },
                LessonForChangeOreder { id: 3, order: 2 },
                LessonForChangeOreder { id: 5, order: 3 },
                LessonForChangeOreder { id: 1, order: 4 },
                LessonForChangeOreder { id: 10, order: 5 },
                LessonForChangeOreder { id: 16, order: 6 },
            ],
            lesson_for_u_order: LessonForChangeOreder { 
                id: 9, 
                order: 1,
            },
        }
    ];

    for test_data in data {
        let repository_manager = Arc::new(CommandRepositoryManagerMock::new(test_data.lessons));

        let lesson_interactor = CreatorLessonInteractor::new(repository_manager.clone());

        let ctx = Ctx::new(user_id).unwrap();

        lesson_interactor.change_order(&ctx, test_data.lesson_for_u_order).await.unwrap();

        let lesson_repository = repository_manager.get_lesson_repository();
        let result = lesson_repository.get_course_lessons_ordered(&ctx, course_id).await.unwrap();

        assert_eq!(result, test_data.result);
    }
}