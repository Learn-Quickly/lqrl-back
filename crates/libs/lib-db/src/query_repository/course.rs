use derive_more::Display;
use lib_core::ctx::Ctx;
use modql::field::Fields;
use modql::filter::{FilterNodes, ListOptions, OpValsFloat64, OpValsInt64, OpValsString, OpValsValue};
use sea_query::Nullable;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::FromRow;
use time::OffsetDateTime;
use crate::query_repository::modql_utils::time_to_sea_value;
use lib_utils::time::Rfc3339;

use crate::{base::{self, DbRepository}, store::{error::{DbError, DbResult}, DbManager}};

#[serde_as]
#[derive(Clone, Fields, FromRow, Debug, Serialize)]
pub struct CourseQuery {
	pub id: i64,
	pub title: String,
	pub description: String,
	pub course_type: String,
	pub price: f64,
	pub color: String,
	#[serde_as(as = "Option<Rfc3339>")]
	pub published_date: Option<OffsetDateTime>,
	pub img_url: Option<String>,
	#[field(cast_as = "course_state")]
	pub state: CourseStateQuery,
}

#[derive(Debug, Clone, Display, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "course_state")]
pub enum CourseStateQuery {
    Draft,
    Published,
	Archived,
	None,
}

impl From<CourseStateQuery> for sea_query::Value {
	fn from(value: CourseStateQuery) -> Self {
		value.to_string().into()
	}
}

impl Nullable for CourseStateQuery {
	fn null() -> sea_query::Value {
		CourseStateQuery::None.into()
	}
}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct CourseFilter {
	pub id: Option<OpValsInt64>,
	pub title: Option<OpValsString>,
	pub description: Option<OpValsString>,
	pub course_type: Option<OpValsString>,
	pub price: Option<OpValsFloat64>,
	pub color: Option<OpValsString>,
	pub state: Option<OpValsString>,

	#[modql(to_sea_value_fn = "time_to_sea_value")]
	pub publish_date: Option<OpValsInt64>,

	pub cid: Option<OpValsInt64>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	pub ctime: Option<OpValsValue>,
	pub mid: Option<OpValsInt64>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	pub mtime: Option<OpValsValue>,
}

pub struct CourseQueryRepository;

impl DbRepository for CourseQueryRepository {
    const TABLE: &'static str = "course";
}

impl CourseQueryRepository {
	pub async fn list(
		ctx: &Ctx,
		dbm: &DbManager,
		filter: Option<Vec<CourseFilter>>,
		list_options: Option<ListOptions>,
	) -> DbResult<Vec<CourseQuery>> {
		let result = base::list::<Self, CourseQuery, _>(ctx, dbm, filter, list_options)
			.await
			.map_err(Into::<DbError>::into)?;

		Ok(result)
	}

	pub async fn get(ctx: &Ctx, dbm: &DbManager, id: i64) -> DbResult<CourseQuery>
	{
		let result = base::get::<Self, CourseQuery>(ctx, dbm, id)
			.await
			.map_err(Into::<DbError>::into)?;

		Ok(result)
	}
}