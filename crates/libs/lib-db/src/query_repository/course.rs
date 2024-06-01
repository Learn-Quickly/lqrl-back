use lib_core::ctx::Ctx;
use modql::field::{Fields, HasFields};
use modql::filter::{FilterGroups, FilterNodes, ListOptions, OpValsFloat64, OpValsInt64, OpValsString, OpValsValue};
use sea_query::{Condition, Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::FromRow;
use time::OffsetDateTime;
use crate::base::compute_list_options;
use crate::base::idens::{CommonIden, CourseIden, UserCourseIden};
use crate::base::table_ref::get_users_courses_table_ref;
use crate::query_repository::modql_utils::time_to_sea_value;
use crate::store::db_manager::DbManager;
use crate::store::dbx::error::DbxError;
use lib_utils::time::Rfc3339;

use crate::{base::{self, DbRepository}, store::error::{DbError, DbResult}};

use super::users_courses::UsersCoursesQueryRepository;

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
	pub state: String,
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

#[derive(Clone)]
pub struct CourseQueryRepository {
	dbm: DbManager,
} 

impl CourseQueryRepository {
	pub fn new(dbm: DbManager) -> Self {
		Self {
    		dbm,
		}
	}
}

impl DbRepository for CourseQueryRepository {
    const TABLE: &'static str = "course";
}

impl CourseQueryRepository {
	pub async fn list(
		&self, 
		_: &Ctx,
		user_id: i64,
		filter: Option<Vec<CourseFilter>>,
		list_options: Option<ListOptions>,
	) -> DbResult<Vec<CourseQuery>> {
		let mut query = Query::select();
		query.from(Self::table_ref()).columns(CourseQuery::field_column_refs())
			.left_join(
				get_users_courses_table_ref(), 
				Expr::col((UserCourseIden::UsersCourses, UserCourseIden::CourseId))
				.equals((CourseIden::Course, CommonIden::Id))
			)
			.cond_where(
				Condition::any()
					.add(
						Condition::all()
							.add(Expr::col((UserCourseIden::UsersCourses, UserCourseIden::UserId)).ne(user_id))
					)
					.add(
						Condition::all()
							.add(Expr::col((UserCourseIden::UsersCourses, UserCourseIden::UserId)).eq(user_id))
							.add(Expr::col((UserCourseIden::UsersCourses, UserCourseIden::UserRole)).ne("Creator"))
					)
			);

		if let Some(filter) = filter {
			let filters: FilterGroups = filter.into();
			let cond: Condition = filters.try_into().map_err(Into::<DbxError>::into)?;
			query.cond_where(cond);
		}


		let list_options = compute_list_options(list_options)?;
		list_options.apply_to_sea_query(&mut query);

		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

		let sqlx_query = sqlx::query_as_with::<_, CourseQuery, _>(&sql, values);
		let entities = self.dbm.dbx().fetch_all(sqlx_query).await?;

		Ok(entities)
	}

	pub async fn count(
		&self, 
		_: &Ctx,
		user_id: i64,
		filter: Option<Vec<CourseFilter>>,
	) -> DbResult<i64> {
		let mut query = Query::select();
		query
			.expr(Expr::col((CourseIden::Course, CommonIden::Id)).count())
			.from(Self::table_ref())
			.left_join(
				get_users_courses_table_ref(), 
				Expr::col((UserCourseIden::UsersCourses, UserCourseIden::CourseId))
				.equals((CourseIden::Course, CommonIden::Id))
			)
			.cond_where(
				Condition::any()
					.add(
						Condition::all()
							.add(Expr::col((UserCourseIden::UsersCourses, UserCourseIden::UserId)).ne(user_id))
					)
					.add(
						Condition::all()
							.add(Expr::col((UserCourseIden::UsersCourses, UserCourseIden::UserId)).eq(user_id))
							.add(Expr::col((UserCourseIden::UsersCourses, UserCourseIden::UserRole)).ne("Creator"))
					)
			);

		if let Some(filter) = filter {
			let filters: FilterGroups = filter.into();
			let cond: Condition = filters.try_into().map_err(Into::<DbxError>::into)?;
			query.cond_where(cond);
		}

		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

		let sqlx_query = sqlx::query_scalar_with::<_, i64, _>(&sql, values);
		let entities = self.dbm.dbx().fetch_one_scalar(sqlx_query).await?;

		Ok(entities)
	}

	pub async fn get(&self, ctx: &Ctx, id: i64) -> DbResult<CourseQuery>
	{
		let result = base::get::<Self, CourseQuery>(ctx, &self.dbm, id)
			.await
			.map_err(Into::<DbError>::into)?;

		Ok(result)
	}

	pub async fn get_user_courses_registered(
		&self,
		_ctx: &Ctx,
		user_id: i64,
	) -> DbResult<Vec<CourseQuery>> {
		let user_courses = UsersCoursesQueryRepository::get_user_courses(
			&self.dbm,
			user_id, 
			super::users_courses::UserCourseRoleQuery::Student
		).await?;

		let courses_ids: Vec<i64> = user_courses.iter().map(|user_course| user_course.course_id).collect();

		let mut query = Query::select();
		query
			.from(Self::table_ref())
			.columns(CourseQuery::field_column_refs())
			.and_where(Expr::col(CommonIden::Id).is_in(courses_ids));

		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
		let sqlx_query = sqlx::query_as_with::<_, CourseQuery, _>(&sql, values);
		let result = self.dbm.dbx()
			.fetch_all(sqlx_query)
			.await?;

		Ok(result)
	}

	pub async fn get_user_courses_created(
		&self,
		_ctx: &Ctx,
		user_id: i64,
	) -> DbResult<Vec<CourseQuery>> {
		let user_courses = UsersCoursesQueryRepository::get_user_courses(
			&self.dbm,
			user_id, 
			super::users_courses::UserCourseRoleQuery::Creator
		).await?;

		let courses_ids: Vec<i64> = user_courses.iter().map(|user_course| user_course.course_id).collect();

		let mut query = Query::select();
		query
			.from(Self::table_ref())
			.columns(CourseQuery::field_column_refs())
			.and_where(Expr::col(CommonIden::Id).is_in(courses_ids));

		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
		let sqlx_query = sqlx::query_as_with::<_, CourseQuery, _>(&sql, values);
		let result = self.dbm.dbx()
			.fetch_all(sqlx_query)
			.await?;

		Ok(result)
	}
}