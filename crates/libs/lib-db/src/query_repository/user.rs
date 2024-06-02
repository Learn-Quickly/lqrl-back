use lib_core::{ctx::Ctx, models::course::UserCourseRole};
use modql::{field::{Fields, HasFields}, filter::{FilterNodes, ListOptions, OpValsInt64, OpValsString, OpValsValue}};
use time::OffsetDateTime;
use uuid::Uuid;
use crate::{base::{compute_list_options, idens::{CommonIden, LessonProgressIden, UserCourseIden, UserIden}, table_ref::{get_lesson_progress_table_ref, get_user_table_ref, get_users_courses_table_ref}}, query_repository::modql_utils::time_to_sea_value, store::db_manager::DbManager};
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::Deserialize;
use sqlx::{postgres::PgRow, FromRow};

use crate::{base::{self, DbRepository}, store::error::{DbError, DbResult}};

use super::lesson::LessonQueryRepository;

#[derive(FromRow, Fields)]
pub struct User {
	pub id: i64,
	pub date_registered: OffsetDateTime,
	pub username: String,
}

pub struct UserStats {
	pub id: i64,
	pub username: String,
	pub date_registered: i64,
	pub number_of_completed_lessons: i64,
}

#[derive(Clone, Fields, FromRow, Debug)]
pub struct UserData {
	pub id: i64,
	pub username: String,

	pub pwd: Option<String>, 
	pub pwd_salt: Uuid,
	pub token_salt: Uuid,
}

pub trait UserBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for User {}
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
		let mut query = Query::select();
		query
			.from(Self::table_ref())
			.columns(E::field_idens())
			.and_where(Expr::col(UserIden::Username).eq(username));

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

	pub async fn get_attendants(
		&self,
		ctx: &Ctx,
		course_id: i64,
		list_options: Option<ListOptions>,
	) -> DbResult<Vec<UserStats>> {
		let mut query = Query::select();

		let users_courses_table = get_users_courses_table_ref();
		let user_table = get_user_table_ref();

		query.from(users_courses_table)
			.inner_join(user_table, 
				Expr::col((UserCourseIden::UsersCourses, UserCourseIden::UserId))
				.equals((UserIden::User, CommonIden::Id))
			)
			.columns(User::field_column_refs())
			.and_where(Expr::col(UserCourseIden::CourseId).eq(course_id))
			.and_where(Expr::col(UserCourseIden::UserRole).eq(UserCourseRole::Student.to_string()));
	
		let list_options = compute_list_options(list_options)?;
		list_options.apply_to_sea_query(&mut query);
	
		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	
		let sqlx_query = sqlx::query_as_with::<_, User, _>(&sql, values);
		let users = self.dbm.dbx().fetch_all(sqlx_query).await?;
		
		let lesson_repo = LessonQueryRepository::new(self.dbm.clone());
		let lesson_ids: Vec<i64> = lesson_repo.get_course_lessons_ordered(ctx, course_id)
			.await?
			.iter()
			.map(|lesson| lesson.id)
			.collect();

		let mut result = Vec::new();
		for user in users {
			let number_of_completed_lessons = self.get_number_of_completed_lessons(user.id, lesson_ids.clone()).await?;
			result.push(UserStats {
    			id: user.id,
    			username: user.username,
    			number_of_completed_lessons,
    			date_registered: user.date_registered.unix_timestamp(),
			})
		}
	
		Ok(result)
	}

	async fn get_number_of_completed_lessons(
		&self, 
		user_id: i64, 
		lesson_ids: Vec<i64>
	) -> DbResult<i64> {
		let mut query = Query::select();

		let lesson_progress_tbl = get_lesson_progress_table_ref();

		query
			.expr(Expr::col((LessonProgressIden::LessonProgress, LessonProgressIden::LessonId)).count())
			.from(lesson_progress_tbl)
			.and_where(Expr::col(LessonProgressIden::LessonId).is_in(lesson_ids))
			.and_where(Expr::col(LessonProgressIden::UserId).eq(user_id))
			.and_where(Expr::col(LessonProgressIden::State).eq("Done"));

		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

		let sqlx_query = sqlx::query_scalar_with::<_, i64, _>(&sql, values);
		let result = self.dbm.dbx().fetch_one_scalar(sqlx_query).await?;

		Ok(result)
	}

	pub async fn get_number_of_attendance(&self, course_id: i64) -> DbResult<i64> {
		let mut query = Query::select();

		let users_courses_table = get_users_courses_table_ref();

		query.from(users_courses_table)
			.expr(Expr::col((UserCourseIden::UsersCourses, UserCourseIden::UserId)).count())
			.and_where(Expr::col(UserCourseIden::CourseId).eq(course_id))
			.and_where(Expr::col(UserCourseIden::UserRole).eq(UserCourseRole::Student.to_string()));

		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

		let sqlx_query = sqlx::query_scalar_with::<_, i64, _>(&sql, values);
		let entities = self.dbm.dbx().fetch_one_scalar(sqlx_query).await?;

		Ok(entities)
	}
}