use derive_more::Display;
use lib_core::model::course::{UserCourse, UserCourseRole};
use modql::field::{Fields, HasFields};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use crate::{base::DbRepository, store::{dbx::error::DbxError, error::{DbError, DbResult}, DbManager}};

#[derive(Iden)]
pub enum UserCourseIden {
	CourseId,
	UserId,
}

#[derive(Debug, Clone, Display, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "user_course_roles")]
pub enum UserCourseRoleRequest {
    Student,
    Creator,
}

impl From<UserCourseRoleRequest> for sea_query::Value {
	fn from(value: UserCourseRoleRequest) -> Self {
		value.to_string().into()
	}
}

impl From<UserCourseRole> for UserCourseRoleRequest {
	fn from(value: UserCourseRole) -> Self {
		match value {
			UserCourseRole::Creator => UserCourseRoleRequest::Creator,
			UserCourseRole::Student => UserCourseRoleRequest::Student,
		}
	}
}

impl From<UserCourseRoleRequest> for UserCourseRole {
	fn from(value: UserCourseRoleRequest) -> Self {
		match value {
			UserCourseRoleRequest::Creator => UserCourseRole::Creator,
			UserCourseRoleRequest::Student => UserCourseRole::Student,
		}
	}
}

#[derive(Fields, FromRow)]
pub struct UsersCoursesRequest {
    pub user_id: i64,
    pub course_id: i64,
	#[field(cast_as = "user_course_roles")]
    pub user_role: UserCourseRoleRequest,
}

impl From<UsersCoursesRequest> for UserCourse {
	fn from(value: UsersCoursesRequest) -> Self {
		Self {
    		user_id: value.user_id,
    		course_id: value.course_id,
    		user_role: value.user_role.into(),
		}
	}
}

pub struct UsersCoursesForDelete {
    pub user_id: i64,
    pub course_id: i64,
}

pub struct UsersCoursesRepository;

impl DbRepository for UsersCoursesRepository {
    const TABLE: &'static str = "users_courses";

	fn has_timestamps() -> bool {
        false
	}
}

impl UsersCoursesRepository {
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