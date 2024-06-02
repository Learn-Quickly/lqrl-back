use lib_core::{ctx::Ctx, interactors::error::ExerciseError, interfaces::exercise::ExerciseResult, models::exercise_completion::{ExerciseCompletion, ExerciseCompletionForCompleteCommand, ExerciseCompletionForCreate, ExerciseCompletionForUpdate}};
use modql::field::{Fields, HasFields};
use sea_query::{Expr, PostgresQueryBuilder, Query, Value};
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

#[derive(Fields)]
struct ExerciseCompletionForSaveChanges {
    pub date_last_changes: i64,
    pub body: Value,
}

#[derive(Fields)]
struct ExerciseCompletionForComplete {
    pub points_scored: f32,
    pub max_points: f32,
    pub state: String,
}

#[derive(Fields, FromRow)]
struct ExerciseCompletionQuery {
    pub id: i64,
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
            date_last_changes: value.date_last_changes,
            state: value.state.try_into()?,
            body: value.body.clone(),
            max_points: value.max_points,
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

    pub async fn get(
        ctx: &Ctx,
        dbm: &DbManager, 
        id: i64,
    ) -> ExerciseResult<ExerciseCompletion> {
        let result = base::get::<Self, ExerciseCompletionQuery>(ctx, dbm, id)
            .await
            .map_err(Into::<DbError>::into)?;

        Ok(result.try_into()?)
    }

    pub async fn create_exercise_completion(
        ctx: &Ctx, 
        dbm: &DbManager, 
        ex_comp_for_c: ExerciseCompletionForCreate,
    ) -> ExerciseResult<i64> {
        let ex_comp_for_c = ExerciseCompletionData {
            exercise_id: ex_comp_for_c.exercise_id,
            user_id: ex_comp_for_c.user_id,
            number_of_attempts: ex_comp_for_c.number_of_attempts as i32,
            date_started: ex_comp_for_c.date_started,
        };

        let id = base::create::<Self, ExerciseCompletionData>(ctx, dbm, ex_comp_for_c).await.map_err(Into::<DbError>::into)?;

        Ok(id)
    }

    pub async fn update_exercise_completion(
        dbm: &DbManager, 
        ctx: &Ctx, 
        ex_comp_for_u: ExerciseCompletionForUpdate,
    ) -> ExerciseResult<()> {
        let ex_comp_for_u = ExerciseCompletionForSaveChanges {
            date_last_changes: ex_comp_for_u.date_last_changes,
            body: Value::Json(Some(Box::new(ex_comp_for_u.body))),
        };

        base::create::<Self, ExerciseCompletionForSaveChanges>(ctx, dbm, ex_comp_for_u).await.map_err(Into::<DbError>::into)?;
        
        Ok(())
    }

    pub async fn complete_exercise(
        dbm: &DbManager,
        ctx: &Ctx,
        ex_comp_for_u: ExerciseCompletionForCompleteCommand,
    ) -> ExerciseResult<()> {
        let ex_comp_for_u = ExerciseCompletionForComplete {
            points_scored: ex_comp_for_u.points_scored,
            max_points: ex_comp_for_u.max_points,
            state: ex_comp_for_u.state.to_string(),
        };

        base::create::<Self, ExerciseCompletionForComplete>(ctx, dbm, ex_comp_for_u).await.map_err(Into::<DbError>::into)?;
        
        Ok(())
    }
}