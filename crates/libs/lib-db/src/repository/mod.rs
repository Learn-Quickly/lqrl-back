mod base;
mod error;
mod store;

pub mod modql_utils;
pub mod user;
pub mod course;
pub mod users_courses;

pub use self::error::DbError;
pub use self::error::Result;

use crate::repository::store::dbx::Dbx;
use crate::repository::store::new_db_pool;

// endregion: --- Modules

// region:    --- DbManager

#[derive(Clone)]
pub struct DbManager {
	dbx: Dbx,
}

impl DbManager {
	/// Constructor
	pub async fn new() -> Result<Self> {
		let db_pool = new_db_pool()
			.await
			.map_err(|ex| DbError::CantCreateDbManagerProvider(ex.to_string()))?;
		let dbx = Dbx::new(db_pool, false)?;
		Ok(DbManager { dbx })
	}

	pub fn new_with_txn(&self) -> Result<Self> {
		let dbx = Dbx::new(self.dbx.db().clone(), true)?;
		Ok(DbManager { dbx })
	}

	pub fn dbx(&self) -> &Dbx {
		&self.dbx
	}
}

// endregion: --- DbManager
