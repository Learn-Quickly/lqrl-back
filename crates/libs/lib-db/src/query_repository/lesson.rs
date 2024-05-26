use lib_core::ctx::Ctx;
use modql::field::{Fields, HasFields};
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::prelude::FromRow;

use crate::{base::{idens::LessonIden, DbRepository}, store::{db_manager::DbManager, error::{DbError, DbResult}}};


#[derive(Clone, Fields, FromRow, Debug)]
pub struct LessonData {
	pub id: i64,
	pub course_id: i64,
    pub title: String,
    pub lesson_order: i32,
}

#[derive(Clone)]
pub struct LessonQueryRepository {
    dbm: DbManager,
}

impl LessonQueryRepository {
    pub fn new(dbm: DbManager) -> Self {
        Self {
            dbm,
        }
    }
}

impl DbRepository for LessonQueryRepository {
    const TABLE: &'static str = "lesson";
}

impl LessonQueryRepository {
    pub async fn get_course_lessons_ordered(
        &self, 
        _: &Ctx, 
        course_id: i64
    ) -> DbResult<Vec<LessonData>> {
        let mut query = Query::select();
        query
            .from(Self::table_ref())
            .columns(LessonData::field_column_refs())
            .and_where(Expr::col(LessonIden::CourseId).eq(course_id))
            .order_by(LessonIden::LessonOrder, sea_query::Order::Asc);
    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, LessonData, _>(&sql, values);
        let lessons =
            self.dbm.dbx()
                .fetch_all(sqlx_query)
                .await
                .map_err(Into::<DbError>::into)?;

        Ok(lessons)
    }
}