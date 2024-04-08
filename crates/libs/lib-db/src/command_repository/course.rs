use async_trait::async_trait;
use derive_more::Display;
use lib_core::ctx::Ctx;
use lib_core::interfaces::course::{ICourseCommandRepository, CourseResult};
use lib_core::model::course::{Course, CourseForCreate, CourseForUpdateCommand, CourseState, UserCourse};
use modql::field::{Fields, HasFields};
use sea_query::Nullable;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use time::OffsetDateTime;
use typed_builder::TypedBuilder;
use crate::base::{self, DbRepository};
use crate::store::dbx::error::DbxError;
use crate::store::error::DbError;
use crate::store::DbManager;
use lib_utils::time::{from_unix_timestamp, Rfc3339};

use super::users_courses::{UserCourseRoleRequest, UsersCoursesForDelete, UsersCoursesCommandRepository, UsersCoursesRequest};

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

impl From<CourseState> for CourseStateRequest {
	fn from(value: CourseState) -> Self {
		match value {
			CourseState::Draft => CourseStateRequest::Draft,
			CourseState::Published => CourseStateRequest::Published,
			CourseState::Archived => CourseStateRequest::Archived,
			CourseState::None => CourseStateRequest::None,
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

pub struct CourseCommandRepository {
	dbm: DbManager,
}

impl DbRepository for CourseCommandRepository {
    const TABLE: &'static str = "course";
}

impl CourseCommandRepository {
	pub fn new(dbm: DbManager) -> Self {
		Self {
    		dbm,
		}
	}
}

#[async_trait]
impl ICourseCommandRepository for CourseCommandRepository {
	async fn get_course(&self, ctx: &Ctx, course_id: i64) -> CourseResult<Course> {
		let result = base::get::<Self, CourseRequest>(ctx, &self.dbm, course_id)
			.await
			.map_err(Into::<DbError>::into)?
			.try_into()?;

		Ok(result)
	}

	async fn create_draft(
		&self,
		ctx: &Ctx,
		course_c: CourseForCreate,
	) -> CourseResult<i64> {
		let dbm = self.dbm.new_with_txn()?;
		dbm.dbx().begin_txn().await.map_err(Into::<DbError>::into)?;

		let title = course_c.title.clone();

		let course_req_c = CourseRequest::builder()
			.title(course_c.title)
			.description(course_c.description)
			.course_type(course_c.course_type)
			.price(course_c.price)
			.color(course_c.color)
			.build();

		let course_id = base::create::<Self, CourseRequest>(ctx, &dbm, course_req_c).await.map_err(
			|model_error| {
				DbxError::resolve_unique_violation(
					model_error,
					Some(|table: &str, constraint: &str| {
						if table == "course" && constraint.contains("title") {
							Some(DbError::CourseAlreadyExists { title })
						} else {
							None // Error::UniqueViolation will be created by resolve_unique_violation
						}
					}),
				)
			}
		)?;

		let users_courses_c = UsersCoursesRequest {
    		user_id: ctx.user_id(),
    		course_id: course_id,
    		user_role: UserCourseRoleRequest::Creator,
		};

		UsersCoursesCommandRepository::create(&dbm, users_courses_c).await?;

		dbm.dbx().commit_txn().await.map_err(Into::<DbError>::into)?;

		Ok(course_id)
	}

	async fn update_course(
		&self, 
		ctx: &Ctx,
		course_for_u: CourseForUpdateCommand,
		course_id: i64,
	) -> CourseResult<()> {
		let published_date = if let Some(seconds) = course_for_u.published_date {
			Some(from_unix_timestamp(seconds).map_err(DbError::DateError)?)
		} else {
			None
		};

		let course_req_u = CourseRequest { 
			id: None,
			title: course_for_u.title, 
			description: course_for_u.description, 
			course_type: course_for_u.course_type, 
			price: course_for_u.price, 
			color: course_for_u.color, 
			img_url: course_for_u.img_url, 
			published_date,
			state: course_for_u.state.and_then(|state| Some(state.into())),
		};

		base::update::<Self, CourseRequest>(&ctx, &self.dbm, course_id, course_req_u)
			.await
			.map_err(Into::<DbError>::into)?;

		Ok(())
	}

	async fn get_user_course(&self, _: &Ctx, user_id: i64, course_id: i64) -> CourseResult<UserCourse> {
		let user_course_req = UsersCoursesCommandRepository::get(&self.dbm, user_id, course_id).await?;

		Ok(user_course_req.into())
	}

	async fn get_user_course_optional(
		&self,
		_:
		&Ctx,
		user_id: i64, 
		course_id: i64
	) -> CourseResult<Option<UserCourse>> {
		let user_course_req = UsersCoursesCommandRepository::get_optional(&self.dbm, user_id, course_id).await?;

		Ok(user_course_req.and_then(|user_course| Some(user_course.into())))
	}

	async fn create_user_course(
		&self,
		_: &Ctx,
		users_courses_c: UserCourse,
	) -> CourseResult<()> {
		let user_course_req = UsersCoursesRequest { 
			user_id: users_courses_c.user_id, 
			course_id: users_courses_c.course_id, 
			user_role: users_courses_c.user_role.into(),
		};

		UsersCoursesCommandRepository::create(&self.dbm, user_course_req).await?;

		Ok(())
	}

	async fn delete_user_course(
		&self,
		_: &Ctx,
		user_id: i64,
		course_id: i64,
	) -> CourseResult<()> {
		let user_course_req = UsersCoursesForDelete { 
			user_id,
			course_id,
		};

		UsersCoursesCommandRepository::delete(&self.dbm, user_course_req).await?;

		Ok(())
	}
}
