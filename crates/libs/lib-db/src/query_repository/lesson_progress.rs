use lib_core::{ctx::Ctx, interactors::error::CoreError, models::lesson_progress::LessonProgress};
use modql::field::{Fields, HasFields};
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::FromRow;
use time::OffsetDateTime;

use crate::{base::{self, idens::{CommonIden, LessonIden, LessonProgressIden}, DbRepository}, store::{db_manager::DbManager, error::{DbError, DbResult}}};

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

#[derive(Clone)]
pub struct LessonProgressQueryRepository {
	dbm: DbManager,
}

impl LessonProgressQueryRepository {
	pub fn new(dbm: DbManager) -> Self {
		Self {
    		dbm,
		}
	}
}
impl DbRepository for LessonProgressQueryRepository {
	const TABLE: &'static str = "lesson_progress";
}

impl LessonProgressQueryRepository {
    pub async fn get_lessons_progresses(
        &self,
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
            self.dbm.dbx()
                .fetch_all(sqlx_query)
                .await
                .map_err(Into::<DbError>::into)?;

        Ok(lesson_progresses)
    }
}