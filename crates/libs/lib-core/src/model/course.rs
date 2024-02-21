use serde::Deserialize;

use super::base::DbBmc;

#[derive(Deserialize)] 
pub struct CourseForCreate {
	title: String,
	description: String,
	course_type: String,
	price: f64,
	color: String,
}

pub struct CourseBmc;

impl DbBmc for CourseBmc {
    const TABLE: &'static str = "course";
}