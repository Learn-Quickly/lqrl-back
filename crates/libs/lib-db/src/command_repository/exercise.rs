use async_trait::async_trait;
use lib_core::{ctx::Ctx, interfaces::exercise::{ExerciseResult, IExerciseCommandRepository}, models::exercise::{ExerciseForChangeOreder, ExerciseForCreateCommand}};
use modql::field::{Fields, HasFields};
use sea_query::{Expr, PostgresQueryBuilder, Query, Value};
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, prelude::FromRow};

use crate::{base::{self, idens::ExerciseIden, DbRepository}, store::{db_manager::DbManager, error::DbError}};

#[derive(Fields)]
struct Exercise {
	pub lesson_id: i64,
    pub title: String,
    pub description: String,
    pub exercise_type: String,
    pub exercise_order: i32,
    pub body: Value,
    pub difficult: String,
    pub time_to_complete: Option<i64>,
}

#[derive(Fields, FromRow)]
struct ExerciseData {
    pub id: i64,
	pub lesson_id: i64,
    pub title: String,
    pub description: String,
    pub exercise_type: String,
    pub exercise_order: i32,
    pub body: serde_json::Value,
    pub difficult: String,
    pub time_to_complete: Option<i64>,
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
}

#[async_trait]
impl IExerciseCommandRepository for ExerciseCommandRepository {
    async fn get_lesson_exercises_ordered(
        &self,
        _: &Ctx,
        lesson_id: i64
    ) ->  ExerciseResult<Vec<ExerciseForChangeOreder>> {
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

        let result = lessons.iter().map(|exercise | ExerciseForChangeOreder { 
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
            body: Value::Json(Some(Box::new(exercise_c.body))), 
            difficult: exercise_c.difficult.to_string(), 
            time_to_complete: exercise_c.time_to_complete, 
        };

        let exercise_id = base::create::<Self, Exercise>(ctx, &self.dbm, exercise_fi)
            .await
            .map_err(Into::<DbError>::into)?;

        Ok(exercise_id)
    }
}