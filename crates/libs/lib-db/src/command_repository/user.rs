use crate::base::{self, DbRepository};
use crate::store::dbx::error::DbxError;
use crate::store::error::{DbError, DbResult};
use crate::store::DbManager;
use async_trait::async_trait;
use lib_auth::pwd::{self, ContentToHash};
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

	pub pwd: String, // encrypted, #_scheme_id_#....
	pub pwd_salt: Uuid,
	pub token_salt: Uuid,
}

#[derive(Fields)]
pub struct UserForInsert {
	pub username: String,
}

#[derive(Fields)]
pub struct UserForUpdateData {
	pub username: String,
}

#[derive(Fields)]
pub struct UserForUpdatePwd {
	pub pwd: String,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForLogin {
	pub id: i64,
	pub username: String,

	// -- pwd and token info
	pub pwd: Option<String>, // encrypted, #_scheme_id_#....
	pub pwd_salt: Uuid,
	pub token_salt: Uuid,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForAuth {
	pub id: i64,
	pub username: String,

	// -- token info
	pub token_salt: Uuid,
}

/// Marker trait
pub trait UserBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for UserData {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}

// endregion: --- User Types

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
			pwd_clear,
		} = user_c;

		// -- Create the user row
		let user_fi = UserForInsert {
			username: username.to_string(),
		};

		// Start the transaction
		let dbm = self.dbm.new_with_txn()?;

		dbm.dbx().begin_txn().await.map_err(Into::<DbError>::into)?;

		let user_id = base::create::<Self, _>(ctx, &dbm, user_fi).await.map_err(
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

		// -- Update the database
		self.update_pwd(ctx, user_id, &pwd_clear).await?;

		// Commit the transaction
		dbm.dbx().commit_txn().await.map_err(Into::<DbError>::into)?;

		Ok(user_id)
	}
	
	async fn update_pwd(
		&self,
		ctx: &Ctx,
		id: i64,
		pwd_clear: &str,
	) -> UserResult<()> {
		// -- Prep password
		let user: UserForLogin = self.get(ctx, id).await?;
		let pwd = pwd::hash_pwd(ContentToHash {
			content: pwd_clear.to_string(),
			salt: user.pwd_salt,
		})
		.await
		.map_err(Into::<DbError>::into)?;

		let user_for_upwd = UserForUpdatePwd {
    		pwd,
		};

		base::update::<Self, UserForUpdatePwd>(ctx, &self.dbm, id, user_for_upwd)
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