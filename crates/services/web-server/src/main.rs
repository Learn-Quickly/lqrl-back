mod config;
mod error;
mod middleware;
mod routes;
mod api_doc;
mod app_state;

use config::web_config;
use lib_db::_dev_utils;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use error::AppResult;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api_doc::ApiDoc;
use crate::app_state::AppState;
use crate::middleware::mw_auth::{mw_ctx_require, mw_ctx_resolver};
use crate::middleware::mw_req_stamp::mw_req_stamp_resolver;
use crate::middleware::mw_res_map::mw_reponse_map;
use crate::routes::routes_static;
use crate::routes::user::{login, register};
use axum::{middleware as axum_middleware, Router};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> AppResult<()> {	
	tracing_subscriber::fmt()
		.without_time() // For early local development.
		.with_target(false)
		.with_env_filter(EnvFilter::from_default_env())
		.with_level(true)
		.init();

	// -- FOR DEV ONLY
	// _dev_utils::init_dev().await;

	let app_state = AppState::new().await?;

	let cors = CorsLayer::new()
		.allow_methods(Any)
		.allow_headers(Any)
		.allow_origin(Any);

	let routes_user = routes::user::user::routes(app_state.clone())
		.route_layer(axum_middleware::from_fn(mw_ctx_require));

	let routes_user_course = routes::user::course::routes(app_state.clone())
		.route_layer(axum_middleware::from_fn(mw_ctx_require));

	let routes_student_course = routes::student::course::routes(app_state.clone())
		.route_layer(axum_middleware::from_fn(mw_ctx_require));

	let routes_creator_course = routes::creator::course::routes(app_state.clone())
		.route_layer(axum_middleware::from_fn(mw_ctx_require));

	let routes_lesson = routes::creator::lesson::routes(app_state.clone())
		.route_layer(axum_middleware::from_fn(mw_ctx_require));

	let routes_all = Router::new()
		.nest("/api/course", routes_user_course)
		.nest("/api/course", routes_student_course)
		.nest("/api/course", routes_creator_course)
		.nest("/api/course/lesson", routes_lesson)
		.nest("/api/user", routes_user)
        .layer(axum_middleware::from_fn_with_state(app_state.clone(), mw_ctx_resolver))
		.merge(login::routes(app_state.clone()))
		.merge(register::routes(app_state.clone()))
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
