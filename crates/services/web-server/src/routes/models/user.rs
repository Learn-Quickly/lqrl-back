use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct UserPayload {
    pub id: i64,
    pub username: String,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct GetAttendatsPayload {
    #[param(example = 1000)]
    pub course_id: i64,
    #[param(example = "{\"limit\": 5, \"offset\": 2}")]
	pub list_options: Option<String>,
}