use std::collections::HashMap;

use lib_core::{ctx::Ctx, models::exercise::ExerciseDifficulty};
use modql::field::{Fields, HasFields};
use sea_query::{Alias, Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde_json::Value;
use sqlx::FromRow;

use crate::{base::{self, idens::{CommonIden, ExerciseCompletionIden, ExerciseIden, LessonIden, UserIden}, table_ref::{get_exercise_completion_table_ref, get_lesson_table_ref, get_user_table_ref}, DbRepository}, store::{db_manager::DbManager, error::{DbError, DbResult}}};

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

#[derive(Clone, Fields, FromRow, Debug)]
pub struct Id {
	pub id: i64,
}

#[derive(Clone, Fields, FromRow, Debug)]
pub struct ExercisePoitQuery {
	pub id: i64,
    pub difficult: String,
}

#[derive(Clone, Fields, FromRow, Debug)]
pub struct UsersCompltedExercises {
    pub exercise_id: i64,
    pub user_id: i64,
    pub username: String,
    pub points_scored: f32,
}

pub struct CoursePointStatistics {
    pub max_points: i64,
    pub users_points: Vec<UserPoints>,
}

pub struct UserPoints {
    pub user_id: i64,
    pub username: String,
    pub points: f32,
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

    pub async fn get_number_of_lesson_completed_exercises(&self, _: &Ctx, lesson_id: i64, user_id: i64) -> DbResult<i64> {
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

    pub async fn get_course_point_statistics(
        &self, 
        ctx: &Ctx, 
        course_id: i64,
    ) -> DbResult<CoursePointStatistics> {
        let exercises = self.get_course_exercises(ctx, course_id).await?;
        let ex_ids: Vec<i64> = exercises.iter().map(|exercise| exercise.id).collect();

    	let mut query = Query::select();
    	query.from(get_exercise_completion_table_ref())
        	.columns(UsersCompltedExercises::field_column_refs())
        	.inner_join(
            	get_user_table_ref(), 
            	Expr::col((ExerciseCompletionIden::ExerciseCompletion, ExerciseCompletionIden::UserId))
            	.equals((UserIden::User, CommonIden::Id))
        	)
            .and_where(Expr::col((ExerciseCompletionIden::ExerciseCompletion, ExerciseCompletionIden::ExerciseId)).is_in(ex_ids))
        	.and_where(Expr::col((ExerciseCompletionIden::ExerciseCompletion, ExerciseCompletionIden::State)).eq("Succeeded"))
            .distinct();

        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, UsersCompltedExercises, _>(&sql, values);

        let users_completed_exs =
            self.dbm.dbx()
                .fetch_all(sqlx_query)
                .await
                .map_err(Into::<DbError>::into)?;

        let users_points = self.calculate_user_points(&users_completed_exs);

        let result = CoursePointStatistics {
            max_points: self.calculate_course_max_points(&exercises)?,
            users_points,
        };

        Ok(result)
    }

    async fn get_course_exercises(
        &self, 
        _: &Ctx, 
        course_id: i64
    ) -> DbResult<Vec<ExercisePoitQuery>> {
        let mut query = Query::select();
        query
            .from(get_lesson_table_ref())
            .column(CommonIden::Id)
            .and_where(Expr::col(LessonIden::CourseId).eq(course_id));
    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, Id, _>(&sql, values);
        let lessons =
            self.dbm.dbx()
                .fetch_all(sqlx_query)
                .await
                .map_err(Into::<DbError>::into)?;

        let lesson_ids: Vec<i64> = lessons.iter().map(|lesson_id| lesson_id.id).collect();

        let mut query = Query::select();
        query
            .from(Self::table_ref())
            .columns(ExercisePoitQuery::field_column_refs())
            .and_where(Expr::col(ExerciseIden::LessonId).is_in(lesson_ids));
    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, ExercisePoitQuery, _>(&sql, values);
        let exercises =
            self.dbm.dbx()
                .fetch_all(sqlx_query)
                .await
                .map_err(Into::<DbError>::into)?;

        Ok(exercises)
    }

    fn calculate_course_max_points(&self, exercises: &Vec<ExercisePoitQuery>) -> DbResult<i64> {
        let mut max_points = 0;
        for exercise in exercises {
            let exercise_max_points = f32::from(
                ExerciseDifficulty::try_from(exercise.difficult.clone())
                .map_err(|_| DbError::ParseFieldError)?
            ) * 100.0;
            max_points += exercise_max_points as i64;
        }

        Ok(max_points)
    }

    fn calculate_user_points(&self, users_completed_exs: &Vec<UsersCompltedExercises>) -> Vec<UserPoints> {
        let mut points_map: HashMap<i64, (String, f32)> = HashMap::new();

        for exercise in users_completed_exs {
            let entry = points_map.entry(exercise.user_id).or_insert((exercise.username.clone(), 0.0));
            entry.1 += exercise.points_scored;
        }
    
        points_map.into_iter().map(|(user_id, (username, points))| {
            UserPoints {
                user_id,
                username,
                points,
            }
        }).collect()
    }
}