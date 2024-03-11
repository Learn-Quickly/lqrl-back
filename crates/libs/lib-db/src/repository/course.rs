use async_trait::async_trait;
use derive_more::Display;
use lib_core::ctx::Ctx;
use lib_core::interfaces::course::{ICourseRepository, CourseResult};
use lib_core::model::course::{Course, CourseForCreate, CourseForUpdate, CourseState, UserCourse};
use modql::field::{Fields, HasFields};
use modql::filter::{FilterNodes, ListOptions, OpValsFloat64, OpValsInt64, OpValsString, OpValsValue};
use sea_query::Nullable;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use time::OffsetDateTime;
use typed_builder::TypedBuilder;
use crate::repository::modql_utils::time_to_sea_value;
use lib_utils::time::Rfc3339;

use super::base::{self, DbRepository};
use super::error::DbError;
use super::users_courses::{UserCourseRoleRequest, UsersCoursesController, UsersCoursesForCreate};
use super::DbManager;

#[serde_as]
#[derive(Clone, Fields, FromRow, Debug, Serialize, TypedBuilder)]
pub struct CourseRequest {
	#[builder(default, setter(strip_option))]
	pub id: Option<i64>,
	#[builder(default, setter(strip_option))]
	pub title: Option<String>,
	#[builder(default, setter(strip_option))]
	pub description: Option<String>,
	#[builder(default, setter(strip_option))]
	pub course_type: Option<String>,
	#[builder(default, setter(strip_option))]
	pub price: Option<f64>,
	#[builder(default, setter(strip_option))]
	pub color: Option<String>,
	#[builder(default, setter(strip_option))]
	#[serde_as(as = "Option<Rfc3339>")]
	pub published_date: Option<OffsetDateTime>,
	#[builder(default, setter(strip_option))]
	pub img_url: Option<String>,
	#[builder(default, setter(strip_option))]
	#[field(cast_as = "course_state")]
	pub state: Option<CourseStateRequest>,
}

impl TryFrom<CourseRequest> for Course {
	type Error = DbError;

	fn try_from(value: CourseRequest) -> Result<Self, Self::Error> {
		let entity = "Course".to_string();
		let id = DbError::handle_option_field(value.id, &entity, "id".to_string())?;
		let title = DbError::handle_option_field(value.title, &entity, "title".to_string())?;
		let description = DbError::handle_option_field(value.description, &entity, "description".to_string())?;
		let course_type = DbError::handle_option_field(value.course_type, &entity, "course_type".to_string())?;
		let price = DbError::handle_option_field(value.price, &entity, "price".to_string())?;
		let color = DbError::handle_option_field(value.color, &entity, "color".to_string())?;
		let state = DbError::handle_option_field(value.state, &entity, "state".to_string())?;
		let published_date = value.published_date.and_then(|date| Some(date.unix_timestamp()));

		Ok(Course {
    		id,
    		title,
		    description,
 		    course_type,
    		price,
    		color,
    		published_date,
    		img_url: value.img_url,
    		state: state.into(),
		})
	}
}

#[derive(Debug, Clone, Display, sqlx::Type, Deserialize, Serialize, PartialEq, Eq)]
#[sqlx(type_name = "course_state")]
pub enum CourseStateRequest {
    Draft,
    Published,
	Archived,
	None,
}

impl From<CourseStateRequest> for sea_query::Value {
	fn from(value: CourseStateRequest) -> Self {
		value.to_string().into()
	}
}

impl From<CourseStateRequest> for CourseState {
	fn from(value: CourseStateRequest) -> Self {
		match value {
			CourseStateRequest::Draft => Self::Draft,
			CourseStateRequest::Published => Self::Published,
			CourseStateRequest::Archived => Self::Archived,
			CourseStateRequest::None => Self::None,
		}
	}
}

impl Nullable for CourseStateRequest {
	fn null() -> sea_query::Value {
		CourseStateRequest::None.into()
	}
}

/// Marker trait
pub trait CourseBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl CourseBy for CourseRequest {}

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

pub struct CourseRepository {
	dbm: DbManager,
}

impl DbRepository for CourseRepository {
    const TABLE: &'static str = "course";
}

impl CourseRepository {
	pub fn new(dbm: DbManager) -> Self {
		Self {
    		dbm,
		}
	}

	pub async fn list(
		ctx: &Ctx,
		dbm: &DbManager,
		filter: Option<Vec<CourseFilter>>,
		list_options: Option<ListOptions>,
	) -> CourseResult<Vec<CourseRequest>> {
		let result = base::list::<Self, _, _>(ctx, dbm, filter, list_options)
			.await
			.map_err(|db_err| Box::new(db_err))?;

		Ok(result)
	}

	async fn get(ctx: &Ctx, dbm: &DbManager, id: i64) -> CourseResult<CourseRequest>
	{
		let result = base::get::<Self, _>(ctx, dbm, id)
			.await
			.map_err(|db_err| Box::new(db_err))?;

		Ok(result)
	}
}

#[async_trait]
impl ICourseRepository for CourseRepository {
	async fn get_course(&self, ctx: &Ctx, course_id: i64) -> CourseResult<Course> {
		let course_request = Self::get(&ctx, &self.dbm, course_id).await?;

		Ok(course_request.try_into()?)
	}

	async fn create_draft(
		&self,
		ctx: &Ctx,
		course_c: CourseForCreate,
	) -> CourseResult<i64> {
		let dbm = self.dbm.new_with_txn()?;

		let title = course_c.title.clone();
		dbm.dbx().begin_txn().await?;

		let course_req_c = CourseRequest::builder()
			.title(course_c.title)
			.description(course_c.description)
			.course_type(course_c.course_type)
			.price(course_c.price)
			.color(course_c.color)
			.build();

		let course_id = base::create::<Self, CourseRequest>(ctx, &dbm, course_req_c).await.map_err(
			|model_error| {
				Box::new(DbError::resolve_unique_violation(
					model_error,
					Some(|table: &str, constraint: &str| {
						if table == "course" && constraint.contains("title") {
							Some(DbError::CourseAlreadyExists { title })
						} else {
							None // Error::UniqueViolation will be created by resolve_unique_violation
						}
					}),
				))
			}
		)?;

		let users_courses_c = UsersCoursesForCreate {
    		user_id: ctx.user_id(),
    		course_id: course_id,
    		user_role: UserCourseRoleRequest::Creator,
		};

		UsersCoursesController::create(&dbm, users_courses_c).await?;

		dbm.dbx().commit_txn().await?;

		Ok(course_id)
	}

	async fn update_course(
		&self, 
		ctx: &Ctx,
		course_for_u: CourseForUpdate,
		course_id: i64,
	) -> CourseResult<()> {
		let course_req_u = CourseRequest { 
			title: course_for_u.title, 
			description: course_for_u.description, 
			course_type: course_for_u.course_type, 
			price: course_for_u.price, 
			color: course_for_u.color, 
			img_url: course_for_u.img_url, 
			id: None,
			published_date: None,
			state: None,
		};

		base::update::<Self, CourseRequest>(&ctx, &self.dbm, course_id, course_req_u)
			.await
			.map_err(|db_err| Box::new(db_err))?;

		Ok(())
	}

	async fn publish_course(
		&self,
		ctx: &Ctx,
		course_id: i64,
	) -> CourseResult<()> {
		let now_utc = lib_utils::time::now_utc();
		let course_req_publish = CourseRequest::builder()
			.state(CourseStateRequest::Published)
			.published_date(now_utc)
			.build();

		base::update::<Self, CourseRequest>(&ctx, &self.dbm, course_id, course_req_publish)
			.await
			.map_err(|db_err| Box::new(db_err))?;

		Ok(())
	}

	async fn archive_course(
		&self,
		ctx: &Ctx,
		course_id: i64,
	) -> CourseResult<()> {
		let course_req_archive = CourseRequest::builder()
			.state(CourseStateRequest::Archived)
			.build();

		base::update::<Self, CourseRequest>(&ctx, &self.dbm, course_id, course_req_archive)
			.await
			.map_err(|db_err| Box::new(db_err))?;

		Ok(())
	}

	async fn create_user_course(
		&self,
		_: &Ctx,
		users_courses_c: UserCourse,
	) -> CourseResult<()> {
		let user_course_req = UsersCoursesForCreate { 
			user_id: users_courses_c.user_id, 
			course_id: users_courses_c.course_id, 
			user_role: users_courses_c.user_role.into(),
		};

		UsersCoursesController::create(&self.dbm, user_course_req)
			.await
			.map_err(|db_err| Box::new(db_err))?;

		Ok(())
	}
}
