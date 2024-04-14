use sea_query::Iden;


#[derive(Iden)]
pub enum CommonIden {
	Id,
	OwnerId,
}

#[derive(Iden)]
pub enum TimestampIden {
	Cid,
	Ctime,
	Mid,
	Mtime,
}

#[derive(Iden)]
pub enum UserIden {
    User, // Table name
	Username,
}

#[derive(Iden)]
pub enum CourseIden {
    Course, // Table name
}

#[derive(Iden)]
pub enum UserCourseIden {
    UsersCourses, // Table name
	CourseId,
	UserId,
    UserRole,
}

#[derive(Iden)]
pub enum LessonIden {
	CourseId,
    LessonOrder,
}
