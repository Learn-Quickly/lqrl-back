use lib_db::query_repository::course::{CourseQuery, CourseStateQuery};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;
use utoipa::ToSchema;


#[derive(Debug, Deserialize, ToSchema)]
pub struct CourseCreateDraftPayload {
	pub title: String,
	pub description: String,
	pub course_type: String,
	pub price: f64,
	pub color: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreatedCourseDraft {
	pub course_id: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct CourseUpdatePayload {
	pub id: i64,
	pub title: Option<String>,
	pub description: Option<String>,
	pub course_type: Option<String>,
	pub price: Option<f64>,
	pub color: Option<String>,
}

#[derive(ToSchema, Deserialize)]
pub struct CourseId {
	pub course_id: i64,
}

#[serde_as]
#[derive(Serialize, ToSchema)]
pub struct CoursePayload {
	pub id: i64,
	pub title: String,
	pub description: String,
	pub course_type: String,
	pub price: f64,
	pub color: String,
	pub published_date: Option<i64>,
	pub img_url: Option<String>,
	pub state: CourseStatePayload,
}

#[serde_as]
#[derive(Serialize, ToSchema)]
pub enum CourseStatePayload {
    Draft,
    Published,
	Archived,
    None,
}

impl From<CourseStateQuery> for CourseStatePayload {
	fn from(value: CourseStateQuery) -> Self {
		match value {
			CourseStateQuery::Draft => Self::Draft,
			CourseStateQuery::Published => Self::Published,
			CourseStateQuery::Archived => Self::Archived,
			CourseStateQuery::None => Self::None,
		}
	}
}

impl From<CourseQuery> for CoursePayload {
	fn from(value: CourseQuery) -> Self {
		let published_date = value.published_date.and_then(|date| Some(date.unix_timestamp()));

		Self {
    		id: value.id,
    		title: value.title,
    		description: value.description,
    		course_type: value.course_type,
    		price: value.price,
    		color: value.color,
    		published_date,
    		img_url: value.img_url,
    		state: value.state.into(),
		}
	}
}

#[derive(Deserialize, ToSchema)]
pub struct CourseFilterPayload {
	#[schema(example = r#"{"price": {"$gte": 1000}}""#)]
	pub filters: Option<Value>,
	#[schema(example = r#"{"limit": 2, "offset": 0, "order_bys": "!title"}""#)]
	pub list_options: Option<Value>,
}
