use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct UserPayload {
    pub id: i64,
    pub username: String,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct GetAttendatsPayload {
    pub course_id: i64,
	pub list_options: Option<String>,
}