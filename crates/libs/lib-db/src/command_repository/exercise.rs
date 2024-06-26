use async_trait::async_trait;
use lib_core::{ctx::Ctx, interactors::error::CoreError, interfaces::exercise::{ExerciseResult, IExerciseCommandRepository}, models::{exercise::{ExerciseForChangeOrder, ExerciseForCreateCommand}, exercise_completion::{ExerciseCompletion, ExerciseCompletionForCompleteCommand, ExerciseCompletionForCreate, ExerciseCompletionForUpdate}}};
use modql::field::{Fields, HasFields};
use sea_query::{Alias, Expr, PostgresQueryBuilder, Query, Value};
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, prelude::FromRow};

use crate::{base::{self, idens::{CommonIden, ExerciseCompletionIden, ExerciseIden}, table_ref::get_exercise_completion_table_ref, DbRepository}, store::{db_manager::DbManager, error::DbError}};

use super::exercise_completion::ExerciseCompletionCommandRepository;

#[derive(Fields)]
struct Exercise {
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

#[derive(Fields, FromRow)]
struct ExerciseData {
    pub id: i64,
	pub lesson_id: i64,
    pub title: String,
    pub description: String,
    pub exercise_type: String,
    pub exercise_order: i32,
    pub exercise_body: serde_json::Value,
    pub answer_body: serde_json::Value,
    pub difficult: String,
    pub time_to_complete: Option<i32>,
}

impl TryFrom<ExerciseData> for lib_core::models::exercise::Exercise {
    type Error = CoreError;

    fn try_from(value: ExerciseData) -> Result<Self, Self::Error> {
        Ok(Self {
            lesson_id: value.lesson_id,
            title: value.title,
            description: value.description,
            exercise_type: value.exercise_type.try_into()?,
            difficult: value.difficult.try_into()?,
            time_to_complete: value.time_to_complete,
            exercise_order: value.exercise_order,
            answer_body: value.answer_body,
            exercise_body: value.exercise_body,
        })
    }
}

#[derive(Fields)]
struct ExerciseForUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub exercise_type: Option<String>,
    pub difficult: Option<String>,
    pub time_to_complete: Option<i64>,  
}

#[derive(Fields)]
struct ExerciseForUpdateAnswerBody {
    pub answer_body: Value,
}

#[derive(Fields)]
struct ExerciseForUpdateExerciseBody {
    pub exercise_body: Value,
}

#[derive(Fields)]
struct ExerciseForUpdateOrder {
    pub exercise_order: i32,
}

pub trait ExerciseBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl ExerciseBy for ExerciseData {}

#[derive(Clone)]
pub struct ExerciseCommandRepository {
    dbm: DbManager,
}

impl DbRepository for ExerciseCommandRepository {
	const TABLE: &'static str = "exercise";
}

impl ExerciseCommandRepository {
	pub fn new(dbm: DbManager) -> Self {
		Self {
			dbm,
		}
	}

    async fn update_exercise_body(&self, ctx: &Ctx, body: Option<serde_json::Value>, exercise_id: i64) -> ExerciseResult<()> {
        if let Some(body) = body {
            let exercise_for_u_b = ExerciseForUpdateExerciseBody { 
                exercise_body: Value::Json(Some(Box::new(body))),
            };

		    base::update::<Self, ExerciseForUpdateExerciseBody>(&ctx, &self.dbm, exercise_id, exercise_for_u_b)
			    .await
			    .map_err(Into::<DbError>::into)?;
        }

        Ok(())
    }

    async fn update_answer_body(&self, ctx: &Ctx, body: Option<serde_json::Value>, exercise_id: i64) -> ExerciseResult<()> {
        if let Some(body) = body {
            let exercise_for_u_b = ExerciseForUpdateAnswerBody { 
                answer_body: Value::Json(Some(Box::new(body))),
            };

		    base::update::<Self, ExerciseForUpdateAnswerBody>(&ctx, &self.dbm, exercise_id, exercise_for_u_b)
			    .await
			    .map_err(Into::<DbError>::into)?;
        }

        Ok(())
    }
}

#[async_trait]
impl IExerciseCommandRepository for ExerciseCommandRepository {
    async fn get_exercise(&self, ctx: &Ctx, exercise_id: i64) -> ExerciseResult<lib_core::models::exercise::Exercise> {
        let res = base::get::<Self, ExerciseData>(ctx, &self.dbm, exercise_id)
            .await
            .map_err(Into::<DbError>::into)?
            .try_into()?;

        Ok(res)
    }

    async fn get_lesson_exercises_ordered(
        &self,
        _: &Ctx,
        lesson_id: i64
    ) ->  ExerciseResult<Vec<ExerciseForChangeOrder>> {
        let mut query = Query::select();
        query
            .from(Self::table_ref())
            .columns(ExerciseData::field_column_refs())
            .and_where(Expr::col(ExerciseIden::LessonId).eq(lesson_id))
            .order_by(ExerciseIden::ExerciseOrder, sea_query::Order::Asc);
    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, ExerciseData, _>(&sql, values);
        let lessons =
            self.dbm.dbx()
                .fetch_all(sqlx_query)
                .await
                .map_err(Into::<DbError>::into)?;

        let result = lessons.iter().map(|exercise | ExerciseForChangeOrder { 
            id: exercise.id, 
            order: exercise.exercise_order, 
        }).collect();

        Ok(result)    
    }

    async fn create(&self, ctx: &Ctx, exercise_c: ExerciseForCreateCommand) ->  ExerciseResult<i64> {
        let exercise_fi = Exercise { 
            lesson_id: exercise_c.lesson_id, 
            title: exercise_c.title, 
            description: exercise_c.description, 
            exercise_type: exercise_c.exercise_type.to_string(), 
            exercise_order: exercise_c.exercise_order, 
            answer_body: Value::Json(Some(Box::new(exercise_c.answer_body))), 
            exercise_body: Value::Json(Some(Box::new(exercise_c.exercise_body))), 
            difficult: exercise_c.difficult.to_string(), 
            time_to_complete: exercise_c.time_to_complete, 
        };

        let exercise_id = base::create::<Self, Exercise>(ctx, &self.dbm, exercise_fi)
            .await
            .map_err(Into::<DbError>::into)?;

        Ok(exercise_id)
    }

    async fn update(
        &self,
        ctx: &Ctx, 
        exercise_for_u: lib_core::models::exercise::ExerciseForUpdate, 
    ) -> ExerciseResult<()> {
		let dbm = self.dbm.new_with_txn()?;
		dbm.dbx().begin_txn().await.map_err(Into::<DbError>::into)?;

        let data = ExerciseForUpdate {
            title: exercise_for_u.title.clone(), 
            description: exercise_for_u.description.clone(), 
            exercise_type: exercise_for_u.exercise_type.and_then(|t| Some(t.to_string())), 
            difficult: exercise_for_u.difficult.clone().and_then(|d| Some(d.to_string())), 
            time_to_complete: exercise_for_u.time_to_complete,
        };

		base::update::<Self, ExerciseForUpdate>(&ctx, &self.dbm, exercise_for_u.id, data)
			.await
			.map_err(Into::<DbError>::into)?;

        self.update_answer_body(ctx, exercise_for_u.answer_body, exercise_for_u.id).await?;
        self.update_exercise_body(ctx, exercise_for_u.exercise_body, exercise_for_u.id).await?;

		dbm.dbx().commit_txn().await.map_err(Into::<DbError>::into)?;

		Ok(())
    }

    async fn update_exercise_orders(
        &self, 
        ctx: &Ctx, 
        lesson_exercises: Vec<ExerciseForChangeOrder>
    ) -> ExerciseResult<()> {
        let dbm = self.dbm.new_with_txn()?;
		dbm.dbx().begin_txn().await.map_err(Into::<DbError>::into)?;

        for exercise in &lesson_exercises {
            let exercise_for_u_order = ExerciseForUpdateOrder { 
                exercise_order: exercise.order,
            };

            base::update::<Self, ExerciseForUpdateOrder>(&ctx, &dbm, exercise.id, exercise_for_u_order)
			    .await
			    .map_err(Into::<DbError>::into)?;
        }

		dbm.dbx().commit_txn().await.map_err(Into::<DbError>::into)?;
        
        Ok(())
    }

    async fn get_exercise_user_completions(
        &self,
        ctx: &Ctx,
        user_id: i64,
        exercise_id: i64,
    ) -> ExerciseResult<Vec<ExerciseCompletion>> {
        ExerciseCompletionCommandRepository::get_exercise_user_completions(&ctx, &self.dbm, user_id, exercise_id).await
    }

    async fn create_exercise_completion(
        &self,
        ctx: &Ctx, 
        ex_comp_for_c: ExerciseCompletionForCreate
    ) -> ExerciseResult<i64> {
        ExerciseCompletionCommandRepository::create_exercise_completion(ctx, &self.dbm, ex_comp_for_c).await
    }

    async fn update_exercise_completion(
        &self, 
        ctx: &Ctx, 
        ex_comp_for_u: ExerciseCompletionForUpdate
    ) -> ExerciseResult<()> {
        ExerciseCompletionCommandRepository::update_exercise_completion(&self.dbm, ctx, ex_comp_for_u).await
    }

    async fn complete_exercise_completion(
        &self, 
        ctx: &Ctx, 
        ex_comp_for_u: ExerciseCompletionForCompleteCommand, 
    ) -> ExerciseResult<()> {
        ExerciseCompletionCommandRepository::complete_exercise(&self.dbm, ctx, ex_comp_for_u).await
    }

    async fn get_exercise_completion(&self, ctx: &Ctx, ex_comp_id: i64) -> ExerciseResult<ExerciseCompletion> {
        ExerciseCompletionCommandRepository::get(ctx, &self.dbm, ex_comp_id).await
    }

    async fn get_uncompleted_exercises(&self, ctx: &Ctx) -> ExerciseResult<Vec<ExerciseCompletion>> {
        ExerciseCompletionCommandRepository::get_uncompleted_exercises(&self.dbm, ctx).await
    }

    async fn remove_exercise_completions(&self, ctx: &Ctx, exercise_id: i64) -> ExerciseResult<()> {
        ExerciseCompletionCommandRepository::remove_exercise_completions(&self.dbm, ctx, exercise_id).await
    }

    async fn get_number_of_lesson_completed_exercises(&self, _: &Ctx, lesson_id: i64, user_id: i64) -> ExerciseResult<i64> {
    	let mut subquery = Query::select();
    	subquery.from(Self::table_ref())
        	.distinct_on([CommonIden::Id])
        	.column(CommonIden::Id)
        	.inner_join(
            	get_exercise_completion_table_ref(), 
            	Expr::col((ExerciseCompletionIden::ExerciseCompletion, ExerciseCompletionIden::ExerciseId))
            	.equals((ExerciseIden::Exercise, CommonIden::Id))
        	)
        	.and_where(Expr::col((ExerciseIden::Exercise, ExerciseIden::LessonId)).eq(lesson_id))
        	.and_where(Expr::col((ExerciseCompletionIden::ExerciseCompletion, ExerciseCompletionIden::UserId)).eq(user_id))
        	.and_where(Expr::col((ExerciseCompletionIden::ExerciseCompletion, ExerciseCompletionIden::State)).eq("Succeeded"));

    	let mut query = Query::select();
    	query.expr(Expr::col(Alias::new("subquery")).count())
        	.from_subquery(subquery, Alias::new("subquery"));

    	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    	let sqlx_query = sqlx::query_scalar_with::<_, i64, _>(&sql, values);
    	let entities = self.dbm.dbx().fetch_one_scalar(sqlx_query).await
			    .map_err(Into::<DbError>::into)?;

    	Ok(entities)
    }
}