// region:    --- Modules

mod config;
mod error;
mod log;
mod web;

pub use self::error::{Error, Result};
use config::web_config;
use tower_http::services::ServeDir;
use utoipa::openapi::security::{ApiKey, ApiKeyValue};

use crate::web::mw_auth::{mw_ctx_require, mw_ctx_resolver};
use crate::web::mw_req_stamp::mw_req_stamp_resolver;
use crate::web::mw_res_map::mw_reponse_map;
use crate::web::{routes_login, routes_register, routes_static, routes_course};
use axum::{middleware, Router};
use lib_core::_dev_utils;
use lib_core::model::ModelManager;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;
use utoipa::{
	openapi::security::SecurityScheme,
	Modify, OpenApi,
};

use utoipa_swagger_ui::SwaggerUi;
// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
	#[derive(OpenApi)]
    #[openapi(
        paths(
			routes_login::api_login_handler,
			routes_login::api_logoff_handler,
			routes_register::api_register_handler,
			routes_course::api_set_course_img_handler,
        ),
        components(
			schemas(
				routes_login::LoginPayload,
				routes_register::RegisterPayload,
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
                    "auth_token",
                    SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("auth-token"))),
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

	let mm = ModelManager::new().await?;

	let routes_course = web::routes_course::routes(mm.clone())
		.route_layer(middleware::from_fn(mw_ctx_require));

	let routes_all = Router::new()
		.merge(routes_login::routes(mm.clone()))
		.merge(routes_register::routes(mm.clone()))
		.nest("/api", routes_course)
		.layer(middleware::map_response(mw_reponse_map))
		.layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolver))
		.layer(CookieManagerLayer::new())
		.layer(middleware::from_fn(mw_req_stamp_resolver))
		.nest_service("/", ServeDir::new("public"))
		.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
		.fallback_service(routes_static::serve_dir());

	// region:    --- Start Server
	// Note: For this block, ok to unwrap.
	let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
	info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());
	axum::serve(listener, routes_all.into_make_service())
		.await
		.unwrap();
	// endregion: --- Start Server

	Ok(())
}
