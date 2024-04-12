use lib_core::ctx::Ctx;
use modql::{field::{Fields, HasFields}, filter::{FilterNodes, ListOptions, OpValsInt64, OpValsString, OpValsValue}};
use uuid::Uuid;
use crate::{query_repository::modql_utils::time_to_sea_value, store::db_manager::DbManager};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::Deserialize;
use sqlx::{postgres::PgRow, FromRow};

use crate::{base::{self, DbRepository}, store::error::{DbError, DbResult}};

#[derive(Clone, Fields, FromRow, Debug)]
pub struct UserData {
	pub id: i64,
	pub username: String,

	pub pwd: Option<String>, 
	pub pwd_salt: Uuid,
	pub token_salt: Uuid,
}

pub trait UserBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for UserData {}

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

#[derive(Iden)]
enum UserIden {
	Username,
}

#[derive(Clone)]
pub struct UserQueryRepository {
	dbm: DbManager,
} 

impl UserQueryRepository {
	pub fn new(dbm: DbManager) -> Self {
		Self {
    		dbm,
		}
	}
}

impl DbRepository for UserQueryRepository {
    const TABLE: &'static str = "user";
}

impl UserQueryRepository {
	pub async fn first_by_username<E>(
		&self,
		_ctx: &Ctx,
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
		let entity = self.dbm.dbx().fetch_optional(sqlx_query).await?;

		Ok(entity)
	}

	pub async fn get<E>(&self, ctx: &Ctx, id: i64) -> DbResult<E>
	where
		E: UserBy,
	{
		base::get::<Self, _>(ctx, &self.dbm, id).await.map_err(Into::<DbError>::into)
	}

	pub async fn list(
		&self,
		ctx: &Ctx,
		filter: Option<Vec<UserFilter>>,
		list_options: Option<ListOptions>,
	) -> DbResult<Vec<UserData>> {
		base::list::<Self, _, _>(ctx, &self.dbm, filter, list_options)
			.await
			.map_err(Into::<DbError>::into)
	}
}