use derive_more::Display;
use modql::field::{Fields, HasFields};
use sea_query::{Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use crate::model::Result;

use super::{base::DbBmc, ModelManager};

#[derive(Iden)]
pub enum UserCourseIden {
	CourseId,
	UserId,
}

#[derive(Debug, Clone, Display, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "user_course_roles")]
pub enum UserCourseRole {
    Student,
    Creator,
}

impl From<UserCourseRole> for sea_query::Value {
	fn from(value: UserCourseRole) -> Self {
		value.to_string().into()
	}
}

#[derive(Fields)]
pub struct UsersCoursesForCreate {
    pub(crate) user_id: i64,
    pub(crate) course_id: i64,
	#[field(cast_as = "user_course_roles")]
    pub(crate) user_role: UserCourseRole,
}

pub struct UsersCoursesBmc;

impl DbBmc for UsersCoursesBmc {
    const TABLE: &'static str = "users_courses";

	fn has_timestamps() -> bool {
        false
	}
}

impl UsersCoursesBmc {
    pub async fn create(
        mm: &ModelManager,
        users_courses_c: UsersCoursesForCreate,
    ) -> Result<()> {
	    let fields = users_courses_c.not_none_fields();

	    let (columns, sea_values) = fields.for_sea_insert();
	    let mut query = Query::insert();
	    query
		    .into_table(Self::table_ref())
		    .columns(columns)
		    .values(sea_values)?
			.returning(Query::returning().columns([UserCourseIden::CourseId, UserCourseIden::UserId]));

	    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	    let sqlx_query = sqlx::query_as_with::<_, (i64, i64), _>(&sql, values);

	    let (_, _) = mm.dbx().fetch_one(sqlx_query).await?;

	    Ok(())
    }
}