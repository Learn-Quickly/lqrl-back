#[derive(Clone)]
pub struct Lesson {
    pub id: i64,
    pub course_id: i64,
    pub titile: String,
    pub lesson_order: i32,
}

pub struct LessonForCreate {
    pub course_id: i64,
    pub titile: String,
}

pub struct LessonForCreateCommand {
    pub course_id: i64,
    pub titile: String,
    pub order: i32,
}

pub struct LessonForUpdate {
    pub id: i64,
    pub titile: String,
}

#[derive(Debug, PartialEq)]
pub struct LessonForChangeOreder {
    pub id: i64,
    pub order: i32,
}