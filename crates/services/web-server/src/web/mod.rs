// region:    --- Modules

mod error;
pub mod mw_auth;
pub mod mw_req_stamp;
pub mod mw_res_map;
pub mod routes_register;
pub mod routes_login;
pub mod routes_course;
pub mod routes_static;
pub mod file_upload;

pub use self::error::ClientError;
pub use self::error::{Error, Result};
