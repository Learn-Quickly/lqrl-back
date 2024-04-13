use axum::{extract::{Path, State}, routing::{get, put}, Json, Router};
use lib_core::{interactors::user::user::UserInteractor, model::user::{UserForChangePassword, UserForUpdate}};
use lib_db::query_repository::user::UserData;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;

use crate::{app_state::AppState, error::AppResult, middleware::mw_auth::CtxW};


pub fn routes(app_state: AppState) -> Router {
	Router::new()
		.route("/get_user_data", get(api_get_user_data_handler))
		.route("/get_user/:i64", get(api_get_user_by_id_handler))
		.route("/update", put(api_update_user_handler))
		.route("/change_pwd", put(api_change_pwd_handler))
		.with_state(app_state)
}

#[derive(Serialize, ToSchema)]
pub struct UserDataPayload {
    pub id: i64,
    pub username: String,
}

#[utoipa::path(
	get,
	path = "/api/user/get_user_data",
	responses(
		(status = 200, body=UserDataPayload),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_get_user_data_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
) -> AppResult<Json<UserDataPayload>> {
    let ctx = ctx.0;
    let user_id = ctx.user_id();

	let user_query_repository = app_state.query_repository_manager.get_user_repository();
    let user: UserData = user_query_repository.get(&ctx, user_id).await?;

    let result = UserDataPayload {
        id: user.id,
        username: user.username,
    };

    Ok(Json(result))
}

#[utoipa::path(
	get,
	path = "/api/user/get_user/{user_id}",
	params(
		("user_id", description = "ID of the user")
	),
	responses(
		(status = 200, body=UserDataPayload),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_get_user_by_id_handler(
	ctx: CtxW,
	State(app_state): State<AppState>,
	Path(user_id): Path<i64>,
) -> AppResult<Json<UserDataPayload>> {
    let ctx = ctx.0;

	let user_query_repo = app_state.query_repository_manager.get_user_repository();
    let user: UserData = user_query_repo.get(&ctx, user_id).await?;

    let result = UserDataPayload {
        id: user.id,
        username: user.username,
    };

    Ok(Json(result))
}

#[derive(Deserialize, ToSchema)]
pub struct UserUpdatePayload {
    pub username: String,
}

#[utoipa::path(
	put,
	path = "/api/user/update",
	request_body = UserUpdatePayload,
	responses(
		(status = 200, description = "Course updated successfully"),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_update_user_handler(
    ctx: CtxW,
    State(app_state): State<AppState>,
    Json(payload): Json<UserUpdatePayload>
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;

    let user_for_u = UserForUpdate {
        username: payload.username,
    };

	let command_repository_manager = app_state.command_repository_manager;
    let controller = UserInteractor::new(command_repository_manager.as_ref());

    controller.update_user(&ctx, user_for_u).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UserChangePwdPayload {
    pub pwd_clear: String,
    pub new_pwd_clear: String,
}

#[utoipa::path(
	put,
	path = "/api/user/change_pwd",
	request_body = UserChangePwdPayload,
	responses(
		(status = 200),
	),
	security(
		("bearerAuth" = [])
	)
)]
async fn api_change_pwd_handler(
    ctx: CtxW,
    State(app_state): State<AppState>,
    Json(payload): Json<UserChangePwdPayload>,
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;

    let user_for_u_pwd = UserForChangePassword {
        pwd_clear: payload.pwd_clear,
        new_pwd_clear: payload.new_pwd_clear,
    };

	let command_repository_manager = app_state.command_repository_manager;
    let controller = UserInteractor::new(command_repository_manager.as_ref());

    controller.change_pwd(&ctx, user_for_u_pwd).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}