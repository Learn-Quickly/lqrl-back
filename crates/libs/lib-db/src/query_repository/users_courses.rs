use derive_more::Display;
use modql::field::{Fields, HasFields};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use crate::{base::DbRepository, store::{error::DbResult, DbManager}};

#[derive(Iden)]
pub enum UserCourseIden {
	UserId,
    UserRole,
}

#[derive(Debug, Clone, Display, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "user_course_roles")]
pub enum UserCourseRoleQuery {
    Student,
    Creator,
}

impl From<UserCourseRoleQuery> for sea_query::Value {
	fn from(value: UserCourseRoleQuery) -> Self {
		value.to_string().into()
	}
}


#[derive(Fields, FromRow)]
pub struct UsersCoursesQuery {
    pub user_id: i64,
    pub course_id: i64,
	#[field(cast_as = "user_course_roles")]
    pub user_role: UserCourseRoleQuery,
}


pub struct UsersCoursesQueryRepository;

impl DbRepository for UsersCoursesQueryRepository {
    const TABLE: &'static str = "users_courses";

	fn has_timestamps() -> bool {
        false
	}
}

impl UsersCoursesQueryRepository {
    pub async fn get_user_courses(
        dbm: &DbManager,
        user_id: i64,
        user_role: UserCourseRoleQuery,
    ) -> DbResult<Vec<UsersCoursesQuery>> {
		let mut query = Query::select();
		query
			.from(Self::table_ref())
			.columns(UsersCoursesQuery::field_column_refs())
			.and_where(Expr::col(UserCourseIden::UserId).eq(user_id))
			.and_where(Expr::col(UserCourseIden::UserRole).eq(user_role));

		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
		let sqlx_query = sqlx::query_as_with::<_, UsersCoursesQuery, _>(&sql, values);
		let entity =
			dbm.dbx()
				.fetch_all(sqlx_query)
				.await?;

		Ok(entity)
    }
}