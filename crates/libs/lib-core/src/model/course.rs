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

#[derive(PartialEq, Eq)]
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

pub enum UserCourseRole {
    Student,
    Creator,
}