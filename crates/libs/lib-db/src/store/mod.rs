// region:    --- Modules

pub(crate) mod dbx;
pub mod error;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

use crate::config::core_config;

use self::dbx::Dbx;
use self::error::{DbError, DbResult};

// endregion: --- Modules

pub type Db = Pool<Postgres>;

#[derive(Clone)]
pub struct DbManager {
	dbx: Dbx,
}

impl DbManager {
	/// Constructor
	pub async fn new() -> DbResult<Self> {
		let db_pool = new_db_pool()
			.await
			.map_err(|ex| DbError::CantCreateDbManagerProvider(ex.to_string()))?;
		let dbx = Dbx::new(db_pool, false)?;
		Ok(DbManager { dbx })
	}

	pub fn new_with_txn(&self) -> DbResult<Self> {
		let dbx = Dbx::new(self.dbx.db().clone(), true)?;
		Ok(DbManager { dbx })
	}

	pub fn dbx(&self) -> &Dbx {
		&self.dbx
	}
}

pub async fn new_db_pool() -> sqlx::Result<Db> {
	// * See NOTE 1) below
	let max_connections = if cfg!(test) { 1 } else { 5 };

	PgPoolOptions::new()
		.max_connections(max_connections)
		.connect(&core_config().DB_URL)
		.await
}

// NOTE 1) This is not an ideal situation; however, with sqlx 0.7.1, when executing `cargo test`, some tests that use sqlx fail at a
//         rather low level (in the tokio scheduler). It appears to be a low-level thread/async issue, as removing/adding
//         tests causes different tests to fail. The cause remains uncertain, but setting max_connections to 1 resolves the issue.
//         The good news is that max_connections still function normally for a regular run.
//         This issue is likely due to the unique requirements unit tests impose on their execution, and therefore,
//         while not ideal, it should serve as an acceptable temporary solution.
//         It's a very challenging issue to investigate and narrow down. The alternative would have been to stick with sqlx 0.6.x, which
//         is potentially less ideal and might lead to confusion as to why we are maintaining the older version in this blueprint.
