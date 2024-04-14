use modql::SIden;
use sea_query::{IntoIden, TableRef};

pub fn get_user_table_ref() -> TableRef {
	TableRef::Table(SIden("user").into_iden())
}

pub fn get_course_table_ref() -> TableRef {
	TableRef::Table(SIden("course").into_iden())
}

pub fn get_users_courses_table_ref() -> TableRef {
	TableRef::Table(SIden("users_courses").into_iden())
}

pub fn get_lesson_table_ref() -> TableRef {
	TableRef::Table(SIden("lesson").into_iden())
}