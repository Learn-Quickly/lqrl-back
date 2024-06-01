use utoipa::{
	openapi::security::SecurityScheme,
	Modify, OpenApi,
};

use utoipa::openapi::security::Http;
use crate::routes::models;
use crate::routes::user::{login, register, user, course as user_course, lesson as user_lesson};
use crate::routes::student::{course as student_course, lesson as student_lesson, lesson_progress as student_lesson_progress, exercise as student_exercise};
use crate::routes::creator::{course as creator_course, lesson as creator_lesson, exercise as creator_exercise};

#[derive(OpenApi)]
#[openapi(
    paths(
		// User
		user::api_get_user_data_handler,
		user::api_get_user_by_id_handler,

		user::api_update_user_handler,
		user::api_change_pwd_handler,
		login::api_login_handler,
		login::api_refresh_access_token_handler,
		register::api_register_handler,
 
		// Course user 
		user_course::api_get_course_handler,
		user_course::api_get_courses_handler,

		// Course creator
		creator_course::api_set_course_img_handler,
		creator_course::api_create_course_draft_handler,
		creator_course::api_update_course_handler,
		creator_course::api_publish_course_handler,
		creator_course::api_archive_course_handler,
		creator_course::api_get_created_courses_handler,
		creator_course::api_get_attendants,

		// Course student
		student_course::api_register_for_course_handler,
		student_course::api_unsubscribe_from_course_handler,
		student_course::api_get_user_courses_registered_handler,

		// Lesson
		creator_lesson::api_create_lesson_handler,
		creator_lesson::api_update_lesson_handler,
		creator_lesson::api_lesson_change_order_handler,
        creator_lesson::api_delete_lesson_handler,

		user_lesson::api_get_lessons_handler,

		student_lesson::api_start_lesson_handler,
		student_lesson_progress::api_get_lesson_progresses_handler,

		// Exercise
		creator_exercise::api_create_exercise_handler,
		creator_exercise::api_update_exercise_handler,
		creator_exercise::api_exercise_change_order_handler,

		student_exercise::api_start_exercise_handler,
    ),
    components(
		schemas(
			// User
			user::UserDataPayload,
			user::UserUpdatePayload,
			user::UserChangePwdPayload,
			models::user::GetAttendatsPayload,
			models::user::UserPayload,
			register::RegisterPayload,
			login::RefreshTokenPayload,

			// Course
			models::course::CourseCreateDraftPayload,
			models::course::CreatedCourseDraft,
			models::course::CourseUpdatePayload,
			models::course::CoursePayload,
			models::course::CourseStatePayload,
			models::course::CourseId,
			models::course::CourseFilterPayload,
			models::course::CoursesPayload,

			// Lesson
			models::lesson::LessonCreatedPayload,
			models::lesson::LessonCreatePayload,
  			models::lesson::LessonDeletePayload,
			models::lesson::LessonUpdatePayload,
			models::lesson::LessonChangeOrderPayload,
			models::lesson::LessonDataPayload,
			models::lesson::StartLessonPayload,

			// Lesson progress
			models::lesson_progress::GetLessonProgressesPayload,
			models::lesson_progress::LessonProgressPayload,

			// Exercise
			models::exercise::ExerciseCreatePayload,
			models::exercise::ExerciseCreatedPayload,
			models::exercise::ExerciseForUpdatePayload,
			models::exercise::ExerciseChangeOrderPayload,

			models::exercise::ExerciseId,
		)
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "LQRL", description = "A great Rust backend API for the awesome LQRL project")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(Http::new(utoipa::openapi::security::HttpAuthScheme::Bearer)),
                );
			components.add_security_scheme(
				"basicAuth",
				SecurityScheme::Http(Http::new(utoipa::openapi::security::HttpAuthScheme::Basic))
			)
        }
    }
}