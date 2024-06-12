use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::config::core_config;

use super::{dbx::Dbx, error::{DbError, DbResult}};

pub type Db = Pool<Postgres>;

#[derive(Clone)]
pub struct DbManager {
	dbx: Dbx,
}

impl DbManager {
	pub async fn new() -> DbResult<Self> {
		let db_pool = new_db_pool()
			.await
			.map_err(|ex| DbError::CantCreateDbManagerProvider(ex.to_string()))?;
		let dbx = Dbx::new(db_pool, false).map_err(|err| DbError::Dbx(err.to_string()))?;
		Ok(DbManager { dbx })
	}

	pub fn new_with_txn(&self) -> DbResult<Self> {
		let dbx = Dbx::new(self.dbx.db().clone(), true).map_err(|err| DbError::Dbx(err.to_string()))?;
		Ok(DbManager { dbx })
	}

	pub fn dbx(&self) -> &Dbx {
		&self.dbx
	}
}

pub async fn new_db_pool() -> sqlx::Result<Db> {
	let max_connections = if cfg!(test) { 1 } else { 8 };

	PgPoolOptions::new()
		.max_connections(max_connections)
		.connect(&core_config().DB_URL)
		.await
}
