use utoipa::{
	openapi::security::SecurityScheme,
	Modify, OpenApi,
};

use utoipa::openapi::security::Http;
use crate::routes::user_routes::{routes_login, routes_register, routes_user};
use crate::routes::{routes_course, routes_lesson};

#[derive(OpenApi)]
#[openapi(
    paths(
		// User
		routes_user::api_get_user_data_handler,
		routes_user::api_get_user_by_id_handler,

		routes_user::api_update_user_handler,
		routes_user::api_change_pwd_handler,
		routes_login::api_login_handler,
		routes_login::api_refresh_access_token_handler,
		routes_register::api_register_handler,
 
		// Course
		routes_course::api_set_course_img_handler,
		routes_course::api_create_course_draft_handler,
		routes_course::api_update_course_handler,
		routes_course::api_publish_course_handler,
		routes_course::api_archive_course_handler,
		routes_course::api_register_for_course_handler,
		routes_course::api_unsubscribe_from_course_handler,

		routes_course::api_get_course_handler,
		routes_course::api_get_courses_handler,
		routes_course::api_get_user_courses_created_handler,
		routes_course::api_get_user_courses_registered_handler,

		// Lesson
		routes_lesson::api_create_lesson_handler,
		routes_lesson::api_update_lesson_handler,
		routes_lesson::api_lesson_change_order_handler,
        routes_lesson::api_delete_lesson_handler,
    ),
    components(
		schemas(
			// User
			routes_user::UserDataPayload,
			routes_user::UserUpdatePayload,
			routes_user::UserChangePwdPayload,
			routes_register::RegisterPayload,
			routes_login::RefreshTokenPayload,

			// Course
			routes_course::CourseCreateDraftPayload,
			routes_course::CreatedCourseDraft,
			routes_course::CourseUpdatePayload,
			routes_course::CoursePayload,
			routes_course::CourseStatePayload,
			routes_course::CourseId,
			routes_course::CourseFilterPayload,

			// Lesson
			routes_lesson::LessonCreatedPayload,
			routes_lesson::LessonCreatePayload,
            routes_lesson::LessonDeletePayload,
			routes_lesson::LessonUpdatePayload,
			routes_lesson::LessonChangeOrderPayload,
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