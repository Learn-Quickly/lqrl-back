use crate::base::{self, prep_fields_for_update, DbRepository};
use crate::command_repository::modql_utils::time_to_sea_value;
use crate::store::error::{DbError, DbResult};
use crate::store::DbManager;
use lib_auth::pwd::{self, ContentToHash};
use lib_core::ctx::Ctx;
use modql::field::{Field, Fields, HasFields};
use modql::filter::{
	FilterNodes, ListOptions, OpValsInt64, OpValsString, OpValsValue,
};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Fields, FromRow, Debug, Serialize)]
pub struct User {
	pub id: i64,
	pub username: String,
}

#[derive(Deserialize)]
pub struct UserForCreate {
	pub username: String,
	pub pwd_clear: String,
}

#[derive(Fields)]
pub struct UserForInsert {
	pub username: String,
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

impl UserBy for User {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}

// Note: Since the entity properties Iden will be given by modql
//       UserIden does not have to be exhaustive, but just have the columns
//       we use in our specific code.
#[derive(Iden)]
enum UserIden {
	Id,
	Username,
	Pwd,
}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct UserFilter {
	pub id: Option<OpValsInt64>,

	pub username: Option<OpValsString>,

	pub cid: Option<OpValsInt64>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	pub ctime: Option<OpValsValue>,
	pub mid: Option<OpValsInt64>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	pub mtime: Option<OpValsValue>,
}

// endregion: --- User Types

// region:    --- UserBmc

pub struct UserBmc;

impl DbRepository for UserBmc {
	const TABLE: &'static str = "user";
}

impl UserBmc {
	pub async fn create(
		ctx: &Ctx,
		dbm: &DbManager,
		user_c: UserForCreate,
	) -> DbResult<i64> {
		let UserForCreate {
			username,
			pwd_clear,
		} = user_c;

		// -- Create the user row
		let user_fi = UserForInsert {
			username: username.to_string(),
		};

		// Start the transaction
		let dbm = dbm.new_with_txn()?;

		dbm.dbx().begin_txn().await?;

		let user_id = base::create::<Self, _>(ctx, &dbm, user_fi).await.map_err(
			|model_error| {
				DbError::resolve_unique_violation(
					model_error,
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
		Self::update_pwd(ctx, &dbm, user_id, &pwd_clear).await?;

		// Commit the transaction
		dbm.dbx().commit_txn().await?;

		Ok(user_id)
	}

	pub async fn get<E>(ctx: &Ctx, dbm: &DbManager, id: i64) -> DbResult<E>
	where
		E: UserBy,
	{
		base::get::<Self, _>(ctx, dbm, id).await
	}

	pub async fn first_by_username<E>(
		_ctx: &Ctx,
		dbm: &DbManager,
		username: &str,
	) -> DbResult<Option<E>>
	where
		E: UserBy,
	{
		// -- Build query
		let mut query = Query::select();
		query
			.from(Self::table_ref())
			.columns(E::field_idens())
			.and_where(Expr::col(UserIden::Username).eq(username));

		// -- Execute query
		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

		let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
		let entity = dbm.dbx().fetch_optional(sqlx_query).await?;

		Ok(entity)
	}

	pub async fn list(
		ctx: &Ctx,
		dbm: &DbManager,
		filter: Option<Vec<UserFilter>>,
		list_options: Option<ListOptions>,
	) -> DbResult<Vec<User>> {
		base::list::<Self, _, _>(ctx, dbm, filter, list_options).await
	}

	pub async fn update_pwd(
		ctx: &Ctx,
		dbm: &DbManager,
		id: i64,
		pwd_clear: &str,
	) -> DbResult<()> {
		// -- Prep password
		let user: UserForLogin = Self::get(ctx, dbm, id).await?;
		let pwd = pwd::hash_pwd(ContentToHash {
			content: pwd_clear.to_string(),
			salt: user.pwd_salt,
		})
		.await?;

		// -- Prep the data
		let mut fields = Fields::new(vec![Field::new(UserIden::Pwd, pwd.into())]);
		prep_fields_for_update::<Self>(&mut fields, ctx.user_id());

		// -- Build query
		let fields = fields.for_sea_update();
		let mut query = Query::update();
		query
			.table(Self::table_ref())
			.values(fields)
			.and_where(Expr::col(UserIden::Id).eq(id));

		// -- Exec query
		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
		let sqlx_query = sqlx::query_with(&sql, values);
		let _count = dbm.dbx().execute(sqlx_query).await?;

		Ok(())
	}

	/// TODO: For User, deletion will require a soft-delete approach:
	///       - Set `deleted: true`.
	///       - Change `username` to "DELETED-_user_id_".
	///       - Clear any other UUIDs or PII (Personally Identifiable Information).
	///       - The automatically set `mid`/`mtime` will record who performed the deletion.
	///       - It's likely necessary to record this action in a `um_change_log` (a user management change audit table).
	///       - Remove or clean up any user-specific assets (messages, etc.).
	pub async fn delete(ctx: &Ctx, dbm: &DbManager, id: i64) -> DbResult<()> {
		base::delete::<Self>(ctx, dbm, id).await
	}
}

// endregion: --- UserBmc

// region:    --- Tests

#[cfg(test)]
mod tests {
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>; // For tests.

	use super::*;
	use crate::_dev_utils;
	use serial_test::serial;

	#[serial]
	#[tokio::test]
	async fn test_create_ok() -> Result<()> {
		// -- Setup & Fixtures
		let dbm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_username = "test_create_ok-user-01";
		let fx_pwd_clear = "test_create_ok pwd 01";

		// -- Exec
		let user_id = UserBmc::create(
			&ctx,
			&dbm,
			UserForCreate {
				username: fx_username.to_string(),
				pwd_clear: fx_pwd_clear.to_string(),
			},
		)
		.await?;

		// -- Check
		let user: UserForLogin = UserBmc::get(&ctx, &dbm, user_id).await?;
		assert_eq!(user.username, fx_username);

		// -- Clean
		UserBmc::delete(&ctx, &dbm, user_id).await?;

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_first_ok_demo1() -> Result<()> {
		// -- Setup & Fixtures
		let dbm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_username = "demo1";

		// -- Exec
		let user: User = UserBmc::first_by_username(&ctx, &dbm, fx_username)
			.await?
			.ok_or("Should have user 'demo1'")?;

		// -- Check
		assert_eq!(user.username, fx_username);

		Ok(())
	}
}

// endregion: --- Tests
