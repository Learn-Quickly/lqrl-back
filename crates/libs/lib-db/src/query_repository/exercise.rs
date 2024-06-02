use lib_core::ctx::Ctx;
use modql::field::{Fields, HasFields};
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde_json::Value;
use sqlx::FromRow;

use crate::{base::{self, idens::ExerciseIden, DbRepository}, store::{db_manager::DbManager, error::{DbError, DbResult}}};

#[derive(Clone, Fields, FromRow, Debug)]
pub struct ExerciseQuery {
	pub id: i64,
	pub lesson_id: i64,
    pub title: String,
    pub description: String,
    pub exercise_type: String,
    pub exercise_order: i32,
    pub exercise_body: Value,
    pub answer_body: Value,
    pub difficult: String,
    pub time_to_complete: Option<i32>, 
}

#[derive(Clone)]
pub struct ExerciseQueryRepository {
    dbm: DbManager,
}

impl ExerciseQueryRepository {
    pub fn new(dbm: DbManager) -> Self {
        Self {
            dbm,
        }
    }
}

impl DbRepository for ExerciseQueryRepository {
    const TABLE: &'static str = "exercise";
}

impl ExerciseQueryRepository {
    pub async fn get_lesson_exercises(
        &self,
        _: &Ctx, 
        lesson_id: i64
    ) -> DbResult<Vec<ExerciseQuery>> {
        let mut query = Query::select();
        query
            .from(Self::table_ref())
            .columns(ExerciseQuery::field_column_refs())
            .and_where(Expr::col(ExerciseIden::LessonId).eq(lesson_id))
            .order_by(ExerciseIden::ExerciseOrder, sea_query::Order::Asc);
    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, ExerciseQuery, _>(&sql, values);

        let exercises =
            self.dbm.dbx()
                .fetch_all(sqlx_query)
                .await
                .map_err(Into::<DbError>::into)?;

        Ok(exercises)
    }

    pub async fn get_exercise(
        &self,
        ctx: &Ctx,
        exercise_id: i64,
    ) -> DbResult<ExerciseQuery> {
        base::get::<Self, ExerciseQuery>(ctx, &self.dbm, exercise_id)
            .await
            .map_err(Into::<DbError>::into)
    }
}
