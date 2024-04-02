use uuid::Uuid;

pub struct User {
	pub id: i64,
	pub username: String,

	// -- pwd and token info
	pub pwd: String, // encrypted, #_scheme_id_#....
	pub pwd_salt: Uuid,
	pub token_salt: Uuid,
}

pub struct UserForCreate {
	pub username: String,
	pub pwd_clear: String,
}

pub struct UserForUpdate {
	pub username: String,
}

pub struct UserForChangePassword {
	pub pwd_clear: String,
	pub new_pwd_clear: String,
}