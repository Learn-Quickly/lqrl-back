use lib_core::interactors::error::{CoreError, CourseError};
use lib_db::query_repository::course::CourseQuery;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use utoipa::{IntoParams, ToSchema};


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

#[derive(ToSchema, Serialize)]
pub struct CoursesPayload {
	pub courses: Vec<CoursePayload>,
	pub count: i64,
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
}

impl TryFrom<String> for CourseStatePayload {
	type Error = CoreError;
	
	fn try_from(value: String) -> Result<Self, Self::Error> {
		match value.as_str() {
			"Draft" => Ok(Self::Draft),
			"Published" => Ok(Self::Published),
			"Archived" => Ok(Self::Archived),
			state => Err(CourseError::CourseStateDoesNotExist { state: state.to_string() }.into())
		}
	}
}

impl TryFrom<CourseQuery> for CoursePayload {
	type Error = CoreError;
	
	fn try_from(value: CourseQuery) -> Result<Self, Self::Error> {
		let published_date = value.published_date.and_then(|date| Some(date.unix_timestamp()));

		let result = Self {
    		id: value.id,
    		title: value.title,
    		description: value.description,
    		course_type: value.course_type,
    		price: value.price,
    		color: value.color,
    		published_date,
    		img_url: value.img_url,
    		state: value.state.try_into()?,
		};

		Ok(result)
	}
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct CourseFilterPayload {
	#[param(example = "[{\"price\": {\"$gte\": 40, \"$lte\": 70}}, {\"color\": {\"$eq\": \"indigo\"}}]")]
	pub filters: Option<String>,
	#[param(example = "{\"limit\": 5, \"offset\": 2, \"order_bys\": \"!price\"}")]
	pub list_options: Option<String>,
}
