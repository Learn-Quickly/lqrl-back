use crate::base::{self, DbRepository};
use crate::store::db_manager::DbManager;
use crate::store::dbx::error::DbxError;
use crate::store::error::{DbError, DbResult};
use async_trait::async_trait;
use lib_core::ctx::Ctx;
use lib_core::interfaces::user::{IUserCommandRepository, UserResult};
use lib_core::model::user::{User, UserForCreate, UserForUpdate};
use modql::field::{Fields, HasFields};
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Fields, FromRow, Debug)]
pub struct UserData {
	pub id: i64,
	pub username: String,

	pub pwd: String, 
	pub pwd_salt: Uuid,
	pub token_salt: Uuid,
}

#[derive(Fields)]
pub struct UserForInsert {
	pub username: String,
	pub pwd: String,
	pub pwd_salt: Uuid,
	pub token_salt: Uuid,
}

#[derive(Fields)]
pub struct UserForUpdatePwd {
	pub pwd: String, 
}

#[derive(Fields)]
pub struct UserForUpdateData {
	pub username: String,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForLogin {
	pub id: i64,
	pub username: String,

	pub pwd: Option<String>, 
	pub pwd_salt: Uuid,
	pub token_salt: Uuid,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForAuth {
	pub id: i64,
	pub username: String,

	pub token_salt: Uuid,
}

pub trait UserBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for UserData {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}

#[derive(Clone)]
pub struct UserCommandRepository {
	dbm: DbManager,
}

impl DbRepository for UserCommandRepository {
	const TABLE: &'static str = "user";
}

impl UserCommandRepository {
	pub fn new(dbm: DbManager) -> Self {
		Self {
			dbm,
		}
	}

	async fn get<E>(&self, ctx: &Ctx, id: i64) -> DbResult<E>
	where
		E: UserBy,
	{
		base::get::<Self, _>(ctx, &self.dbm, id).await.map_err(Into::<DbError>::into)
	}
}

#[async_trait]
impl IUserCommandRepository for UserCommandRepository {
	async fn get_user(&self, ctx: &Ctx, user_id: i64) -> UserResult<User> {
		let user_data: UserData = self.get(ctx, user_id).await?;
		let user = User { 
			id: user_data.id, 
			username: user_data.username, 
			pwd: user_data.pwd, 
			pwd_salt: user_data.pwd_salt, 
			token_salt: user_data.token_salt,
		};

		Ok(user)
	}

	async fn create_user(
		&self,
		ctx: &Ctx,
		user_c: UserForCreate,
	) -> UserResult<i64> {
		let UserForCreate {
			username,
			pwd,
			pwd_salt,
			token_salt,
		} = user_c;

		let user_fi = UserForInsert { 
			username: username.clone(),
			pwd, 
			pwd_salt, 
			token_salt,
		};

		let user_id = base::create::<Self, _>(ctx, &self.dbm, user_fi).await.map_err(
			|model_error| {
				DbxError::resolve_unique_violation(
					model_error.into(),
					Some(|table: &str, constraint: &str| {
						if table == "user" && constraint.contains("username") {
							Some(DbError::UserAlreadyExists { username })
						} else {
							None // Error::UniqueViolation will be created by resolve_unique_violation
						}
					}),
				)
			},
		)?;

		Ok(user_id)
	}
	
	async fn update_pwd(
		&self,
		ctx: &Ctx,
		id: i64,
		pwd: String,
	) -> UserResult<()> {
		let user_for_update_pwd = UserForUpdatePwd { 
			pwd,
		};

		base::update::<Self, UserForUpdatePwd>(&ctx, &self.dbm, id, user_for_update_pwd)
			.await
			.map_err(Into::<DbError>::into)?;

		Ok(())
	}

	async fn update_user(&self, ctx: &Ctx, user_for_u: UserForUpdate) -> UserResult<()> {
		let user_for_u = UserForUpdateData { 
			username: user_for_u.username
		};

		base::update::<Self, UserForUpdateData>(&ctx, &self.dbm, ctx.user_id(), user_for_u)
			.await
			.map_err(Into::<DbError>::into)?;

		Ok(())
	}
}