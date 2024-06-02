use lib_core::ctx::Ctx;
use modql::field::{Fields, HasFields};
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::FromRow;

use crate::{base::{idens::{CommonIden, ExerciseCompletionIden, ExerciseIden}, table_ref::get_exercise_table_ref, DbRepository}, store::{db_manager::DbManager, error::{DbError, DbResult}}};

#[derive(Clone, Fields, FromRow, Debug)]
pub struct ExerciseCompletionQuery {
    pub exercise_completion_id: i64,
    pub exercise_id: i64,
    pub user_id: i64,
    pub points_scored: Option<f32>,
    pub max_points: Option<f32>,
    pub number_of_attempts: i32,
    pub date_started: i64,
    pub date_last_changes: Option<i64>,
    pub state: String,   
    pub body: serde_json::Value,
}

#[derive(Clone)]
pub struct ExerciseCompletionQueryRepository {
    dbm: DbManager,
}

impl ExerciseCompletionQueryRepository {
    pub fn new(dbm: DbManager) -> Self {
        Self {
            dbm,
        }
    }
}

impl DbRepository for ExerciseCompletionQueryRepository {
    const TABLE: &'static str = "exercise_completion";
}

impl ExerciseCompletionQueryRepository {
    pub async fn get_exercise_completions(
        &self,
        _: &Ctx, 
        exercise_id: i64,
        user_id: i64,
    ) -> DbResult<Vec<ExerciseCompletionQuery>> {
        let mut query = Query::select();
        query
            .from(Self::table_ref())
            .columns(ExerciseCompletionQuery::field_column_refs())
            .and_where(Expr::col(ExerciseCompletionIden::ExerciseId).eq(exercise_id))
            .and_where(Expr::col(ExerciseCompletionIden::UserId).eq(user_id));
    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, ExerciseCompletionQuery, _>(&sql, values);

        let exercises =
            self.dbm.dbx()
                .fetch_all(sqlx_query)
                .await
                .map_err(Into::<DbError>::into)?;

        Ok(exercises)
    }

    pub async fn get_exercises_completions(
        &self,
        _: &Ctx, 
        lesson_id: i64,
        user_id: i64,
    ) -> DbResult<Vec<ExerciseCompletionQuery>> {
        let mut query = Query::select();
        query
            .from(Self::table_ref())
            .columns(ExerciseCompletionQuery::field_column_refs())
            .inner_join(get_exercise_table_ref(), 
                Expr::col((ExerciseCompletionIden::ExerciseCompletion, ExerciseCompletionIden::ExerciseCompletionId))
                .equals((ExerciseIden::Exercise, CommonIden::Id))
            )
            .and_where(Expr::col(ExerciseCompletionIden::UserId).eq(user_id))
            .and_where(Expr::col(ExerciseIden::LessonId).eq(lesson_id));
    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, ExerciseCompletionQuery, _>(&sql, values);

        let exercises =
            self.dbm.dbx()
                .fetch_all(sqlx_query)
                .await
                .map_err(Into::<DbError>::into)?;

        Ok(exercises)
    }
}
