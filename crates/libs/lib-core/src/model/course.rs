use typed_builder::TypedBuilder;

#[derive(Clone)]
pub struct Course {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub course_type: String,
    pub price: f64,
    pub color: String,
    pub published_date: Option<i64>,
    pub img_url: Option<String>,
    pub state: CourseState,
}

#[derive(Clone, PartialEq, Eq)]
pub enum CourseState {
    Draft,
    Published,
	Archived,
    None,
}

pub struct CourseForCreate {
	pub title: String,
	pub description: String,
	pub course_type: String,
	pub price: f64,
	pub color: String,
}

#[derive(TypedBuilder)]
pub struct CourseForUpdateCommand {
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
	pub img_url: Option<String>,
	#[builder(default, setter(strip_option))]
	pub published_date: Option<i64>,
	#[builder(default, setter(strip_option))]
	pub state: Option<CourseState>,
}

pub struct CourseForUpdate {
	pub title: Option<String>,
	pub description: Option<String>,
	pub course_type: Option<String>,
	pub price: Option<f64>,
	pub color: Option<String>,
	pub img_url: Option<String>,
}

pub struct UserCourse {
    pub user_id: i64,
    pub course_id: i64,
    pub user_role: UserCourseRole,
}

#[derive(PartialEq, Eq)]
pub enum UserCourseRole {
    Student,
    Creator,
}