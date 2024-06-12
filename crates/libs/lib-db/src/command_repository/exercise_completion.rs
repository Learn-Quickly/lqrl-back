use lib_core::{ctx::Ctx, interactors::error::ExerciseError, interfaces::exercise::ExerciseResult, models::exercise_completion::{ExerciseCompletion, ExerciseCompletionForCompleteCommand, ExerciseCompletionForCreate, ExerciseCompletionForUpdate}};
use lib_utils::time::from_unix_timestamp;
use modql::field::{Fields, HasFields};
use sea_query::{Expr, PostgresQueryBuilder, Query, Value};
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, prelude::FromRow};
use time::OffsetDateTime;

use crate::{base::{self, idens::ExerciseCompletionIden, prep_fields_for_create, prep_fields_for_update, DbRepository}, store::{db_manager::DbManager, dbx::error::DbxError, error::DbError}};

#[derive(Fields)]
struct ExerciseCompletionData {
    pub exercise_id: i64,
    pub user_id: i64,
    pub number_of_attempts: i32,
    pub date_started: OffsetDateTime,
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

impl TryFrom<ExerciseCompletionQuery> for ExerciseCompletion {
    type Error = ExerciseError;

    fn try_from(value: ExerciseCompletionQuery) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.exercise_completion_id,
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
            date_started: from_unix_timestamp(ex_comp_for_c.date_started)?,
        };
        let user_id = ctx.user_id();

        let mut fields = ex_comp_for_c.not_none_fields();
        prep_fields_for_create::<Self>(&mut fields, user_id);
    
        let (columns, sea_values) = fields.for_sea_insert();
        let mut query = Query::insert();
        query
            .into_table(Self::table_ref())
            .columns(columns)
            .values(sea_values)
			.map_err(DbxError::SeaQuery)
            .map_err(Into::<DbError>::into)?
            .returning(Query::returning().columns([ExerciseCompletionIden::ExerciseCompletionId]));
    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);

        let (id,) = dbm.dbx().fetch_one(sqlx_query).await.map_err(Into::<DbError>::into)?;

        Ok(id)
    }

    pub async fn update_exercise_completion(
        dbm: &DbManager, 
        ctx: &Ctx, 
        ex_comp_for_u: ExerciseCompletionForUpdate,
    ) -> ExerciseResult<()> {
        let ex_comp_for_u_req = ExerciseCompletionForSaveChanges {
            date_last_changes: ex_comp_for_u.date_last_changes,
            body: Value::Json(Some(Box::new(ex_comp_for_u.body))),
        };
	    let mut fields = ex_comp_for_u_req.not_none_fields();
	    prep_fields_for_update::<Self>(&mut fields, ctx.user_id());

	    let fields = fields.for_sea_update();
	    let mut query = Query::update();
	    query
		    .table(Self::table_ref())
		    .values(fields)
		    .and_where(Expr::col(ExerciseCompletionIden::ExerciseCompletionId).eq(ex_comp_for_u.id));

	    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	    let sqlx_query = sqlx::query_with(&sql, values);
	    let count = dbm.dbx().execute(sqlx_query).await.map_err(Into::<DbError>::into)?;

	    if count == 0 {
		    Err(DbError::EntityNotFound {
			    entity: Self::TABLE.to_string(),
			    id: ex_comp_for_u.id,
		    }.into())
	    } else {
		    Ok(())
	    }
    }

    pub async fn complete_exercise(
        dbm: &DbManager,
        ctx: &Ctx,
        ex_comp_for_u: ExerciseCompletionForCompleteCommand,
    ) -> ExerciseResult<()> {
        let ex_comp_for_u_req = ExerciseCompletionForComplete {
            points_scored: ex_comp_for_u.points_scored,
            max_points: ex_comp_for_u.max_points,
            state: ex_comp_for_u.state.to_string(),
        };

	    let mut fields = ex_comp_for_u_req.not_none_fields();
	    prep_fields_for_update::<Self>(&mut fields, ctx.user_id());

	    let fields = fields.for_sea_update();
	    let mut query = Query::update();
	    query
		    .table(Self::table_ref())
		    .values(fields)
		    .and_where(Expr::col(ExerciseCompletionIden::ExerciseCompletionId).eq(ex_comp_for_u.id));

	    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	    let sqlx_query = sqlx::query_with(&sql, values);
	    let count = dbm.dbx().execute(sqlx_query).await.map_err(Into::<DbError>::into)?;

	    if count == 0 {
		    Err(DbError::EntityNotFound {
			    entity: Self::TABLE.to_string(),
			    id: ex_comp_for_u.id,
		    }.into())
	    } else {
		    Ok(())
	    }        
    }

    pub async fn get_uncompleted_exercises(
        dbm: &DbManager,
        _: &Ctx,
    ) -> ExerciseResult<Vec<ExerciseCompletion>> {
        let mut query = Query::select();
        query
            .from(Self::table_ref())
            .columns(ExerciseCompletionQuery::field_column_refs())
            .and_where(Expr::col(ExerciseCompletionIden::State).eq("InProgress"));
    
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
}