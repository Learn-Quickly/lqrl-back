use axum::{extract::{Path, State}, routing::{get, put}, Json, Router};
use lib_core::{core::user::UserInteractor, model::user::{UserForChangePassword, UserForUpdate}};
use lib_db::{command_repository::user::UserCommandRepository, query_repository::user::{UserData, UserQueryRepository}, store::DbManager};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;

use crate::{error::AppResult, middleware::mw_auth::CtxW};


pub fn routes(dbm: DbManager) -> Router {
	Router::new()
		.route("/get_user_data", get(api_get_user_data_handler))
		.route("/get_user/:i64", get(api_get_user_by_id_handler))
		.route("/update", put(api_update_user_handler))
		.route("/change_pwd", put(api_change_pwd_handler))
		.with_state(dbm)
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
	State(dbm): State<DbManager>,
) -> AppResult<Json<UserDataPayload>> {
    let ctx = ctx.0;
    let user_id = ctx.user_id();

    let user: UserData = UserQueryRepository::get(&ctx, &dbm, user_id).await?;

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
	State(dbm): State<DbManager>,
	Path(user_id): Path<i64>,
) -> AppResult<Json<UserDataPayload>> {
    let ctx = ctx.0;

    let user: UserData = UserQueryRepository::get(&ctx, &dbm, user_id).await?;

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
    State(dbm): State<DbManager>,
    Json(payload): Json<UserUpdatePayload>
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;

    let user_for_u = UserForUpdate {
        username: payload.username,
    };

    let repository = UserCommandRepository::new(dbm);
    let controller = UserInteractor::new(&ctx, &repository);

    controller.update_user(user_for_u).await?;

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
    State(dbm): State<DbManager>,
    Json(payload): Json<UserChangePwdPayload>,
) -> AppResult<Json<Value>> {
    let ctx = ctx.0;

    let user_for_u_pwd = UserForChangePassword {
        pwd_clear: payload.pwd_clear,
        new_pwd_clear: payload.new_pwd_clear,
    };

    let repository = UserCommandRepository::new(dbm);
    let controller = UserInteractor::new(&ctx, &repository);

    controller.change_pwd(user_for_u_pwd).await?;

	let body = Json(json!({
		"result": {
			"success": true,
		}
	}));

	Ok(body)
}