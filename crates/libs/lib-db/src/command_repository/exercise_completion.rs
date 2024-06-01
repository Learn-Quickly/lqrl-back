use lib_core::{ctx::Ctx, interactors::error::ExerciseError, interfaces::exercise::ExerciseResult, models::exercise_completion::{ExerciseCompletion, ExerciseCompletionForCreate}};
use modql::field::{Fields, HasFields};
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, prelude::FromRow};

use crate::{base::{self, idens::ExerciseCompletionIden, DbRepository}, store::{db_manager::DbManager, error::DbError}};

#[derive(Fields)]
struct ExerciseCompletionData {
    pub exercise_id: i64,
    pub user_id: i64,
    pub number_of_attempts: i32,
    pub date_started: i64,
}

#[derive(Fields, FromRow)]
struct ExerciseCompletionQuery {
    pub id: i64,
    pub exercise_id: i64,
    pub user_id: i64,
    pub points_scored: Option<i32>,
    pub number_of_attempts: i32,
    pub date_started: i64,
    pub date_completed: Option<i64>,
    pub state: String,   
    pub body: serde_json::Value,
}

impl TryFrom<ExerciseCompletionQuery> for ExerciseCompletion {
    type Error = ExerciseError;

    fn try_from(value: ExerciseCompletionQuery) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            exercise_id: value.exercise_id,
            user_id: value.user_id,
            points_scored: value.points_scored,
            number_of_attempts: value.number_of_attempts,
            date_started: value.date_started,
            date_completed: value.date_completed,
            state: value.state.try_into()?,
            body: value.body.clone(),
        })
    }
}

pub trait ExerciseCompletionBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl ExerciseCompletionBy for ExerciseCompletionQuery {}

pub struct ExerciseCompletionCommandRepository;

impl DbRepository for ExerciseCompletionCommandRepository {
    const TABLE: &'static str = "exercise_completion";
}

impl ExerciseCompletionCommandRepository { 
    pub async fn get_exercise_user_completions(
        _: &Ctx,
        dbm: &DbManager, 
        user_id: i64, 
        exercise_id: i64,
    ) -> ExerciseResult<Vec<ExerciseCompletion>> {
        let mut query = Query::select();
        query
            .from(Self::table_ref())
            .columns(ExerciseCompletionQuery::field_column_refs())
            .and_where(Expr::col(ExerciseCompletionIden::UserId).eq(user_id))
            .and_where(Expr::col(ExerciseCompletionIden::ExerciseId).eq(exercise_id));
    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, ExerciseCompletionQuery, _>(&sql, values);
        let entities =
            dbm.dbx()
                .fetch_all(sqlx_query)
                .await.map_err(Into::<DbError>::into)?;
        
        let mut result = Vec::new();

        for ex_comp in entities {
            result.push(ex_comp.try_into()?);
        }

        Ok(result)
    }

    pub async fn create_exercise_completion(
        ctx: &Ctx, 
        dbm: &DbManager, 
        ex_comp_for_c: ExerciseCompletionForCreate,
    ) -> ExerciseResult<()> {
        let ex_comp_for_c = ExerciseCompletionData {
            exercise_id: ex_comp_for_c.exercise_id,
            user_id: ex_comp_for_c.user_id,
            number_of_attempts: ex_comp_for_c.number_of_attempts as i32,
            date_started: ex_comp_for_c.date_started,
        };

        base::create::<Self, ExerciseCompletionData>(ctx, dbm, ex_comp_for_c).await.map_err(Into::<DbError>::into)?;

        Ok(())
    }
}