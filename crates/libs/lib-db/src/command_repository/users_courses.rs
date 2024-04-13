use lib_core::{interactors::error::CoreError, models::course::UserCourse};
use modql::field::{Fields, HasFields};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::prelude::FromRow;
use crate::{base::DbRepository, store::{db_manager::DbManager, dbx::error::DbxError, error::{DbError, DbResult}}};

#[derive(Iden)]
pub enum UserCourseIden {
	CourseId,
	UserId,
}

#[derive(Fields, FromRow)]
pub struct UsersCoursesRequest {
    pub user_id: i64,
    pub course_id: i64,
    pub user_role: String,
}

impl TryFrom<UsersCoursesRequest> for UserCourse {
	type Error = CoreError;
	
	fn try_from(value: UsersCoursesRequest) -> Result<Self, Self::Error> {
		let user_role = value.user_role.try_into()?;

		let result = Self {
    		user_id: value.user_id,
    		course_id: value.course_id,
    		user_role,
		};

		Ok(result)
	}
}

pub struct UsersCoursesForDelete {
    pub user_id: i64,
    pub course_id: i64,
}

pub struct UsersCoursesCommandRepository;

impl DbRepository for UsersCoursesCommandRepository {
    const TABLE: &'static str = "users_courses";

	fn has_timestamps() -> bool {
        false
	}
}

impl UsersCoursesCommandRepository {
    pub async fn create(
        dbm: &DbManager,
        users_courses_c: UsersCoursesRequest,
    ) -> DbResult<()> {
	    let fields = users_courses_c.not_none_fields();
		
	    let (columns, sea_values) = fields.for_sea_insert();
	    let mut query = Query::insert();
	    query
		    .into_table(Self::table_ref())
		    .columns(columns)
		    .values(sea_values)
			.map_err(DbxError::SeaQuery)?
			.returning(Query::returning().columns([UserCourseIden::CourseId, UserCourseIden::UserId]));

	    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	    let sqlx_query = sqlx::query_as_with::<_, (i64, i64), _>(&sql, values);

	    let (_, _) = dbm.dbx().fetch_one(sqlx_query).await?;

	    Ok(())
    }

	pub async fn get(
		dbm: &DbManager,
		user_id: i64,
		course_id: i64,
	) -> DbResult<UsersCoursesRequest> {
		let mut query = Query::select();
		query
			.from(Self::table_ref())
			.columns(UsersCoursesRequest::field_column_refs())
			.and_where(Expr::col(UserCourseIden::UserId).eq(user_id))
			.and_where(Expr::col(UserCourseIden::CourseId).eq(course_id));

		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
		let sqlx_query = sqlx::query_as_with::<_, UsersCoursesRequest, _>(&sql, values);
		let entity =
			dbm.dbx()
				.fetch_optional(sqlx_query)
				.await?
				.ok_or(DbError::UserCourseNotFound { 
					entity: Self::TABLE.to_string(), 
					user_id, 
					course_id,
				})?;

		Ok(entity)
	}

	pub async fn get_optional(
		dbm: &DbManager, 
		user_id: i64, 
		course_id: i64
	) -> DbResult<Option<UsersCoursesRequest>> {
		let mut query = Query::select();
		query
			.from(Self::table_ref())
			.columns(UsersCoursesRequest::field_column_refs())
			.and_where(Expr::col(UserCourseIden::UserId).eq(user_id))
			.and_where(Expr::col(UserCourseIden::CourseId).eq(course_id));

		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
		let sqlx_query = sqlx::query_as_with::<_, UsersCoursesRequest, _>(&sql, values);
		let entity =
			dbm.dbx()
				.fetch_optional(sqlx_query)
				.await?;

		Ok(entity)
	}
	
	pub async fn delete(
		dbm: &DbManager,
		users_courses_d: UsersCoursesForDelete,
	) -> DbResult<()> {
		// -- Build query
		let mut query = Query::delete();
		query
			.from_table(Self::table_ref())
			.and_where(Expr::col(UserCourseIden::UserId).eq(users_courses_d.user_id))
			.and_where(Expr::col(UserCourseIden::CourseId).eq(users_courses_d.course_id));

		// -- Execute query
		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
		let sqlx_query = sqlx::query_with(&sql, values);
		let count = dbm.dbx().execute(sqlx_query).await?;

		// -- Check result
		if count == 0 {
			Err(DbError::UserCourseNotFound {
				entity: Self::TABLE.to_string(),
				user_id: users_courses_d.user_id,
				course_id: users_courses_d.course_id,
			}) 
		} else {
			Ok(())
		}
	}
}