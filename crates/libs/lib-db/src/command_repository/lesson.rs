use async_trait::async_trait;
use lib_core::{
    ctx::Ctx, 
    interfaces::lesson::{ILessonCommandRepository, LessonResult}, 
    models::{lesson::{
        Lesson, LessonForChangeOreder, LessonForCreateCommand, LessonForUpdate
    }, lesson_progress::{LessonProgress, LessonProgressState}}
};
use modql::field::{Fields, HasFields};
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, FromRow};

use crate::{base::{self, idens::{CommonIden, LessonIden, LessonProgressIden}, prep_fields_for_update, table_ref::get_lesson_progress_table_ref, DbRepository}, store::{db_manager::DbManager, error::DbError}};

use super::lesson_progress::LessonProgressCommandRepository;

#[derive(Clone, Fields, FromRow, Debug)]
struct LessonData {
	pub id: i64,
	pub course_id: i64,
    pub title: String,
    pub lesson_order: i32,
}

#[derive(Fields)]
struct LessonForInsert {
	pub course_id: i64,
    pub title: String,
    pub description: String,
    pub lesson_order: i32,
}

#[derive(Fields)]
struct LessonForUpdateOrder {
    pub lesson_order: i32,
}

#[derive(Fields)]
struct LessonForUpdateData {
    pub title: String,
    pub description: String,
}

#[derive(Fields)]
pub struct LessonForUpdateState {
    pub state: String,
}

pub trait LessonBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl LessonBy for LessonData {}

#[derive(Clone)]
pub struct LessonCommandRepository {
    dbm: DbManager,
}

impl DbRepository for LessonCommandRepository {
	const TABLE: &'static str = "lesson";
}

impl LessonCommandRepository {
	pub fn new(dbm: DbManager) -> Self {
		Self {
			dbm,
		}
	}
}

#[async_trait]
impl ILessonCommandRepository for LessonCommandRepository {
    async fn get_lesson(&self, ctx: &Ctx, lesson_id: i64) -> LessonResult<Lesson> {
        let lesson = base::get::<Self, LessonData>(ctx, &self.dbm, lesson_id)
            .await
            .map_err(Into::<DbError>::into)?;

        let result = Lesson { 
            id: lesson.id, 
            course_id: lesson.course_id, 
            title: lesson.title, 
            lesson_order: lesson.lesson_order, 
        };

        Ok(result)
    }

    async fn get_course_lessons_ordered(
        &self, 
        _: &Ctx, 
        course_id: i64
    ) -> LessonResult<Vec<LessonForChangeOreder>> {
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

        let result = lessons.iter().map(|lesson| LessonForChangeOreder { 
            id: lesson.id, 
            order: lesson.lesson_order, 
        }).collect();

        Ok(result)
    }

    async fn create_lesson_progress(
        &self,
        ctx: &Ctx, 
        lesson_id: i64, 
        user_id: i64
    ) -> LessonResult<()> {
        LessonProgressCommandRepository::create(ctx, &self.dbm, user_id, lesson_id).await?;
        Ok(())
    }

    async fn get_lessons_progresses(
        &self,
        ctx: &Ctx,
        course_id: i64, 
        user_id: i64
    ) -> LessonResult<Vec<LessonProgress>> {
        let lesson_progresses_data = LessonProgressCommandRepository::get_lessons_progresses(&self.dbm, &ctx, course_id, user_id)
            .await?;

        let mut result = Vec::new();
        for lesson_progress_date in &lesson_progresses_data {
            result.push(lesson_progress_date.try_into()?)
        }

        Ok(result)
    }
    
    async fn create_lesson(
        &self,
        ctx: &Ctx,
        lesson_for_c: LessonForCreateCommand,
    ) -> LessonResult<i64> {
        let lesson_fi = LessonForInsert {
            course_id: lesson_for_c.course_id, 
            title: lesson_for_c.title, 
            description: lesson_for_c.description,
            lesson_order:  lesson_for_c.order,
        };

        let lesson_id = base::create::<Self, LessonForInsert>(ctx, &self.dbm, lesson_fi)
            .await
            .map_err(Into::<DbError>::into)?;

        Ok(lesson_id)
    }

    async fn delete_lesson(&self, ctx: &Ctx, lesson_id: i64) -> LessonResult<()> {
        base::delete::<Self>(ctx, &self.dbm, lesson_id)
            .await
            .map_err(Into::<DbError>::into)?;

        Ok(())
    }

    async fn update_lesson(
        &self,
        ctx: &Ctx, 
        lesson_for_u: LessonForUpdate
    ) -> LessonResult<()> {
        let data = LessonForUpdateData {title:lesson_for_u.title, description: lesson_for_u.description };

		base::update::<Self, LessonForUpdateData>(&ctx, &self.dbm, lesson_for_u.id, data)
			.await
			.map_err(Into::<DbError>::into)?;

		Ok(())
    }

    async fn update_lesson_orders(
        &self, 
        ctx: &Ctx, 
        lessons_for_c_order: Vec<LessonForChangeOreder>
    ) -> LessonResult<()> {
		let dbm = self.dbm.new_with_txn()?;
		dbm.dbx().begin_txn().await.map_err(Into::<DbError>::into)?;

        for lesson in &lessons_for_c_order {
            let lesson_for_u_order = LessonForUpdateOrder { 
                lesson_order: lesson.order,
            };

            base::update::<Self, LessonForUpdateOrder>(&ctx, &dbm, lesson.id, lesson_for_u_order)
			    .await
			    .map_err(Into::<DbError>::into)?;
        }

		dbm.dbx().commit_txn().await.map_err(Into::<DbError>::into)?;

        Ok(())
    }

    async fn change_lesson_progress_states_for_update_exercise(
        &self, 
        ctx: &Ctx, 
        course_id: i64, 
        order: i32
    ) -> LessonResult<()> {
		let dbm = self.dbm.new_with_txn()?;
		dbm.dbx().begin_txn().await.map_err(Into::<DbError>::into)?;

        let pause_state = LessonForUpdateState {
            state: LessonProgressState::Pause.to_string(),
        };
        let mut fields = pause_state.not_none_fields();
        prep_fields_for_update::<Self>(&mut fields, ctx.user_id());
    
        let lesson_progress_table_ref = get_lesson_progress_table_ref();
        let fields = fields.for_sea_update();
        let mut query = Query::update();
        query
        .table(lesson_progress_table_ref.clone()) 
        .values(fields)
        .and_where(
            Expr::col(LessonProgressIden::LessonId)
                .in_subquery(Query::select()
                    .column(CommonIden::Id)
                    .from(LessonIden::Lesson)
                    .and_where(Expr::col(LessonIden::LessonOrder).gt(order))
                    .and_where(Expr::col(LessonIden::CourseId).eq(course_id))
                    .to_owned())
        );

    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_with(&sql, values);
        dbm.dbx().execute(sqlx_query).await.map_err(Into::<DbError>::into)?;

        let in_progress_state = LessonForUpdateState {
            state: LessonProgressState::InProgress.to_string(),
        };
        let mut fields = in_progress_state.not_none_fields();
        prep_fields_for_update::<Self>(&mut fields, ctx.user_id());
    
        let fields = fields.for_sea_update();
        let mut query = Query::update();
        query
        .table(lesson_progress_table_ref) 
        .values(fields)
        .and_where(
            Expr::col(LessonProgressIden::LessonId)
                .in_subquery(Query::select()
                    .column(CommonIden::Id)
                    .from(LessonIden::Lesson)
                    .and_where(Expr::col(LessonIden::LessonOrder).eq(order))
                    .and_where(Expr::col(LessonIden::CourseId).eq(course_id))
                    .to_owned())
        );
    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_with(&sql, values);
        dbm.dbx().execute(sqlx_query).await.map_err(Into::<DbError>::into)?;

		dbm.dbx().commit_txn().await.map_err(Into::<DbError>::into)?;

        Ok(())
    }

    async fn update_lesson_progress_state(
        &self,
        _: &Ctx, 
        lesson_for_u: LessonProgressState, 
        lesson_id: i64, 
        user_id: i64,
    ) -> LessonResult<()> {
        let lesson_for_u_s = LessonForUpdateState {
            state: lesson_for_u.to_string(),
        };

        let lesson_progress_table_ref = get_lesson_progress_table_ref();

        let fields = lesson_for_u_s.not_none_fields();
    
        let fields = fields.for_sea_update();
        let mut query = Query::update();
        query
            .table(lesson_progress_table_ref)
            .values(fields)
            .and_where(Expr::col(LessonProgressIden::UserId).eq(user_id))
            .and_where(Expr::col(LessonProgressIden::LessonId).eq(lesson_id));
    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_with(&sql, values);
        self.dbm.dbx().execute(sqlx_query).await.map_err(Into::<DbError>::into)?;
    
        Ok(())
    }
}