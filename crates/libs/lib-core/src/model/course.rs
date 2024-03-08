use derive_more::Display;
use modql::field::{Fields, HasFields};
use modql::filter::{FilterNodes, ListOptions, OpValsFloat64, OpValsInt64, OpValsString, OpValsValue};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use time::OffsetDateTime;
use crate::model::modql_utils::time_to_sea_value;
use crate::model::{Error, Result};
use lib_utils::time::Rfc3339;

use crate::ctx::Ctx;

use super::users_courses::{UserCourseRole, UsersCoursesBmc, UsersCoursesForCreate};
use super::{base::{self, DbBmc}, ModelManager};

#[serde_as]
#[derive(Clone, Fields, FromRow, Debug, Serialize)]
pub struct Course {
	pub id: i64,
	pub title: String,
	pub description: String,
	pub course_type: String,
	pub price: f64,
	pub color: String,
	#[serde_as(as = "Option<Rfc3339>")]
	pub published_date: Option<OffsetDateTime>,
	pub img_url: Option<String>,
	#[field(cast_as = "user_course_roles")]
	pub state: CourseState,
}

#[derive(Debug, Clone, Display, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "course_state")]
pub enum CourseState {
    Draft,
    Published,
	Archived,
}

impl From<CourseState> for sea_query::Value {
	fn from(value: CourseState) -> Self {
		value.to_string().into()
	}
}

#[derive(Deserialize, Fields)] 
pub struct CourseForCreate {
	pub title: String,
	pub description: String,
	pub course_type: String,
	pub price: f64,
	pub color: String,
}

#[derive(Fields)]
pub struct CourseForPublish {
	state: CourseState,
}

#[derive(Fields)]
pub struct CourseForArchive{
	state: CourseState,
}

/// Marker trait
pub trait CourseBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl CourseBy for Course {}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct CourseFilter {
	pub id: Option<OpValsInt64>,
	pub title: Option<OpValsString>,
	pub description: Option<OpValsString>,
	pub course_type: Option<OpValsString>,
	pub price: Option<OpValsFloat64>,
	pub color: Option<OpValsString>,
	pub state: Option<OpValsString>,

	#[modql(to_sea_value_fn = "time_to_sea_value")]
	pub publish_date: Option<OpValsInt64>,

	pub cid: Option<OpValsInt64>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	pub ctime: Option<OpValsValue>,
	pub mid: Option<OpValsInt64>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	pub mtime: Option<OpValsValue>,
}

pub struct CourseBmc;

impl DbBmc for CourseBmc {
    const TABLE: &'static str = "course";
}

impl CourseBmc {
	pub async fn create_draft(
		ctx: &Ctx,
		mm: &ModelManager,
		course_c: CourseForCreate,
	) -> Result<i64> {
		let mm = mm.new_with_txn()?;

		let title = course_c.title.clone();
		mm.dbx().begin_txn().await?;

		let course_id = base::create::<Self, _>(ctx, &mm, course_c).await.map_err(
			|model_error| {
				Error::resolve_unique_violation(
					model_error,
					Some(|table: &str, constraint: &str| {
						if table == "course" && constraint.contains("title") {
							Some(Error::CourseAlreadyExists { title })
						} else {
							None // Error::UniqueViolation will be created by resolve_unique_violation
						}
					}),
				)
			}
		)?;

		let users_courses_c = UsersCoursesForCreate {
    		user_id: ctx.user_id(),
    		course_id: course_id,
    		user_role: UserCourseRole::Creator,
		};

		UsersCoursesBmc::create(&mm, users_courses_c).await?;

		mm.dbx().commit_txn().await?;

		Ok(course_id)
	}

	pub async fn publish_course(
		ctx: &Ctx,
		mm: &ModelManager,
		course_id: i64,
	) -> Result<()> {
		let course_for_publish = CourseForPublish {
    		state: CourseState::Published,
		};

		mm.dbx.begin_txn().await?;

		base::update::<Self, _>(&ctx, &mm, course_id, course_for_publish).await?;

		mm.dbx().commit_txn().await?;

		Ok(())
	}

	pub async fn archive_course(
		ctx: &Ctx,
		mm: &ModelManager,
		course_id: i64,
	) -> Result<()> {
		let course_for_publish = CourseForArchive {
    		state: CourseState::Archived,
		};

		mm.dbx.begin_txn().await?;

		base::update::<Self, _>(&ctx, &mm, course_id, course_for_publish).await?;

		mm.dbx().commit_txn().await?;

		Ok(())
	}
	
	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filter: Option<Vec<CourseFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<Course>> {
		base::list::<Self, _, _>(ctx, mm, filter, list_options).await
	}

	pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
	where
		E: CourseBy,
	{
		base::get::<Self, _>(ctx, mm, id).await
	}
}