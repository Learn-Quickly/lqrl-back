mod config;
mod error;
mod middleware;
mod routes;

use config::web_config;
use lib_db::_dev_utils;
use lib_db::store::DbManager;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use utoipa::openapi::security::Http;
use error::AppResult;

use crate::middleware::mw_auth::{mw_ctx_require, mw_ctx_resolver};
use crate::middleware::mw_req_stamp::mw_req_stamp_resolver;
use crate::middleware::mw_res_map::mw_reponse_map;
use crate::routes::{routes_login, routes_register, routes_static, routes_course};
use axum::{middleware as axum_middleware, Router};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;
use utoipa::{
	openapi::security::SecurityScheme,
	Modify, OpenApi,
};

use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> AppResult<()> {
	#[derive(OpenApi)]
    #[openapi(
        paths(
			routes_login::api_login_handler,
			routes_login::api_refresh_access_token_handler,
			routes_register::api_register_handler,
			routes_course::api_set_course_img_handler,
			routes_course::api_create_course_draft,
			routes_course::api_publish_course,
			routes_course::api_archive_course,
			routes_course::api_register_for_course,
			routes_course::api_unsubscribe_from_course,
			routes_course::api_get_course,
			routes_course::api_get_courses,
        ),
        components(
			schemas(
				routes_register::RegisterPayload,
				routes_login::RefreshTokenPayload,
				routes_course::CourseCreateDraftPayload,
				routes_course::CreatedCourseDraft,
				routes_course::CoursePayload,
				routes_course::CourseId,
				routes_course::CourseFilterPayload,
			)
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "LQRL", description = "A great Rust backend API for the awesome LQRL project")
        )
    )]
    struct ApiDoc;

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
	
	tracing_subscriber::fmt()
		.without_time() // For early local development.
		.with_target(false)
		.with_env_filter(EnvFilter::from_default_env())
		.with_level(true)
		.init();

	// -- FOR DEV ONLY
	_dev_utils::init_dev().await;

	let dbm = DbManager::new().await?;

	let routes_course = routes::routes_course::routes(dbm.clone())
		.route_layer(axum_middleware::from_fn(mw_ctx_require));

	let cors = CorsLayer::new()
		.allow_methods(Any)
		.allow_headers(Any)
		.allow_origin(Any);

	let routes_all = Router::new()
		.nest("/api", routes_course)
        .layer(axum_middleware::from_fn_with_state(dbm.clone(), mw_ctx_resolver))
		.merge(routes_login::routes(dbm.clone()))
		.merge(routes_register::routes(dbm.clone()))
		.layer(axum_middleware::map_response(mw_reponse_map))
		.layer(axum_middleware::from_fn(mw_req_stamp_resolver))
		.nest_service("/", ServeDir::new("public"))
		.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
		.fallback_service(routes_static::serve_dir())
		.layer(cors);

	let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
	info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());
	axum::serve(listener, routes_all.into_make_service())
		.await
		.unwrap();

	Ok(())
}
