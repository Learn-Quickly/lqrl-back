use lib_core::{ctx::Ctx, interactors::error::CoreError, models::lesson_progress::LessonProgress};
use lib_utils::time::now_utc;
use modql::field::{Fields, HasFields};
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::FromRow;
use time::OffsetDateTime;

use crate::{base::{self, idens::{CommonIden, LessonIden, LessonProgressIden}, prep_fields_for_create, DbRepository}, store::{db_manager::DbManager, dbx::error::DbxError, error::{DbError, DbResult}}};

#[derive(Clone, Fields, FromRow, Debug)]
pub struct LessonProgressData {
    pub user_id: i64,
    pub lesson_id: i64,

    pub date_started: OffsetDateTime,
    pub date_complete: Option<OffsetDateTime>,  

    pub state: String,
}

impl TryFrom<&LessonProgressData> for LessonProgress {
    type Error = CoreError;

    fn try_from(value: &LessonProgressData) -> Result<Self, Self::Error> {
        Ok(Self {
            user_id: value.user_id,
            lesson_id: value.lesson_id,
            date_started: value.date_started.unix_timestamp(),
            date_complete: value.date_complete.and_then(|date| Some(date.unix_timestamp())),
            state: value.state.clone().try_into()?,
        })
    }
}

#[derive(Fields)]
struct LessonProgreesForInsert {
    user_id: i64,
    lesson_id: i64,
    date_started: OffsetDateTime,
}

#[derive(Clone)]
pub struct LessonProgressCommandRepository;

impl DbRepository for LessonProgressCommandRepository {
	const TABLE: &'static str = "lesson_progress";
}

impl LessonProgressCommandRepository {
    pub async fn create(
        ctx: &Ctx,
        dbm: &DbManager,
        user_id: i64,
        lesson_id: i64,
    ) -> DbResult<()> {
        let date_started = now_utc();
        let lesson_progress_f_i = LessonProgreesForInsert {
            user_id,
            lesson_id,
            date_started,
        };
        let user_id = ctx.user_id();

        let mut fields = lesson_progress_f_i.not_none_fields();
        prep_fields_for_create::<Self>(&mut fields, user_id);
    
        let (columns, sea_values) = fields.for_sea_insert();
        let mut query = Query::insert();
        query
            .into_table(Self::table_ref())
            .columns(columns)
            .values(sea_values)
			.map_err(DbxError::SeaQuery)
            .map_err(Into::<DbError>::into)?
            .returning(Query::returning().columns([LessonProgressIden::LessonProgressId]));
    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
        let (_,) = dbm.dbx().fetch_one(sqlx_query).await?;

        Ok(())
    }

    pub async fn get_lessons_progresses(
        dbm: &DbManager,
        _: &Ctx, 
        course_id: i64,
        user_id: i64,
    ) -> DbResult<Vec<LessonProgressData>> {
        let mut query = Query::select();
        query
            .columns(LessonProgressData::field_column_refs())
            .from(Self::table_ref())
            .join(
                sea_query::JoinType::InnerJoin,
                base::table_ref::get_lesson_table_ref(),
                Expr::col((LessonProgressIden::LessonProgress, LessonProgressIden::LessonId))
                    .equals((LessonIden::Lesson, CommonIden::Id)),
            )
            .and_where(Expr::col(LessonProgressIden::UserId).eq(user_id))
            .and_where(Expr::col(LessonIden::CourseId).eq(course_id));
    
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, LessonProgressData, _>(&sql, values);
    
        let lesson_progresses: Vec<LessonProgressData> = 
            dbm.dbx()
                .fetch_all(sqlx_query)
                .await
                .map_err(Into::<DbError>::into)?;

        Ok(lesson_progresses)
    }
}