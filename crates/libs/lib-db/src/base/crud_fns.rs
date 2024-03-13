use crate::base::{
	prep_fields_for_create, prep_fields_for_update, CommonIden, DbRepository,
	LIST_LIMIT_DEFAULT, LIST_LIMIT_MAX,
};
use crate::store::error::{DbError, DbResult};
use crate::store::DbManager;
use lib_core::ctx::Ctx;
use modql::field::HasFields;
use modql::filter::{FilterGroups, ListOptions};
use sea_query::{Condition, Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::postgres::PgRow;
use sqlx::FromRow;

pub async fn create<MC, E>(ctx: &Ctx, dbm: &DbManager, data: E) -> DbResult<i64>
where
	MC: DbRepository,
	E: HasFields,
{
	let user_id = ctx.user_id();

	// -- Extract fields (name / sea-query value expression)
	let mut fields = data.not_none_fields();
	prep_fields_for_create::<MC>(&mut fields, user_id);

	// -- Build query
	let (columns, sea_values) = fields.for_sea_insert();
	let mut query = Query::insert();
	query
		.into_table(MC::table_ref())
		.columns(columns)
		.values(sea_values)?
		.returning(Query::returning().columns([CommonIden::Id]));

	// -- Exec query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
	// NOTE: For now, we will use the _txn for all create.
	//       We could have a with_txn as function argument if perf is an issue (it should not be)
	let (id,) = dbm.dbx().fetch_one(sqlx_query).await?;

	Ok(id)
}

pub async fn get<MC, E>(_ctx: &Ctx, dbm: &DbManager, id: i64) -> DbResult<E>
where
	MC: DbRepository,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
	E: HasFields,
{
	// -- Build query
	let mut query = Query::select();
	query
		.from(MC::table_ref())
		.columns(E::field_column_refs())
		.and_where(Expr::col(CommonIden::Id).eq(id));

	// -- Exec query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
	let entity =
		dbm.dbx()
			.fetch_optional(sqlx_query)
			.await?
			.ok_or(DbError::EntityNotFound {
				entity: MC::TABLE,
				id,
			})?;

	Ok(entity)
}

pub async fn list<MC, E, F>(
	_ctx: &Ctx,
	dbm: &DbManager,
	filter: Option<F>,
	list_options: Option<ListOptions>,
) -> DbResult<Vec<E>>
where
	MC: DbRepository,
	F: Into<FilterGroups>,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
	E: HasFields,
{
	// -- Build the query
	let mut query = Query::select();
	query.from(MC::table_ref()).columns(E::field_column_refs());

	// condition from filter
	if let Some(filter) = filter {
		let filters: FilterGroups = filter.into();
		let cond: Condition = filters.try_into()?;
		query.cond_where(cond);
	}
	// list options
	let list_options = compute_list_options(list_options)?;
	list_options.apply_to_sea_query(&mut query);

	// -- Execute the query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

	let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
	let entities = dbm.dbx().fetch_all(sqlx_query).await?;

	Ok(entities)
}

pub async fn update<MC, E>(
	ctx: &Ctx,
	dbm: &DbManager,
	id: i64,
	data: E,
) -> DbResult<()>
where
	MC: DbRepository,
	E: HasFields,
{
	// -- Prep Fields
	let mut fields = data.not_none_fields();
	prep_fields_for_update::<MC>(&mut fields, ctx.user_id());

	// -- Build query
	let fields = fields.for_sea_update();
	let mut query = Query::update();
	query
		.table(MC::table_ref())
		.values(fields)
		.and_where(Expr::col(CommonIden::Id).eq(id));

	// -- Execute query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let sqlx_query = sqlx::query_with(&sql, values);
	let count = dbm.dbx().execute(sqlx_query).await?;

	// -- Check result
	if count == 0 {
		Err(DbError::EntityNotFound {
			entity: MC::TABLE,
			id,
		})
	} else {
		Ok(())
	}
}

pub async fn delete<MC>(_ctx: &Ctx, dbm: &DbManager, id: i64) -> DbResult<()>
where
	MC: DbRepository,
{
	// -- Build query
	let mut query = Query::delete();
	query
		.from_table(MC::table_ref())
		.and_where(Expr::col(CommonIden::Id).eq(id));

	// -- Execute query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let sqlx_query = sqlx::query_with(&sql, values);
	let count = dbm.dbx().execute(sqlx_query).await?;

	// -- Check result
	if count == 0 {
		Err(DbError::EntityNotFound {
			entity: MC::TABLE,
			id,
		})
	} else {
		Ok(())
	}
}

pub fn compute_list_options(
	list_options: Option<ListOptions>,
) -> DbResult<ListOptions> {
	if let Some(mut list_options) = list_options {
		// Validate the limit.
		if let Some(limit) = list_options.limit {
			if limit > LIST_LIMIT_MAX {
				return Err(DbError::ListLimitOverMax {
					max: LIST_LIMIT_MAX,
					actual: limit,
				});
			}
		}
		// Set the default limit if no limit
		else {
			list_options.limit = Some(LIST_LIMIT_DEFAULT);
		}
		Ok(list_options)
	}
	// When None, return default
	else {
		Ok(ListOptions {
			limit: Some(LIST_LIMIT_DEFAULT),
			offset: None,
			order_bys: Some("id".into()),
		})
	}
}
